//! Square 2d Lattice Graph. It does not use [`lattice_abstract`](`crate::lattice_abstract`) for historical and performance reason.

use crate::unreachable_debug_checked;
use ndarray::Array2;

use fixedbitset::FixedBitSet;
use petgraph::{
    data::{DataMap, DataMapMut},
    graph::IndexType,
    visit::{
        Data, GraphBase, GraphProp, IntoNodeIdentifiers, IntoNodeReferences, NodeIndexable,
        VisitMap, Visitable,
    },
    Undirected,
};
use std::{
    iter::FusedIterator, marker::PhantomData, ops::Range, usize,
};

mod edges;
pub use edges::*;
mod index;
pub use index::*;
mod neighbors;
pub use neighbors::*;
mod nodes;
pub use nodes::*;

#[cfg(test)]
mod tests;

/// Shape of the [`SquareGraph`]. It tells that the graph loops or not.
pub trait Shape: Copy {
    /// SizeInfo is needed if loop is enabled.
    type SizeInfo: SizeInfo;
    /// Whether the graph loops in horizontal axis.
    const LOOP_HORIZONTAL: bool = false;
    /// Whether the graph loops in vertical axis.
    const LOOP_VERTICAL: bool = false;
    /// Get a size info used in [`EdgeReference`].
    fn get_sizeinfo(h: usize, v: usize) -> Self::SizeInfo;
}

/// It holds a infomation of size of graph if needed.
/// This is used in [`EdgeReference`] to tell the loop info.
/// This trick is to optimize when there is no loop in graph.
pub trait SizeInfo: Copy {
    /// Should only be called when [`Shape::LOOP_HORIZONTAL`] is true.
    unsafe fn horizontal_size(&self) -> usize {
        unreachable_debug_checked()
    }
    /// Should only be called when [`Shape::LOOP_VERTICAL`] is true.
    unsafe fn vertical_size(&self) -> usize {
        unreachable_debug_checked()
    }
}

impl SizeInfo for () {}
impl SizeInfo for (usize, ()) {
    unsafe fn horizontal_size(&self) -> usize {
        self.0
    }
}
impl SizeInfo for ((), usize) {
    unsafe fn vertical_size(&self) -> usize {
        self.1
    }
}
impl SizeInfo for (usize, usize) {
    unsafe fn horizontal_size(&self) -> usize {
        self.0
    }
    unsafe fn vertical_size(&self) -> usize {
        self.1
    }
}

/// Marker that the graph does not loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DefaultShape {}
impl Shape for DefaultShape {
    type SizeInfo = ();
    #[inline]
    fn get_sizeinfo(_h: usize, _v: usize) -> Self::SizeInfo {}
}
/// Marker that the graph does loops in horizontal axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HorizontalLoop {}
impl Shape for HorizontalLoop {
    type SizeInfo = (usize, ());
    const LOOP_HORIZONTAL: bool = true;
    #[inline]
    fn get_sizeinfo(h: usize, _v: usize) -> Self::SizeInfo {
        (h, ())
    }
}
/// Marker that the graph does loops in vertical axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerticalLoop {}
impl Shape for VerticalLoop {
    type SizeInfo = ((), usize);
    const LOOP_VERTICAL: bool = true;
    #[inline]
    fn get_sizeinfo(_h: usize, v: usize) -> Self::SizeInfo {
        ((), v)
    }
}
/// Marker that the graph does loops in horizontal and vertical axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HVLoop {}
impl Shape for HVLoop {
    type SizeInfo = (usize, usize);
    const LOOP_VERTICAL: bool = true;
    const LOOP_HORIZONTAL: bool = true;
    #[inline]
    fn get_sizeinfo(h: usize, v: usize) -> Self::SizeInfo {
        (h, v)
    }
}

/// Undirected Square Grid Graph. It is has rectangle shape.
/// ```text
/// Node(i,j+1) - Edge(i,j+1,Horizontal) - Node(i+1,j+1)
///   |                                     |
/// Edge(i,j,Vertical)                     Edge(i+1,j,Vertical)
///   |                                     |
/// Node(i,j)   - Edge(i,j,Horizontal)   - Node(i+1,j)
/// ```
#[derive(Clone, Debug)]
pub struct SquareGraph<N, E, Ix = usize, S = DefaultShape>
where
    Ix: IndexType,
{
    /// `[horizontal][vertical]`
    nodes: Array2<N>,
    horizontal: Array2<E>, //→
    vertical: Array2<E>,   //↑
    s: PhantomData<fn() -> S>,
    pd: PhantomData<fn() -> Ix>,
}

impl<N, E, Ix, S> SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
    S: Shape,
{
    /// Create a `SquareGraph` from raw data.
    /// It only check whether the size of nodes and edges are correct in `debug_assertion`.
    pub unsafe fn new_raw(
        nodes: Array2<N>,
        horizontal: Array2<E>,
        vertical: Array2<E>,
    ) -> Self {
        let s = Self {
            nodes,
            horizontal,
            vertical,
            s: PhantomData,
            pd: PhantomData,
        };
        debug_assert!(s.check_gen());
        s
    }

    /// Create a `SquareGraph` with the nodes and edges initialized with default.
    pub fn new(h: usize, v: usize) -> Self
    where
        N: Default,
        E: Default,
    {
        Self::new_with(h, v, |_, _| N::default(), |_, _, _| E::default())
    }

    /// Creates a `SquareGraph` with initializing nodes and edges from position.
    pub fn new_with<FN, FE>(h: usize, v: usize, mut fnode: FN, mut fedge: FE) -> Self
    where
        FN: FnMut(usize, usize) -> N,
        FE: FnMut(usize, usize, Axis) -> E,
    {
        assert!(h > 0, "h must be non zero");
        
        // Initialize nodes array
        let mut nodes_vec = Vec::with_capacity(h * v);
        for hi in 0..h {
            for vi in 0..v {
                nodes_vec.push(fnode(hi, vi));
            }
        }
        let nodes = Array2::from_shape_vec((h, v), nodes_vec).expect("Array2 creation failed");
        
        // Initialize horizontal edges
        let mh = if <S as Shape>::LOOP_HORIZONTAL { h } else { h - 1 };
        let mut horizontal_vec = Vec::with_capacity(mh * v);
        for hi in 0..mh {
            for vi in 0..v {
                horizontal_vec.push(fedge(hi, vi, Axis::Horizontal));
            }
        }
        let horizontal = Array2::from_shape_vec((mh, v), horizontal_vec).expect("Array2 creation failed");
        
        // Initialize vertical edges
        let mv = if <S as Shape>::LOOP_VERTICAL { v } else { v - 1 };
        let mut vertical_vec = Vec::with_capacity(h * mv);
        for hi in 0..h {
            for vi in 0..mv {
                vertical_vec.push(fedge(hi, vi, Axis::Vertical));
            }
        }
        let vertical = Array2::from_shape_vec((h, mv), vertical_vec).expect("Array2 creation failed");
        
        unsafe {
            Self::new_raw(nodes, horizontal, vertical)
        }
    }

    /// Check the size of nodes and edges.
    fn check_gen(&self) -> bool {
        self.nodes.nrows()
            == self.horizontal.nrows() + if <S as Shape>::LOOP_HORIZONTAL { 0 } else { 1 }
            && self.nodes.ncols() == self.horizontal.ncols()
            && self.nodes.nrows() == self.vertical.nrows()
            && self.nodes.ncols()
                == self.vertical.ncols() + if <S as Shape>::LOOP_VERTICAL { 0 } else { 1 }
    }

    #[inline]
    /// Get the edge from node.
    pub fn get_edge_id(
        &self,
        node: NodeIndex<Ix>,
        dir: SquareDirection,
    ) -> Option<(EdgeIndex<Ix>, bool)> {
        let x = match dir {
            SquareDirection::Foward(a @ Axis::Vertical)
                if node.vertical.index() + 1 < self.vertical_node_count() =>
            {
                (node, a, true)
            }
            SquareDirection::Foward(a @ Axis::Vertical)
                if <S as Shape>::LOOP_VERTICAL
                    && node.vertical.index() + 1 == self.vertical_node_count() =>
            {
                (
                    NodeIndex {
                        horizontal: node.horizontal,
                        vertical: Ix::default(),
                    },
                    a,
                    true,
                )
            }
            SquareDirection::Foward(a @ Axis::Horizontal)
                if node.horizontal.index() + 1 < self.horizontal_node_count() =>
            {
                (node, a, true)
            }
            SquareDirection::Foward(a @ Axis::Horizontal)
                if <S as Shape>::LOOP_HORIZONTAL
                    && node.horizontal.index() + 1 == self.horizontal_node_count() =>
            {
                (
                    NodeIndex {
                        horizontal: Ix::default(),
                        vertical: node.vertical,
                    },
                    a,
                    true,
                )
            }
            SquareDirection::Backward(a @ Axis::Vertical) if node.vertical.index() != 0 => {
                (node.down(), a, false)
            }
            SquareDirection::Backward(a @ Axis::Vertical)
                if <S as Shape>::LOOP_VERTICAL && node.vertical.index() == 0 =>
            {
                (
                    NodeIndex {
                        horizontal: node.horizontal,
                        vertical: Ix::new(self.vertical_node_count() - 1),
                    },
                    a,
                    false,
                )
            }
            SquareDirection::Backward(a @ Axis::Horizontal) if node.horizontal.index() != 0 => {
                (node.left(), a, false)
            }
            SquareDirection::Backward(a @ Axis::Horizontal)
                if <S as Shape>::LOOP_HORIZONTAL && node.horizontal.index() == 0 =>
            {
                (
                    NodeIndex {
                        horizontal: Ix::new(self.horizontal_node_count() - 1),
                        vertical: node.vertical,
                    },
                    a,
                    false,
                )
            }
            _ => return None,
        };
        Some(((x.0, x.1).into(), x.2))
    }

    #[inline]
    /// Get the edge reference form node.
    pub fn get_edge_reference(
        &self,
        n: NodeIndex<Ix>,
        dir: SquareDirection,
    ) -> Option<EdgeReference<'_, E, Ix, S>> {
        self.get_edge_id(n, dir).map(|(e, fo)| EdgeReference {
            edge_id: e,
            edge_weight: unsafe {
                if dir.is_horizontal() {
                    self.horizontal.uget((e.node.horizontal.index(), e.node.vertical.index()))
                } else {
                    self.vertical.uget((e.node.horizontal.index(), e.node.vertical.index()))
                }
            },
            direction: fo,
            s: S::get_sizeinfo(self.horizontal_node_count(), self.vertical_node_count()),
            spd: PhantomData,
        })
    }
}

impl<N, E, Ix, S> SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    /// Returns the Node count in the horizontal direction.
    pub fn horizontal_node_count(&self) -> usize {
        self.nodes.nrows()
    }

    /// Returns the Node count in the vertical direction.
    pub fn vertical_node_count(&self) -> usize {
        self.nodes.ncols()
    }

    /// Get a reference to the nodes. `[horizontal][vertical]`
    pub fn nodes(&self) -> &Array2<N> {
        &self.nodes
    }

    /// Get a reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal(&self) -> &Array2<E> {
        &self.horizontal
    }

    /// Get a reference to the vertical edges. `[horizontal][vertical]`
    pub fn vertical(&self) -> &Array2<E> {
        &self.vertical
    }

    /// Get a mutable reference to the nodes. `[horizontal][vertical]`
    pub fn nodes_mut(&mut self) -> &mut Array2<N> {
        &mut self.nodes
    }

    /// Get a mutable reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal_mut(&mut self) -> &mut Array2<E> {
        &mut self.horizontal
    }

    /// Get a mutable reference to the vertical edges.
    pub fn vertical_mut(&mut self) -> &mut Array2<E> {
        &mut self.vertical
    }
}

impl<E, Ix, S> SquareGraph<(), E, Ix, S>
where
    Ix: IndexType,
    S: Shape,
{
    /// Create a `SquareGraph` with the edges initialized from position.
    pub fn new_edge_graph<FE>(h: usize, v: usize, fedge: FE) -> Self
    where
        FE: FnMut(usize, usize, Axis) -> E,
    {
        Self::new_with(h, v, |_, _| (), fedge)
    }
}

impl<N, E, Ix, S> GraphBase for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
}

impl<N, E, Ix, S> Data for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, Ix, S> DataMap for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.nodes.get((id.horizontal.index(), id.vertical.index()))
    }

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        match id.axis {
            Axis::Horizontal => &self.horizontal,
            Axis::Vertical => &self.vertical,
        }
        .get((id.node.horizontal.index(), id.node.vertical.index()))
    }
}

impl<N, E, Ix, S> DataMapMut for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.nodes.get_mut((id.horizontal.index(), id.vertical.index()))
    }

    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        match id.axis {
            Axis::Horizontal => &mut self.horizontal,
            Axis::Vertical => &mut self.vertical,
        }
        .get_mut((id.node.horizontal.index(), id.node.vertical.index()))
    }
}

impl<N, E, Ix, S> GraphProp for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    type EdgeType = Undirected;
}

/// [`VisitMap`] of [`SquareGraph`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VisMap {
    v: Vec<FixedBitSet>,
}

impl VisMap {
    pub fn new(h: usize, v: usize) -> Self {
        let mut vec = Vec::with_capacity(h);
        for _ in 0..h {
            vec.push(FixedBitSet::with_capacity(v));
        }
        Self { v: vec }
    }
}

impl<Ix: IndexType> VisitMap<NodeIndex<Ix>> for VisMap {
    fn visit(&mut self, a: NodeIndex<Ix>) -> bool {
        !self.v[a.horizontal.index()].put(a.vertical.index())
    }

    fn is_visited(&self, a: &NodeIndex<Ix>) -> bool {
        self.v[a.horizontal.index()].contains(a.vertical.index())
    }
}

impl<N, E, Ix, S> Visitable for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    type Map = VisMap;

    fn visit_map(&self) -> Self::Map {
        VisMap::new(self.horizontal_node_count(), self.vertical_node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.v.iter_mut().for_each(|x| x.clear())
    }
}
