use std::iter::FusedIterator;

use petgraph::visit::{GetAdjacencyMatrix, IntoNeighbors, IntoNeighborsDirected};

use super::*;

/// Neighbors of the node. See [`neighbors`](`IntoNeighbors::neighbors`).
#[derive(Debug)]
pub struct Neighbors<'a, N, E, S: Shape, C> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    state: usize,
}

impl<'a, N, E, S: Shape, C> Neighbors<'a, N, E, S, C> {
    pub(crate) fn new(graph: &'a LatticeGraph<N, E, S>, node: C) -> Self {
        Self {
            graph,
            node,
            state: 0,
        }
    }
}

impl<'a, N, E, S, C, D> Iterator for Neighbors<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type Item = C;

    fn next(&mut self) -> Option<Self::Item> {
        while self.state < S::Axis::UNDIRECTED_COUNT {
            unsafe {
                let d = D::dir_from_index_unchecked(self.state);
                let n = self.graph.s.move_coord(self.node, d.clone());
                self.state += 1;
                if let Ok(target) = n {
                    return Some(target);
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let x = S::Axis::UNDIRECTED_COUNT - self.state;
        (0, Some(x))
    }
}

impl<'a, N, E, S, C, D> FusedIterator for Neighbors<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
}

impl<'a, N, E, S, C, D> IntoNeighbors for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type Neighbors = Neighbors<'a, N, E, S, C>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors::new(self, a)
    }
}

impl<'a, N, E, S, C, D> IntoNeighborsDirected for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type NeighborsDirected = Neighbors<'a, N, E, S, C>;

    fn neighbors_directed(self: Self, a: Self::NodeId, _d: petgraph::Direction) -> Self::Neighbors {
        Neighbors::new(self, a)
    }
}

impl<N, E, S, C> GetAdjacencyMatrix for LatticeGraph<N, E, S>
where
    C: Copy + PartialEq,
    S: Shape<Coordinate = C>,
{
    type AdjMatrix = ();
    fn adjacency_matrix(self: &Self) -> Self::AdjMatrix {
        ()
    }

    fn is_adjacent(
        self: &Self,
        _matrix: &Self::AdjMatrix,
        a: Self::NodeId,
        b: Self::NodeId,
    ) -> bool {
        self.s.is_neighbor(a, b)
    }
}
