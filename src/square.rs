use fixedbitset::FixedBitSet;
use itertools::*;
use petgraph::{
    data::{DataMap, DataMapMut},
    graph::IndexType,
    visit::{
        Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges, IntoNeighbors,
        IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable, NodeCount, NodeIndexable,
        VisitMap, Visitable,
    },
    Undirected,
};
use std::{
    iter::FusedIterator, marker::PhantomData, num::NonZeroUsize, ops::Range, slice::Iter, usize,
};

use crate::fixedvec2d::FixedVec2D;

/// Axis of the Square grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    /// Check whether axis is horizontal.
    pub fn is_horizontal(&self) -> bool {
        *self == Axis::Horizontal
    }
    /// Check whether axis is vertical.
    pub fn is_vertical(&self) -> bool {
        *self == Axis::Vertical
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SquareDirection {
    Foward(Axis),
    Backward(Axis),
}

impl SquareDirection {
    /// Foward Horizontal
    pub const fn up() -> Self {
        Self::Foward(Axis::Vertical)
    }
    /// Backward Horizontal
    pub const fn down() -> Self {
        Self::Backward(Axis::Vertical)
    }
    /// Backward Vertical
    pub const fn left() -> Self {
        Self::Backward(Axis::Horizontal)
    }
    /// Foward Vertical
    pub const fn right() -> Self {
        Self::Foward(Axis::Horizontal)
    }
}

impl From<(Axis, bool)> for SquareDirection {
    fn from((axis, dir): (Axis, bool)) -> Self {
        if dir {
            SquareDirection::Foward(axis)
        } else {
            SquareDirection::Backward(axis)
        }
    }
}

impl From<SquareDirection> for (Axis, bool) {
    fn from(dir: SquareDirection) -> Self {
        match dir {
            SquareDirection::Backward(x) => (x, false),
            SquareDirection::Foward(x) => (x, true),
        }
    }
}

/// Undirected Square Grid Graph.
/// ```text
/// Node(i,j+1) - Edge(i,j+1,Horizontal) - Node(i+1,j+1)
///   |                                     |
/// Edge(i,j,Vertical)                     Edge(i+1,j,Vertical)
///   |                                     |
/// Node(i,j)   - Edge(i,j,Horizontal)   - Node(i+1,j)
/// ```
#[derive(Clone, Debug)]
pub struct SquareGraph<N, E, Ix = usize>
where
    Ix: IndexType,
{
    /// `[horizontal][vertical]`
    nodes: FixedVec2D<N>,
    horizontal: FixedVec2D<E>, //→
    vertical: FixedVec2D<E>,   //↑
    pd: PhantomData<Ix>,
}

impl<N, E, Ix> SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    /// Create a `SquareGraph` from raw data.
    /// It only check whether the size of nodes and edges are correct in `debug_assertion`.
    pub unsafe fn new_raw(nodes: FixedVec2D<N>, horizontal: FixedVec2D<E>, vertical: FixedVec2D<E>) -> Self {
        let s = Self {
            nodes,
            horizontal,
            vertical,
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
        let mut horizontal = unsafe { FixedVec2D::new_uninit(NonZeroUsize::new_unchecked(h - 1), v) };
        let href = horizontal.mut_2d();
        let mut vertical = unsafe { FixedVec2D::new_uninit(nzh, v - 1) };
        let vref = vertical.mut_2d();

        for hi in 0..h - 1 {
            let nv = &mut nodesref[hi];
            let hv = &mut href[hi];
            let vv = &mut vref[hi];
            for vi in 0..v - 1 {
                unsafe {
                    *nv.get_unchecked_mut(vi) = fnode(hi, vi);
                    *hv.get_unchecked_mut(vi) = fedge(hi, vi, Axis::Horizontal);
                    *vv.get_unchecked_mut(vi) = fedge(hi, vi, Axis::Vertical);
                }
            }
            unsafe {
                *nv.get_unchecked_mut(v - 1) = fnode(hi, v - 1);
                *hv.get_unchecked_mut(v - 1) = fedge(hi, v - 1, Axis::Horizontal);
            }
        }
        let nv = &mut nodesref[h - 1];
        let vv = &mut vref[h - 1];
        for hi in 0..v - 1 {
            unsafe {
                *nv.get_unchecked_mut(hi) = fnode(h - 1, hi);
                *vv.get_unchecked_mut(hi) = fedge(h - 1, hi, Axis::Vertical);
            }
        }
        nv[v - 1] = fnode(h - 1, v - 1);
        unsafe { Self::new_raw(nodes, horizontal, vertical) }
    }

    /// Returns the Node count in the horizontal direction.
    pub fn horizontal_node_count(&self) -> usize {
        self.nodes.h_size()
    }

    /// Returns the Node count in the vertical direction.
    pub fn vertical_node_count(&self) -> usize {
        self.nodes.v_size()
    }

    /// Check the size of nodes and edges.
    fn check_gen(&self) -> bool {
        self.nodes.h_size() == self.horizontal.h_size() + 1
            && self.nodes.v_size() == self.horizontal.v_size()
            && self.nodes.h_size() == self.vertical.h_size()
            && self.nodes.v_size() == self.vertical.v_size() + 1
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

impl<E, Ix> SquareGraph<(), E, Ix>
where
    Ix: IndexType,
{
    /// Create a `SquareGraph` with the edges initialized from position.
    pub fn new_edge_graph<FE>(h: usize, v: usize, fedge: FE) -> Self
    where
        FE: FnMut(usize, usize, Axis) -> E,
    {
        Self::new_with(h, v, |_, _| (), fedge)
    }
}

/// Node index for [`SquareGraph`]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex<Ix: IndexType> {
    pub horizontal: Ix,
    pub vertical: Ix,
}

impl<Ix: IndexType> NodeIndex<Ix> {
    /// Create a Index from horizontal and vertical.
    pub fn new(horizontal: Ix, vertical: Ix) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Returns the manhattan distance
    pub fn distance<T: Into<(usize, usize)>>(&self, target: T) -> usize {
        let target: (usize, usize) = target.into();
        (self.horizontal.index() as isize - target.0 as isize).abs() as usize
            + (self.vertical.index() as isize - target.1 as isize).abs() as usize
    }

    /// Get the edge from this node. This does not check whether the node is valid in graph.
    pub fn get_edge_id(&self, dir: SquareDirection) -> EdgeIndex<Ix> {
        match dir {
            SquareDirection::Foward(x) => (*self, x),
            SquareDirection::Backward(a @ Axis::Vertical) => (
                Self::new(self.horizontal, Ix::new(self.vertical.index() - 1)),
                a,
            ),
            SquareDirection::Backward(a @ Axis::Horizontal) => (
                Self::new(Ix::new(self.horizontal.index() - 1), self.vertical),
                a,
            ),
        }
        .into()
    }
}

impl<Ix: IndexType> PartialEq<(usize, usize)> for NodeIndex<Ix> {
    fn eq(&self, value: &(usize, usize)) -> bool {
        &(self.horizontal.index(), self.vertical.index()) == value
    }
}

impl<Ix: IndexType> From<(usize, usize)> for NodeIndex<Ix> {
    fn from(value: (usize, usize)) -> Self {
        NodeIndex::new(Ix::new(value.0), Ix::new(value.1))
    }
}

impl<Ix: IndexType> From<NodeIndex<Ix>> for (usize, usize) {
    fn from(value: NodeIndex<Ix>) -> Self {
        (value.horizontal.index(), value.vertical.index())
    }
}

/// Edge Index of [`SquareGraph`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIndex<Ix: IndexType>(pub NodeIndex<Ix>, pub Axis);

impl<Ix: IndexType> From<(NodeIndex<Ix>, Axis)> for EdgeIndex<Ix> {
    fn from((n, a): (NodeIndex<Ix>, Axis)) -> Self {
        Self(n, a)
    }
}

impl<Ix: IndexType> From<(NodeIndex<Ix>, SquareDirection)> for EdgeIndex<Ix> {
    fn from((n, a): (NodeIndex<Ix>, SquareDirection)) -> Self {
        n.get_edge_id(a)
    }
}

impl<N, E, Ix> GraphBase for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
}

impl<N, E, Ix> Data for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, Ix> DataMap for SquareGraph<N, E, Ix>
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
        match id.1 {
            Axis::Horizontal => &self.horizontal,
            Axis::Vertical => &self.vertical,
        }
        .ref_2d()
        .get(id.0.horizontal.index())?
        .get(id.0.vertical.index())
    }
}

impl<N, E, Ix> DataMapMut for SquareGraph<N, E, Ix>
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
        match id.1 {
            Axis::Horizontal => &mut self.horizontal,
            Axis::Vertical => &mut self.vertical,
        }
        .mut_2d()
        .get_mut(id.0.horizontal.index())?
        .get_mut(id.0.vertical.index())
    }
}

impl<N, E, Ix> GraphProp for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type EdgeType = Undirected;
}

impl<'a, N, E, Ix> IntoEdgeReferences for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    E: Copy,
    Range<Ix>: Iterator<Item = Ix>,
{
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences::new(&self)
    }
}

/// Reference of Edge data (EdgeIndex, EdgeWeight, direction) in [`SquareGraph`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeReference<'a, E, Ix: IndexType> {
    edge_id: EdgeIndex<Ix>,
    edge_weight: &'a E,
    direction: bool,
}

impl<'a, E, Ix: IndexType> Clone for EdgeReference<'a, E, Ix> {
    fn clone(&self) -> Self {
        Self {
            edge_id: self.edge_id,
            edge_weight: self.edge_weight,
            direction: self.direction,
        }
    }
}

impl<'a, E, Ix: IndexType> Copy for EdgeReference<'a, E, Ix> {}

impl<'a, E, Ix: IndexType> EdgeReference<'a, E, Ix> {
    fn get_node(&self, is_source: bool) -> NodeIndex<Ix> {
        if is_source {
            (self.edge_id).0
        } else {
            match (self.edge_id).1 {
                Axis::Horizontal => NodeIndex::new(
                    Ix::new((self.edge_id).0.horizontal.index() + 1),
                    (self.edge_id).0.vertical,
                ),
                Axis::Vertical => NodeIndex::new(
                    (self.edge_id).0.horizontal,
                    Ix::new((self.edge_id).0.vertical.index() + 1),
                ),
            }
        }
    }
}

impl<'a, E: Copy, Ix: IndexType> EdgeRef for EdgeReference<'a, E, Ix> {
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.get_node(self.direction)
    }

    fn target(&self) -> Self::NodeId {
        self.get_node(!self.direction)
    }

    fn weight(&self) -> &Self::Weight {
        self.edge_weight
    }

    fn id(&self) -> Self::EdgeId {
        self.edge_id
    }
}

/// Iterator for all edges of [`SquareGraph`].
#[derive(Clone, Debug)]
pub struct EdgeReferences<'a, E, Ix: IndexType> {
    horizontal: &'a FixedVec2D<E>,
    vertical: &'a FixedVec2D<E>,
    nodes: NodeIndices<Ix>,
    prv: Option<EdgeIndex<Ix>>,
}

impl<'a, E, Ix: IndexType> EdgeReferences<'a, E, Ix> {
    fn new<N>(graph: &'a SquareGraph<N, E, Ix>) -> Self {
        Self {
            horizontal: &graph.horizontal,
            vertical: &graph.vertical,
            nodes: NodeIndices::new(graph.horizontal_node_count(), graph.vertical_node_count()),
            prv: None,
        }
    }
}

impl<'a, E, Ix> Iterator for EdgeReferences<'a, E, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut e) = self.prv {
                if e.1 == Axis::Horizontal {
                    let item =
                        self.vertical.ref_2d()[e.0.horizontal.index()].get(e.0.vertical.index());
                    if let Some(item) = item {
                        e.1 = Axis::Vertical;
                        return Some(EdgeReference {
                            edge_id: *e,
                            edge_weight: item,
                            direction: true,
                        });
                    }
                }
            }
            if let Some(next) = self.nodes.next() {
                let item = self
                    .horizontal
                    .ref_2d()
                    .get(next.horizontal.index())
                    .map(|x| x.get(next.vertical.index()))
                    .flatten();
                let edge_id = EdgeIndex(
                    NodeIndex::new(next.horizontal, next.vertical),
                    Axis::Horizontal,
                );
                self.prv = Some(edge_id);
                if let Some(edge_weight) = item {
                    return Some(EdgeReference {
                        edge_id,
                        edge_weight,
                        direction: true,
                    });
                }
            } else {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.nodes.size_hint();
        (
            lo.saturating_sub(self.horizontal.size() + self.vertical.size()),
            hi.map(|x| x * 2),
        )
    }
}

pub struct Edges<'a, N, E, Ix: IndexType> {
    g: &'a SquareGraph<N, E, Ix>,
    node: NodeIndex<Ix>,
    state: usize,
}

impl<'a, N, E, Ix> Iterator for Edges<'a, N, E, Ix>
where
    Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        let g = self.g;
        let n = self.node;
        loop {
            'inner: loop {
                let er = match self.state {
                    0 => {
                        if n.horizontal.index() == 0 {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex(
                                NodeIndex::new(Ix::new(n.horizontal.index() - 1), n.vertical),
                                Axis::Horizontal,
                            ),
                            edge_weight: unsafe {
                                g.horizontal
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index() - 1)
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: false,
                        }
                    }
                    1 => {
                        if n.horizontal.index() + 1 >= g.horizontal_node_count() {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex(n, Axis::Horizontal),
                            edge_weight: unsafe {
                                g.horizontal
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: true,
                        }
                    }
                    2 => {
                        if n.vertical.index() == 0 {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex(
                                NodeIndex::new(n.horizontal, Ix::new(n.vertical.index() - 1)),
                                Axis::Vertical,
                            ),
                            edge_weight: unsafe {
                                g.vertical
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index() - 1)
                            },
                            direction: false,
                        }
                    }
                    3 => {
                        if n.vertical.index() + 1 >= g.vertical_node_count() {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex(n, Axis::Vertical),
                            edge_weight: unsafe {
                                g.vertical
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: true,
                        }
                    }
                    _ => return None,
                };
                self.state += 1;
                return Some(er);
            }
            self.state += 1;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(4 - self.state))
    }
}

impl<'a, N, E, Ix> FusedIterator for Edges<'a, N, E, Ix> where Ix: IndexType {}

impl<'a, N, E, Ix> IntoEdges for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    E: Copy,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Edges = Edges<'a, N, E, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        Edges {
            g: &self,
            node: a,
            state: 0,
        }
    }
}

pub struct Neighbors<Ix: IndexType> {
    node: NodeIndex<Ix>,
    state: usize,
    h: usize,
    v: usize,
}

impl<Ix> Iterator for Neighbors<Ix>
where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let n = self.node;
            let x = match self.state {
                0 if n.horizontal.index() != 0 => Some(NodeIndex::new(
                    Ix::new(n.horizontal.index() - 1),
                    n.vertical,
                )),
                1 if n.horizontal.index() + 1 < self.h => Some(NodeIndex::new(
                    Ix::new(n.horizontal.index() + 1),
                    n.vertical,
                )),
                2 if n.vertical.index() != 0 => Some(NodeIndex::new(
                    n.horizontal,
                    Ix::new(n.vertical.index() - 1),
                )),
                3 if n.vertical.index() + 1 < self.v => Some(NodeIndex::new(
                    n.horizontal,
                    Ix::new(n.vertical.index() + 1),
                )),
                4..=usize::MAX => None,
                _ => {
                    self.state += 1;
                    continue;
                }
            };
            self.state += 1;
            return x;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(4 - self.state))
    }
}

impl<Ix> FusedIterator for Neighbors<Ix> where Ix: IndexType {}

impl<'a, N, E, Ix> IntoNeighbors for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Neighbors = Neighbors<Ix>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            node: a,
            state: 0,
            h: self.horizontal_node_count(),
            v: self.vertical_node_count(),
        }
    }
}

impl<'a, N, E, Ix> IntoNodeIdentifiers for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type NodeIdentifiers = NodeIndices<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices::new(self.horizontal_node_count(), self.vertical_node_count())
    }
}

/// Iterate all index of [`SquareGraph`].
#[derive(Clone, Debug)]
pub struct NodeIndices<Ix> {
    p: itertools::Product<Range<usize>, Range<usize>>,
    pd: PhantomData<Ix>,
}

impl<Ix> NodeIndices<Ix> {
    fn new(h: usize, v: usize) -> Self {
        Self {
            p: (0..h).cartesian_product(0..v),
            pd: PhantomData,
        }
    }
}

impl<Ix: IndexType> Iterator for NodeIndices<Ix>
where
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.p.next().map(|x| (x.0, x.1).into())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.p.size_hint()
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        self.p.fold(init, |x, item| (&mut f)(x, item.into()))
    }
}

impl<Ix: IndexType> FusedIterator for NodeIndices<Ix> where Range<Ix>: Iterator<Item = Ix> {}

impl<'a, N: Clone, E, Ix> IntoNodeReferences for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type NodeRef = (NodeIndex<Ix>, &'a N);
    type NodeReferences = NodeReferences<'a, N, Ix>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences {
            indices: self.node_identifiers(),
            nodes: self.nodes.ref_1d().iter(),
        }
    }
}

/// Iterate all nodes of [`SquareGraph`].
pub struct NodeReferences<'a, N, Ix> {
    indices: NodeIndices<Ix>,
    nodes: Iter<'a, N>,
}

impl<'a, N, Ix> Iterator for NodeReferences<'a, N, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = (NodeIndex<Ix>, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.nodes.next()?;
        Some((self.indices.next()?, n))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.nodes.count()
    }
}

impl<N, E, Ix> NodeCompactIndexable for SquareGraph<N, E, Ix> where Ix: IndexType {}

impl<N, E, Ix> NodeCount for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_count(self: &Self) -> usize {
        self.nodes.size()
    }
}

impl<N, E, Ix> NodeIndexable for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_bound(self: &Self) -> usize {
        self.nodes.size()
    }

    fn to_index(self: &Self, a: Self::NodeId) -> usize {
        a.horizontal.index() * self.vertical_node_count() + a.vertical.index()
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        let h = self.vertical_node_count();
        (i / h, i % h).into()
    }
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

impl<N, E, Ix> Visitable for SquareGraph<N, E, Ix>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            4,
            3,
            |x, y| x + 2 * y,
            |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
        );
        assert_eq!(sq.horizontal_node_count(), 4);
        assert_eq!(sq.vertical_node_count(), 3);
        assert_eq!(sq.node_weight((0, 0).into()), Some(&0));
        assert_eq!(sq.node_weight((3, 0).into()), Some(&3));
        assert_eq!(sq.node_weight((4, 0).into()), None);
        assert_eq!(sq.node_weight((0, 2).into()), Some(&4));
        assert_eq!(sq.node_weight((0, 3).into()), None);
        assert_eq!(
            sq.edge_weight(((0, 0).into(), Axis::Horizontal).into()),
            Some(&0)
        );
        assert_eq!(
            sq.edge_weight(((0, 2).into(), Axis::Horizontal).into()),
            Some(&4)
        );
        assert_eq!(sq.edge_weight(((0, 2).into(), Axis::Vertical).into()), None);
        assert_eq!(
            sq.edge_weight(((3, 0).into(), Axis::Horizontal).into()),
            None
        );
        assert_eq!(
            sq.edge_weight(((3, 0).into(), Axis::Vertical).into()),
            Some(&-3)
        );
    }

    #[test]
    fn node_identifiers() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            3,
            5,
            |x, y| x + 2 * y,
            |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
        );
        for (i, x) in sq.node_identifiers().enumerate() {
            let x = x;
            let x2 = sq.to_index(x);
            assert_eq!(x2, i);
            let x3 = sq.from_index(x2);
            assert_eq!(x, x3);
        }
    }

    #[test]
    fn edge_references() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            3,
            5,
            |x, y| x + 2 * y,
            |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
        );

        let mut i = 0;
        let mut x = -1;
        for e in sq
            .edge_references()
            .filter(|x| x.id().1 == Axis::Horizontal)
        {
            let y = sq.to_index(e.edge_id.0) as i32;
            assert!(x < y);
            x = y;
            i += 1;
        }
        assert_eq!(i, 10);
        x = -1;
        i = 0;
        for e in sq.edge_references().filter(|x| x.id().1 == Axis::Vertical) {
            let y = sq.to_index(e.edge_id.0) as i32;
            assert!(x < y);
            x = y;
            i += 1;
        }
        assert_eq!(i, 12);
    }

    #[test]
    fn astar() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            3,
            4,
            |_, _| (),
            |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { 3 }),
        );

        let x = petgraph::algo::astar(
            &sq,
            (0, 0).into(),
            |x| x == (2, 1),
            |e| *e.weight(),
            |x| x.distance((2, 1)) as i32,
        );
        assert!(x.is_some());
        let (d, p) = x.unwrap();
        assert_eq!(d, 5);
        assert_eq!(p, [(0, 0), (0, 1), (1, 1), (2, 1)]);

        let x = petgraph::algo::astar(
            &sq,
            (2, 1).into(),
            |x| x == (0, 0),
            |e| *e.weight(),
            |x| x.distance((0, 0)) as i32,
        );
        assert!(x.is_some());
        let (d, p) = x.unwrap();
        assert_eq!(d, 5);
        assert_eq!(p, [(2, 1), (1, 1), (0, 1), (0, 0)])
    }
}
