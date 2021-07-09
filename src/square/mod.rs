//! Square 2d Lattice Graph. It does not use [`lattice_abstract`](`crate::lattice_abstract`) for historical and performance reason.

use crate::{fixedvec2d::FixedVec2D, unreachable_debug_checked};
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
    iter::FusedIterator, marker::PhantomData, num::NonZeroUsize, ops::Range, slice::Iter, usize,
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
    fn get_sizeinfo(_h: usize, _v: usize) -> Self::SizeInfo {
        ()
    }
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
    nodes: FixedVec2D<N>,
    horizontal: FixedVec2D<E>, //→
    vertical: FixedVec2D<E>,   //↑
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
        nodes: FixedVec2D<N>,
        horizontal: FixedVec2D<E>,
        vertical: FixedVec2D<E>,
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
        let nzh = NonZeroUsize::new(h).expect("h must be non zero");
        let mut nodes = unsafe { FixedVec2D::new_uninit(nzh, v) };
        let nodesref = nodes.mut_2d();
        let mh = if <S as Shape>::LOOP_HORIZONTAL {
            nzh
        } else {
            unsafe { NonZeroUsize::new_unchecked(h - 1) }
        };
        let mut horizontal = unsafe { FixedVec2D::new_uninit(mh, v) };
        let href = horizontal.mut_2d();
        let mv = if <S as Shape>::LOOP_VERTICAL {
            v
        } else {
            v - 1
        };
        let mut vertical = unsafe { FixedVec2D::new_uninit(nzh, mv) };
        let vref = vertical.mut_2d();

        for hi in 0..mh.get() {
            let nv = &mut nodesref[hi];
            let hv = &mut href[hi];
            let vv = &mut vref[hi];
            for vi in 0..mv {
                unsafe {
                    core::ptr::write(nv.get_unchecked_mut(vi), fnode(hi, vi));
                    core::ptr::write(hv.get_unchecked_mut(vi), fedge(hi, vi, Axis::Horizontal));
                    core::ptr::write(vv.get_unchecked_mut(vi), fedge(hi, vi, Axis::Vertical));
                }
            }
            if !<S as Shape>::LOOP_VERTICAL {
                unsafe {
                    core::ptr::write(nv.get_unchecked_mut(v - 1), fnode(hi, v - 1));
                    core::ptr::write(
                        hv.get_unchecked_mut(v - 1),
                        fedge(hi, v - 1, Axis::Horizontal),
                    );
                }
            }
        }
        if !<S as Shape>::LOOP_HORIZONTAL {
            let nv = &mut nodesref[h - 1];
            let vv = &mut vref[h - 1];
            for hi in 0..mv {
                unsafe {
                    core::ptr::write(nv.get_unchecked_mut(hi), fnode(h - 1, hi));
                    core::ptr::write(vv.get_unchecked_mut(hi), fedge(h - 1, hi, Axis::Vertical));
                }
            }
            if !<S as Shape>::LOOP_VERTICAL {
                unsafe { core::ptr::write(nv.get_unchecked_mut(v - 1), fnode(h - 1, v - 1)) };
            }
        }
        unsafe { Self::new_raw(nodes, horizontal, vertical) }
    }

    /// Check the size of nodes and edges.
    fn check_gen(&self) -> bool {
        self.nodes.h_size()
            == self.horizontal.h_size() + if <S as Shape>::LOOP_HORIZONTAL { 0 } else { 1 }
            && self.nodes.v_size() == self.horizontal.v_size()
            && self.nodes.h_size() == self.vertical.h_size()
            && self.nodes.v_size()
                == self.vertical.v_size() + if <S as Shape>::LOOP_VERTICAL { 0 } else { 1 }
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
    pub fn get_edge_reference<'a>(
        &'a self,
        n: NodeIndex<Ix>,
        dir: SquareDirection,
    ) -> Option<EdgeReference<'a, E, Ix, S>> {
        self.get_edge_id(n, dir).map(|(e, fo)| EdgeReference {
            edge_id: e,
            edge_weight: unsafe {
                if dir.is_horizontal() {
                    &self.horizontal
                } else {
                    &self.vertical
                }
                .ref_2d()
                .get_unchecked(e.node.horizontal.index())
                .get_unchecked(e.node.vertical.index())
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
        self.nodes.h_size()
    }

    /// Returns the Node count in the vertical direction.
    pub fn vertical_node_count(&self) -> usize {
        self.nodes.v_size()
    }

    /// Get a reference to the nodes. `[horizontal][vertical]`
    pub fn nodes(&self) -> &[&[N]] {
        self.nodes.as_ref()
    }

    /// Get a reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal(&self) -> &[&[E]] {
        self.horizontal.as_ref()
    }

    /// Get a reference to the vertical edges. `[horizontal][vertical]`
    pub fn vertical(&self) -> &[&[E]] {
        self.vertical.as_ref()
    }

    /// Get a mutable reference to the nodes. `[horizontal][vertical]`
    pub fn nodes_mut(&mut self) -> &mut [&mut [N]] {
        self.nodes.as_mut()
    }

    /// Get a mutable reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal_mut(&mut self) -> &[&mut [E]] {
        self.horizontal.as_mut()
    }

    /// Get a mutable reference to the vertical edges.
    pub fn vertical_mut(&mut self) -> &[&mut [E]] {
        self.vertical.as_mut()
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
    fn node_weight(self: &Self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.nodes
            .ref_2d()
            .get(id.horizontal.index())?
            .get(id.vertical.index())
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        match id.axis {
            Axis::Horizontal => &self.horizontal,
            Axis::Vertical => &self.vertical,
        }
        .ref_2d()
        .get(id.node.horizontal.index())?
        .get(id.node.vertical.index())
    }
}

impl<N, E, Ix, S> DataMapMut for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    fn node_weight_mut(self: &mut Self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.nodes
            .mut_2d()
            .get_mut(id.horizontal.index())?
            .get_mut(id.vertical.index())
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        match id.axis {
            Axis::Horizontal => &mut self.horizontal,
            Axis::Vertical => &mut self.vertical,
        }
        .mut_2d()
        .get_mut(id.node.horizontal.index())?
        .get_mut(id.node.vertical.index())
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

    fn visit_map(self: &Self) -> Self::Map {
        VisMap::new(self.horizontal_node_count(), self.vertical_node_count())
    }

    fn reset_map(self: &Self, map: &mut Self::Map) {
        map.v.iter_mut().for_each(|x| x.clear())
    }
}
