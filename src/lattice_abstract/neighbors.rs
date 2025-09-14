use std::iter::FusedIterator;

use petgraph::visit::{GetAdjacencyMatrix, IntoNeighbors, IntoNeighborsDirected};

use super::*;

/// Neighbors of the node. See [`IntoNeighbors`].
#[derive(Debug)]
pub struct Neighbors<'a, N, E, S: Shape, C = <S as Shape>::Coordinate> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    current_direction: Option<<<S as Shape>::Axis as Axis>::Direction>,
}

impl<'a, N, E, S: Shape, C> Neighbors<'a, N, E, S, C> {
    pub(crate) fn new(graph: &'a LatticeGraph<N, E, S>, node: C) -> Self
    where
        <<S as Shape>::Axis as Axis>::Direction: AxisDirection,
    {
        Self {
            graph,
            node,
            current_direction: unsafe { Some(<<S as Shape>::Axis as Axis>::Direction::dir_from_index_unchecked(0)) },
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

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current_dir) = &self.current_direction {
            let d = current_dir.clone();

            // Move to next direction for next iteration
            self.current_direction = d.next_direction();

            let n = self.graph.s.move_coord(self.node, d);
            if let Ok(target) = n {
                return Some(target);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if let Some(ref current_dir) = self.current_direction {
            let current_index = current_dir.dir_to_index();
            S::Axis::UNDIRECTED_COUNT - current_index
        } else {
            0
        };
        (0, Some(remaining))
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

impl<'a, N, E, S, D> IntoNeighbors for &'a LatticeGraph<N, E, S>
where
    S: Shape,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type Neighbors = Neighbors<'a, N, E, S>;

    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors::new(self, a)
    }
}

impl<'a, N, E, S, D> IntoNeighborsDirected for &'a LatticeGraph<N, E, S>
where
    S: Shape,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type NeighborsDirected = Neighbors<'a, N, E, S>;

    fn neighbors_directed(self, a: Self::NodeId, _d: petgraph::Direction) -> Self::Neighbors {
        Neighbors::new(self, a)
    }
}

impl<N, E, S, C> GetAdjacencyMatrix for LatticeGraph<N, E, S>
where
    C: Copy + PartialEq,
    S: Shape<Coordinate = C>,
{
    type AdjMatrix = ();
    fn adjacency_matrix(&self) -> Self::AdjMatrix {}

    fn is_adjacent(&self, _matrix: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool {
        self.s.is_neighbor(a, b)
    }
}
