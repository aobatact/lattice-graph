use core::panic;
use itertools::*;
use std::{
    any::TypeId,
    collections::btree_map::Range,
    convert::{TryFrom, TryInto},
    iter::{Flatten, FusedIterator},
    marker::PhantomData,
    ops::{Add, RangeInclusive},
};

use petgraph::{
    data::{DataMap, DataMapMut},
    graph::{EdgeIndex, IndexType, NodeIndex},
    visit::{
        Data, EdgeRef, GetAdjacencyMatrix, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges,
        IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCount, NodeIndexable,
    },
    Undirected,
};

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

impl<N, E, Ix> SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    pub fn vertical_node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn horizontal_node_count(&self) -> usize {
        self.nodes[0].len()
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
    RangeInclusive<Ix>: Iterator<Item = Ix>,
{
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences::new(&self)
    }
}
/*
*/
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
    RangeInclusive<Ix>: Iterator<Item = Ix>,
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


impl<'a, N, E, Ix> IntoNeighbors for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Neighbors;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        todo!()
    }
}


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
    RangeInclusive<Ix>: Iterator<Item = Ix>,
{
    type NodeIdentifiers = NodeIndices<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices::new(self.horizontal_node_count(), self.vertical_node_count())
    }
}

#[derive(Clone, Debug)]
pub struct NodeIndices<Ix> {
    p: itertools::Product<RangeInclusive<usize>, RangeInclusive<usize>>,
    pd: PhantomData<Ix>,
}

impl<Ix> NodeIndices<Ix> {
    pub fn new(v: usize, h: usize) -> Self {
        Self {
            p: (0..=v).cartesian_product(0..=h),
            pd: PhantomData,
        }
    }
}

impl<Ix: IndexType> Iterator for NodeIndices<Ix>
where
    RangeInclusive<Ix>: Iterator<Item = Ix>,
{
    type Item = (VerticalIndex<Ix>, HorizontalIndex<Ix>);

    fn next(&mut self) -> Option<Self::Item> {
        self.p
            .next()
            .map(|x| (VerticalIndex::new(x.0), HorizontalIndex::new(x.1)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.p.size_hint()
    }
}

impl<Ix: IndexType> FusedIterator for NodeIndices<Ix> where RangeInclusive<Ix>: Iterator<Item = Ix> {}

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
        a.0.index() * self.horizontal_node_count() + self.vertical_node_count()
    }

    fn from_index(self: &Self, i: usize) -> Self::NodeId {
        let h = self.horizontal_node_count();
        (NodeIndex::new(i / h), NodeIndex::new(i % h))
    }
}
