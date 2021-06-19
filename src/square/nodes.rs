use petgraph::{
    graph::IndexType,
    visit::{NodeCompactIndexable, NodeCount},
};

use super::*;

impl<'a, N, E, Ix, S> IntoNodeIdentifiers for &'a SquareGraph<N, E, Ix, S>
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
    pub(crate) h_max: usize,
    h: usize,
    pub(crate) v_max: usize,
    v: usize,
    pd: PhantomData<Ix>,
}

impl<Ix> NodeIndices<Ix> {
    pub(crate) fn new(h: usize, v: usize) -> Self {
        Self {
            h_max: h,
            h: 0,
            v_max: v,
            v: 0,
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
        let nv: usize;
        if self.v < self.v_max {
            nv = self.v;
            self.v += 1;
        } else {
            if self.h + 1 < self.h_max {
                nv = 0;
                self.v = 1;
                self.h += 1;
            } else {
                return None;
            }
        }
        Some(NodeIndex::new(Ix::new(self.h), Ix::new(nv)))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.v_max * (self.h_max - self.h - 1) + self.v;
        (len, Some(len))
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

impl<N, E, Ix, S> NodeCompactIndexable for SquareGraph<N, E, Ix, S> where Ix: IndexType {}

impl<N, E, Ix, S> NodeCount for SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
{
    fn node_count(self: &Self) -> usize {
        self.nodes.size()
    }
}

impl<N, E, Ix, S> NodeIndexable for SquareGraph<N, E, Ix, S>
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
