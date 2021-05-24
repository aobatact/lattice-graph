use itertools::*;
use petgraph::{
    data::{DataMap, DataMapMut},
    graph::{IndexType, NodeIndex},
    visit::{
        Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoNeighbors,
        IntoNodeIdentifiers, NodeCompactIndexable, NodeCount, NodeIndexable,
    },
    Undirected,
};
use std::{iter::FusedIterator, marker::PhantomData, ops::Range};

pub type VerticalIndex<Ix> = NodeIndex<Ix>;
pub type HorizontalIndex<Ix> = NodeIndex<Ix>;

#[derive(Clone, Debug)]
pub struct SquareGraph<N, E, Ix = usize>
where
    Ix: IndexType,
{
    nodes: Vec</*horizontal*/ Vec<N>>,
    vertical: Vec<Vec<E>>,   //↓
    horizontal: Vec<Vec<E>>, //→
    pd: PhantomData<Ix>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Direction {
    Vertical,
    Horizontal,
}

impl Direction {
    pub fn is_vertical(&self) -> bool {
        *self == Direction::Vertical
    }
    pub fn is_horizontal(&self) -> bool {
        *self == Direction::Horizontal
    }
}

impl<N, E, Ix> SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    pub fn new_raw(nodes: Vec<Vec<N>>, vertical: Vec<Vec<E>>, horizontal: Vec<Vec<E>>) -> Self {
        let s = Self {
            nodes,
            vertical,
            horizontal,
            pd: PhantomData,
        };
        debug_assert!(s.check_gen());
        s
    }

    pub fn new(v: usize, h: usize) -> Self
    where
        N: Default,
        E: Default,
    {
        Self::new_with(v, h, |_, _| N::default(), |_, _, _| E::default())
    }

    pub fn new_with<FN, FE>(v: usize, h: usize, mut fnode: FN, mut fedge: FE) -> Self
    where
        FN: FnMut(usize, usize) -> N,
        FE: FnMut(usize, usize, Direction) -> E,
    {
        let mut nodes = Vec::with_capacity(v);
        let mut vertical = Vec::with_capacity(v - 1);
        let mut horizontal = Vec::with_capacity(v);

        for vi in 0..v - 1 {
            let mut nv = Vec::with_capacity(h);
            let mut vv = Vec::with_capacity(h);
            let mut hv = Vec::with_capacity(h - 1);
            for hi in 0..h - 1 {
                nv.push(fnode(vi, hi));
                vv.push(fedge(vi, hi, Direction::Vertical));
                hv.push(fedge(vi, hi, Direction::Horizontal));
            }
            nv.push(fnode(vi, h - 1));
            vv.push(fedge(vi, h - 1, Direction::Vertical));
            nodes.push(nv);
            vertical.push(vv);
            horizontal.push(hv);
        }
        let mut nv = Vec::with_capacity(h);
        let mut hv = Vec::with_capacity(h - 1);
        for hi in 0..h - 1 {
            nv.push(fnode(v - 1, hi));
            hv.push(fedge(v - 1, hi, Direction::Horizontal));
        }
        nv.push(fnode(v - 1, h - 1));
        nodes.push(nv);
        horizontal.push(hv);
        Self::new_raw(nodes, vertical, horizontal)
    }

    pub fn vertical_node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn horizontal_node_count(&self) -> usize {
        self.nodes.get(0).map(|x| x.len()).unwrap_or(0)
    }

    fn check_gen(&self) -> bool {
        let v = self.vertical_node_count();
        let h = self.horizontal_node_count();
        self.nodes.iter().all(|x| x.len() == h)
            && self.vertical.len() == v - 1
            && self.vertical.iter().all(|x| x.len() == h)
            && self.horizontal.len() == v
            && self.horizontal.iter().all(|x| x.len() == h - 1)
    }
}

impl<N, E, Ix> GraphBase for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeId = (VerticalIndex<Ix>, HorizontalIndex<Ix>);
    type EdgeId = (VerticalIndex<Ix>, HorizontalIndex<Ix>, Direction);
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
        self.nodes.get(id.0.index())?.get(id.1.index())
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        match id.2 {
            Direction::Vertical => &self.vertical,
            Direction::Horizontal => &self.horizontal,
        }
        .get(id.0.index())?
        .get(id.1.index())
    }
}

impl<N, E, Ix> DataMapMut for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_weight_mut(self: &mut Self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.nodes.get_mut(id.0.index())?.get_mut(id.1.index())
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        match id.2 {
            Direction::Vertical => &mut self.vertical,
            Direction::Horizontal => &mut self.horizontal,
        }
        .get_mut(id.0.index())?
        .get_mut(id.1.index())
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeReference<'a, E, Ix>((VerticalIndex<Ix>, HorizontalIndex<Ix>, Direction), &'a E);
impl<'a, E: Copy, Ix: IndexType> EdgeRef for EdgeReference<'a, E, Ix> {
    type NodeId = (VerticalIndex<Ix>, HorizontalIndex<Ix>);
    type EdgeId = (VerticalIndex<Ix>, HorizontalIndex<Ix>, Direction);
    type Weight = E;

    fn source(&self) -> Self::NodeId {
        (self.0 .0, self.0 .1)
    }

    fn target(&self) -> Self::NodeId {
        match self.0 .2 {
            Direction::Vertical => (NodeIndex::new(self.0 .0.index() + 1), self.0 .1),
            Direction::Horizontal => (self.0 .0, NodeIndex::new(self.0 .1.index() + 1)),
        }
    }

    fn weight(&self) -> &Self::Weight {
        self.1
    }

    fn id(&self) -> Self::EdgeId {
        self.0
    }
}
#[derive(Clone, Debug)]
pub struct EdgeReferences<'a, E, Ix> {
    vertical: &'a Vec<Vec<E>>,
    horizontal: &'a Vec<Vec<E>>,
    nodes: NodeIndices<Ix>,
    prv: Option<(VerticalIndex<Ix>, HorizontalIndex<Ix>, Direction)>,
}

impl<'a, E, Ix: IndexType> EdgeReferences<'a, E, Ix> {
    fn new<N>(graph: &'a SquareGraph<N, E, Ix>) -> Self {
        Self {
            vertical: &graph.vertical,
            horizontal: &graph.horizontal,
            nodes: NodeIndices::new(graph.vertical_node_count(), graph.horizontal_node_count()),
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
                if e.2 == Direction::Vertical {
                    let item = self.horizontal[e.0.index()].get(e.1.index());
                    if let Some(item) = item {
                        e.2 = Direction::Horizontal;
                        return Some(EdgeReference(*e, item));
                    }
                }
            }
            if let Some(next) = self.nodes.next() {
                let item = self
                    .vertical
                    .get(next.0.index())
                    .map(|x| x.get(next.1.index()))
                    .flatten();
                let e = (next.0, next.1, Direction::Vertical);
                self.prv = Some(e);
                if let Some(item) = item {
                    return Some(EdgeReference(e, item));
                }
            } else {
                return None;
            }
        }
        /*
        let mut e = (Default::default(), Default::default(), Direction::Vertical);
        let mut hol = false;
        if let Some(ex) = self.prv {
            if ex.2 == Direction::Vertical {
                e = ex;
                hol = true;
            }
        }
        loop {
            if hol {
                let item = self.horizontal[e.0.index()].get(e.1.index());
                if let Some(item) = item {
                    e.2 = Direction::Horizontal;
                    return Some(EdgeReference(e, item));
                }
            }

            if let Some(next) = self.nodes.next() {
                let item = self
                    .vertical
                    .get(next.0.index())
                    .map(|x| x.get(next.1.index()))
                    .flatten();
                e = (next.0, next.1, Direction::Vertical);
                self.prv = Some(e);
                if let Some(item) = item {
                    return Some(EdgeReference(e, item));
                }
            } else {
                return None;
            }
        }
        */
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.nodes.size_hint();
        (lo - self.vertical.len() - self.horizontal.len(), hi)
    }
}

/*
impl<'a, N, E, Ix> IntoEdges for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Edges;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        todo!()
    }
}


impl<'a, N, E, Ix> IntoEdgesDirected for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type EdgesDirected;

    fn edges_directed(self, a: Self::NodeId, dir: petgraph::EdgeDirection) -> Self::EdgesDirected {
        todo!()
    }
}
*/

impl<'a, N, E, Ix> IntoNeighbors for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Neighbors = std::vec::IntoIter<(VerticalIndex<Ix>, HorizontalIndex<Ix>)>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        let v = self.vertical_node_count();
        let h = self.horizontal_node_count();
        let va = a.0.index();
        let ha = a.1.index();
        let mut vec = Vec::new();
        if va != 0 {
            vec.push((NodeIndex::new(va - 1), a.1));
        }
        if va < v - 1 {
            vec.push((NodeIndex::new(va + 1), a.1));
        }
        if ha != 0 {
            vec.push((a.0, NodeIndex::new(ha - 1)));
        }
        if ha < h - 1 {
            vec.push((a.0, NodeIndex::new(ha + 1)));
        }
        vec.into_iter()
    }
}

/*
impl<'a, N, E, Ix> IntoNeighborsDirected for  &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NeighborsDirected;

    fn neighbors_directed(
        self,
        n: Self::NodeId,
        d: petgraph::EdgeDirection,
    ) -> Self::NeighborsDirected {
        todo!()
    }
}
*/

impl<'a, N, E, Ix> IntoNodeIdentifiers for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type NodeIdentifiers = NodeIndices<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices::new(self.vertical_node_count(), self.horizontal_node_count())
    }
}

#[derive(Clone, Debug)]
pub struct NodeIndices<Ix> {
    p: itertools::Product<Range<usize>, Range<usize>>,
    size: usize,
    pd: PhantomData<Ix>,
}

impl<Ix> NodeIndices<Ix> {
    fn new(v: usize, h: usize) -> Self {
        Self {
            p: (0..v).cartesian_product(0..h),
            size: v * h,
            pd: PhantomData,
        }
    }
}

impl<Ix: IndexType> Iterator for NodeIndices<Ix>
where
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = (VerticalIndex<Ix>, HorizontalIndex<Ix>);

    fn next(&mut self) -> Option<Self::Item> {
        self.p.next().map(|x| {
            self.size -= 1;
            (VerticalIndex::new(x.0), HorizontalIndex::new(x.1))
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

impl<Ix: IndexType> FusedIterator for NodeIndices<Ix> where Range<Ix>: Iterator<Item = Ix> {}
impl<Ix: IndexType> ExactSizeIterator for NodeIndices<Ix>
where
    Range<Ix>: Iterator<Item = Ix>,
{
    fn len(&self) -> usize {
        self.size
    }
}

/*
impl<'a, N: Clone, E, Ix> IntoNodeReferences for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type NodeRef;

    type NodeReferences;

    fn node_references(self) -> Self::NodeReferences {
        todo!()
    }
}
*/

impl<N, E, Ix> NodeCompactIndexable for SquareGraph<N, E, Ix> where Ix: IndexType {}

impl<N, E, Ix> NodeCount for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_count(self: &Self) -> usize {
        self.vertical_node_count() * self.horizontal_node_count()
    }
}

impl<N, E, Ix> NodeIndexable for SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    fn node_bound(self: &Self) -> usize {
        self.horizontal_node_count() * self.vertical_node_count()
    }

    fn to_index(self: &Self, a: Self::NodeId) -> usize {
        a.0.index() * self.horizontal_node_count() + a.1.index()
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        let h = self.horizontal_node_count();
        (NodeIndex::new(i / h), NodeIndex::new(i % h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type NI = NodeIndex;

    #[test]
    fn gen() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            4,
            3,
            |x, y| x + 2 * y,
            |x, y, d| (x + 2 * y) as i32 * (if d.is_vertical() { 1 } else { -1 }),
        );
        assert_eq!(sq.node_weight((NI::new(0), NI::new(0))), Some(&0));
        assert_eq!(sq.node_weight((NI::new(3), NI::new(0))), Some(&3));
        assert_eq!(sq.node_weight((NI::new(4), NI::new(0))), None);
        assert_eq!(sq.node_weight((NI::new(0), NI::new(2))), Some(&4));
        assert_eq!(sq.node_weight((NI::new(0), NI::new(3))), None);
        assert_eq!(
            sq.edge_weight((NI::new(0), NI::new(2), Direction::Vertical)),
            Some(&4)
        );
        assert_eq!(
            sq.edge_weight((NI::new(0), NI::new(2), Direction::Horizontal)),
            None
        );
        assert_eq!(
            sq.edge_weight((NI::new(3), NI::new(0), Direction::Vertical)),
            None
        );
        assert_eq!(
            sq.edge_weight((NI::new(3), NI::new(0), Direction::Horizontal)),
            Some(&-3)
        );
    }

    #[test]
    fn node_indices() {
        let sq = SquareGraph::<_, _, u32>::new_with(
            5,
            3,
            |x, y| x + 2 * y,
            |x, y, d| (x + 2 * y) as i32 * (if d.is_vertical() { 1 } else { -1 }),
        );
        for (i, x) in sq.node_identifiers().enumerate() {
            let x = x;
            let x2 = sq.to_index(x);
            assert_eq!(x2, i);
            let x3 = sq.from_index(x2);
            assert_eq!(x, x3);
        }
    }
}
