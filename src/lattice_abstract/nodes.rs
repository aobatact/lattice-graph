use std::iter::FusedIterator;

use petgraph::visit::{
    IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable, NodeIndexable,
};

use super::*;

/// Iterate all index of [`LatticeGraph`]. See [`IntoNodeIdentifiers`].
#[derive(Clone, Debug)]
pub struct NodeIndices<S> {
    current_offset: shapes::Offset,
    s: S,
}

impl<S: shapes::Shape> Iterator for NodeIndices<S> {
    type Item = <S as Shape>::Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset.horizontal >= self.s.horizontal() {
            return None;
        }
        
        let coord = self.s.offset_to_coordinate(self.current_offset);
        
        // Move to next position (row-major order for cache efficiency)
        self.current_offset.vertical += 1;
        if self.current_offset.vertical >= self.s.vertical() {
            self.current_offset.vertical = 0;
            self.current_offset.horizontal += 1;
        }
        
        Some(coord)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current_offset.horizontal < self.s.horizontal() {
            let remaining_in_current_row = self.s.vertical() - self.current_offset.vertical;
            let remaining_rows = self.s.horizontal() - self.current_offset.horizontal - 1;
            remaining_in_current_row + remaining_rows * self.s.vertical()
        } else {
            0
        };
        (remaining, Some(remaining))
    }
}

impl<S: shapes::Shape> FusedIterator for NodeIndices<S> {}

impl<S: shapes::Shape> ExactSizeIterator for NodeIndices<S> {}

impl<N, E, S: Shape> IntoNodeIdentifiers for &LatticeGraph<N, E, S> {
    type NodeIdentifiers = NodeIndices<S>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices {
            current_offset: shapes::Offset::new(0, 0),
            s: self.s.clone(),
        }
    }
}

/// Iterate all nodes of [`LatticeGraph`]. See [`IntoNodeReferences`].
pub struct NodeReferences<'a, N, E, S: Shape> {
    graph: &'a LatticeGraph<N, E, S>,
    current_offset: shapes::Offset,
}

impl<'a, N, E, S: Shape> Iterator for NodeReferences<'a, N, E, S> {
    type Item = (<S as Shape>::Coordinate, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset.horizontal >= self.graph.s.horizontal() {
            return None;
        }
        
        let coord = self.graph.s.offset_to_coordinate(self.current_offset);
        let node_ref = unsafe { self.graph.node_weight_unchecked_raw(self.current_offset) };
        
        // Move to next position (row-major order for cache efficiency)
        self.current_offset.vertical += 1;
        if self.current_offset.vertical >= self.graph.s.vertical() {
            self.current_offset.vertical = 0;
            self.current_offset.horizontal += 1;
        }
        
        Some((coord, node_ref))
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
            current_offset: shapes::Offset::new(0, 0),
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
