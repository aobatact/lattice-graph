use std::iter::FusedIterator;

use petgraph::visit::{
    IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable, NodeIndexable,
};

use super::*;

/// Iterate all index of [`LatticeGraph`]. See [`IntoNodeIdentifiers`].
#[derive(Clone, Debug)]
pub struct NodeIndices<S> {
    index: usize,
    s: S,
}

impl<S: shapes::Shape> Iterator for NodeIndices<S> {
    type Item = <S as Shape>::Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.s.node_count() {
            let x = self.s.index_to_coordinate(self.index);
            self.index += 1;
            Some(x)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.s.node_count() - self.index;
        (len, Some(len))
    }
}

impl<S: shapes::Shape> FusedIterator for NodeIndices<S> {}

impl<S: shapes::Shape> ExactSizeIterator for NodeIndices<S> {}

impl<N, E, S: Shape> IntoNodeIdentifiers for &LatticeGraph<N, E, S> {
    type NodeIdentifiers = NodeIndices<S>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices {
            index: 0,
            s: self.s.clone(),
        }
    }
}

/// Iterate all nodes of [`LatticeGraph`]. See [`IntoNodeReferences`].
pub struct NodeReferences<'a, N, E, S: Shape> {
    graph: &'a LatticeGraph<N, E, S>,
    index: usize,
}

impl<'a, N, E, S: Shape> Iterator for NodeReferences<'a, N, E, S> {
    type Item = (<S as Shape>::Coordinate, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.graph.s.node_count() {
            let x = self.graph.s.index_to_coordinate(self.index);
            self.index += 1;
            Some((x, unsafe { self.graph.node_weight_unchecked(x) }))
        } else {
            None
        }
    }
}

impl<'a, N, E, S: Shape> FusedIterator for NodeReferences<'a, N, E, S> {}

impl<'a, N, E, S: Shape> ExactSizeIterator for NodeReferences<'a, N, E, S> {}

impl<'a, N, E, S: Shape> IntoNodeReferences for &'a LatticeGraph<N, E, S> {
    type NodeRef = (<S as Shape>::Coordinate, &'a N);

    type NodeReferences = NodeReferences<'a, N, E, S>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences {
            graph: self,
            index: 0,
        }
    }
}

impl<N, E, S: Shape> NodeCount for LatticeGraph<N, E, S> {
    #[inline]
    fn node_count(&self) -> usize {
        self.s.node_count()
    }
}

impl<N, E, S: Shape> NodeIndexable for LatticeGraph<N, E, S> {
    #[inline]
    fn node_bound(&self) -> usize {
        self.s.node_count()
    }

    #[inline]
    fn to_index(&self, a: Self::NodeId) -> usize {
        self.s.to_index(a).unwrap()
    }

    #[inline]
    fn from_index(&self, i: usize) -> Self::NodeId {
        self.s.index_to_coordinate(i)
    }
}

impl<N, E, S: Shape> NodeCompactIndexable for LatticeGraph<N, E, S> {}
