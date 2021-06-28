use petgraph::visit::{IntoNodeIdentifiers, IntoNodeReferences};

use super::*;

pub struct NodeIndices<S> {
    index: usize,
    s: S,
}

impl<S: shapes::Shape> Iterator for NodeIndices<S> {
    type Item = <S as Shape>::Coordinate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.s.node_counts() {
            let x = self.s.from_index(self.index);
            self.index += 1;
            Some(x)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.s.node_counts() - self.index;
        (len, Some(len))
    }
}

impl<'a, N, E, S: Shape + Clone> IntoNodeIdentifiers for &'a LatticeGraph<N, E, S> {
    type NodeIdentifiers = NodeIndices<S>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIndices {
            index: 0,
            s: self.s.clone(),
        }
    }
}

pub struct NodeReferences<'a, N, E, S> {
    graph: &'a LatticeGraph<N, E, S>,
    index: usize,
}

impl<'a, N, E, S: Shape> Iterator for NodeReferences<'a, N, E, S> {
    type Item = (<S as Shape>::Coordinate, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.graph.s.node_counts() {
            let x = self.graph.s.from_index(self.index);
            self.index += 1;
            Some((x, self.graph.node_weight(x).unwrap()))
        } else {
            None
        }
    }
}

impl<'a, N, E, S: Shape + Clone> IntoNodeReferences for &'a LatticeGraph<N, E, S> {
    type NodeRef = (<S as Shape>::Coordinate, &'a N);

    type NodeReferences = NodeReferences<'a, N, E, S>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences {
            graph: self,
            index: 0,
        }
    }
}
