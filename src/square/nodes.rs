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
    p: itertools::Product<Range<usize>, Range<usize>>,
    pd: PhantomData<Ix>,
}

impl<Ix> NodeIndices<Ix> {
    pub(crate) fn new(h: usize, v: usize) -> Self {
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
