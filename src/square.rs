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
use smallvec::*;
use std::{iter::FusedIterator, marker::PhantomData, ops::Range, usize};

const BORDER: usize = 64;

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
    nodes: SmallVec<[SmallVec<[N; BORDER]>; BORDER]>,
    horizontal: SmallVec<[SmallVec<[E; BORDER]>; BORDER]>, //↓
    vertical: SmallVec<[SmallVec<[E; BORDER]>; BORDER]>,   //→
    pd: PhantomData<Ix>,
}

impl<N, E, Ix> SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    /// Create a `SquareGraph` from raw data.
    /// It only check whether the size of nodes and edges are correct in `debug_assertion`.
    pub unsafe fn new_raw(
        nodes: SmallVec<[SmallVec<[N; BORDER]>; BORDER]>,
        horizontal: SmallVec<[SmallVec<[E; BORDER]>; BORDER]>,
        vertical: SmallVec<[SmallVec<[E; BORDER]>; BORDER]>,
    ) -> Self {
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
        let mut nodes = SmallVec::with_capacity(h);
        let mut horizontal = SmallVec::with_capacity(h - 1);
        let mut vertical = SmallVec::with_capacity(h);

        for vi in 0..h - 1 {
            let mut nv = SmallVec::with_capacity(v);
            let mut vv = SmallVec::with_capacity(v);
            let mut hv = SmallVec::with_capacity(v - 1);
            for hi in 0..v - 1 {
                nv.push(fnode(vi, hi));
                vv.push(fedge(vi, hi, Axis::Horizontal));
                hv.push(fedge(vi, hi, Axis::Vertical));
            }
            nv.push(fnode(vi, v - 1));
            vv.push(fedge(vi, v - 1, Axis::Horizontal));
            nodes.push(nv);
            horizontal.push(vv);
            vertical.push(hv);
        }
        let mut nv = SmallVec::with_capacity(h);
        let mut hv = SmallVec::with_capacity(h - 1);
        for hi in 0..v - 1 {
            nv.push(fnode(h - 1, hi));
            hv.push(fedge(h - 1, hi, Axis::Vertical));
        }
        nv.push(fnode(v - 1, h - 1));
        nodes.push(nv);
        vertical.push(hv);
        unsafe { Self::new_raw(nodes, horizontal, vertical) }
    }

    /// Returns the Node count in the horizontal direction.
    pub fn horizontal_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the Node count in the vertical direction.
    pub fn vertical_node_count(&self) -> usize {
        self.nodes.get(0).map(|x| x.len()).unwrap_or(0)
    }

    /// Check the size of nodes and edges.
    fn check_gen(&self) -> bool {
        let v = self.horizontal_node_count();
        let h = self.vertical_node_count();
        self.nodes.iter().all(|x| x.len() == h)
            && self.horizontal.len() == v - 1
            && self.horizontal.iter().all(|x| x.len() == h)
            && self.vertical.len() == v
            && self.vertical.iter().all(|x| x.len() == h - 1)
    }

    /// Get a reference to the nodes. `[horizontal][vertical]`
    pub fn nodes(&self) -> &[SmallVec<[N; BORDER]>] {
        &self.nodes
    }

    /// Get a reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal(&self) -> &[SmallVec<[E; BORDER]>] {
        &self.horizontal
    }

    /// Get a reference to the vertical edges. `[horizontal][vertical]`
    pub fn vertical(&self) -> &[SmallVec<[E; BORDER]>] {
        &self.vertical
    }

    /// Get a mutable reference to the nodes. `[horizontal][vertical]`
    pub fn nodes_mut(&mut self) -> &mut [SmallVec<[N; BORDER]>] {
        &mut self.nodes
    }

    /// Get a mutable reference to the horizontal edges. `[horizontal][vertical]`
    pub fn horizontal_mut(&mut self) -> &mut [SmallVec<[E; BORDER]>] {
        &mut self.horizontal
    }

    /// Get a mutable reference to the vertical edges.
    pub fn vertical_mut(&mut self) -> &mut [SmallVec<[E; BORDER]>] {
        &mut self.vertical
    }
}

impl<E, Ix> SquareGraph<(), E, Ix>
where
    Ix: IndexType,
{
    /// Create a `SquareGraph` with the edges initialized from position.
    pub fn new_edge_graph<FE>(v: usize, h: usize, fedge: FE) -> Self
    where
        FE: FnMut(usize, usize, Axis) -> E,
    {
        Self::new_with(v, h, |_, _| (), fedge)
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
            .get(id.horizontal.index())?
            .get(id.vertical.index())
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        match id.1 {
            Axis::Horizontal => &self.horizontal,
            Axis::Vertical => &self.vertical,
        }
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
            .get_mut(id.horizontal.index())?
            .get_mut(id.vertical.index())
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        match id.1 {
            Axis::Horizontal => &mut self.horizontal,
            Axis::Vertical => &mut self.vertical,
        }
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeReference<'a, E, Ix: IndexType> {
    edge_id: EdgeIndex<Ix>,
    edge_weight: &'a E,
    direction: bool,
}

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
    horizontal: &'a SmallVec<[SmallVec<[E; BORDER]>; BORDER]>,
    vertical: &'a SmallVec<[SmallVec<[E; BORDER]>; BORDER]>,
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
                    let item = self.vertical[e.0.horizontal.index()].get(e.0.vertical.index());
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
        (lo - self.horizontal.len() - self.vertical.len(), hi)
    }
}

impl<'a, N, E, Ix> IntoEdges for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    E: Copy,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Edges = smallvec::IntoIter<[EdgeReference<'a, E, Ix>; 4]>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        let v = self.horizontal_node_count();
        let h = self.vertical_node_count();
        let va = a.horizontal.index();
        let ha = a.vertical.index();
        let mut vec = SmallVec::with_capacity(4);
        if va != 0 {
            vec.push(EdgeReference {
                edge_id: EdgeIndex(
                    NodeIndex::new(Ix::new(va - 1), a.vertical),
                    Axis::Horizontal,
                ),
                edge_weight: &self.horizontal[va - 1][ha],
                direction: false,
            });
        }
        if va < v - 1 {
            vec.push(EdgeReference {
                edge_id: EdgeIndex(a, Axis::Horizontal),
                edge_weight: &self.horizontal[va][ha],
                direction: true,
            });
        }
        if ha != 0 {
            vec.push(EdgeReference {
                edge_id: EdgeIndex(
                    NodeIndex::new(a.horizontal, Ix::new(ha - 1)),
                    Axis::Vertical,
                ),
                edge_weight: &self.vertical[va][ha - 1],
                direction: false,
            });
        }
        if ha < h - 1 {
            vec.push(EdgeReference {
                edge_id: EdgeIndex(a, Axis::Vertical),
                edge_weight: &self.vertical[va][ha],
                direction: true,
            });
        }
        vec.into_iter()
    }
}

impl<'a, N, E, Ix> IntoNeighbors for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Neighbors = std::vec::IntoIter<NodeIndex<Ix>>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        let v = self.horizontal_node_count();
        let h = self.vertical_node_count();
        let va = a.horizontal.index();
        let ha = a.vertical.index();
        let mut vec = Vec::new();
        if va != 0 {
            vec.push(NodeIndex::new(Ix::new(va - 1), a.vertical));
        }
        if va < v - 1 {
            vec.push(NodeIndex::new(Ix::new(va + 1), a.vertical));
        }
        if ha != 0 {
            vec.push(NodeIndex::new(a.horizontal, Ix::new(ha - 1)));
        }
        if ha < h - 1 {
            vec.push(NodeIndex::new(a.horizontal, Ix::new(ha + 1)));
        }
        vec.into_iter()
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
    fn new(v: usize, h: usize) -> Self {
        Self {
            p: (0..v).cartesian_product(0..h),
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
            nodes: &self.nodes,
        }
    }
}

/// Iterate all nodes of [`SquareGraph`].
pub struct NodeReferences<'a, N, Ix> {
    indices: NodeIndices<Ix>,
    nodes: &'a SmallVec<[SmallVec<[N; BORDER]>; BORDER]>,
}

impl<'a, N, Ix> Iterator for NodeReferences<'a, N, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = (NodeIndex<Ix>, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|x| {
            (x, unsafe {
                self.nodes
                    .get_unchecked(x.horizontal.index())
                    .get_unchecked(x.vertical.index())
            })
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}

impl<N, E, Ix> NodeCompactIndexable for SquareGraph<N, E, Ix> where Ix: IndexType {}

impl<N, E, Ix> NodeCount for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_count(self: &Self) -> usize {
        self.horizontal_node_count() * self.vertical_node_count()
    }
}

impl<N, E, Ix> NodeIndexable for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_bound(self: &Self) -> usize {
        self.vertical_node_count() * self.horizontal_node_count()
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
