use super::*;
use crate::SquareGraph;
use petgraph::{graph::IndexType, visit::IntoNeighbors};
use std::iter::FusedIterator;

pub struct Neighbors<Ix: IndexType> {
    node: NodeIndex<Ix>,
    state: usize,
    h: usize,
    v: usize,
}

impl<Ix> Iterator for Neighbors<Ix>
where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let n = self.node;
            let x = match self.state {
                0 if n.horizontal.index() != 0 => n.left(),
                1 if n.horizontal.index() + 1 < self.h => n.right(),
                2 if n.vertical.index() != 0 => n.down(),
                3 if n.vertical.index() + 1 < self.v => n.up(),
                4..=usize::MAX => return None,
                _ => {
                    self.state += 1;
                    continue;
                }
            };
            self.state += 1;
            return Some(x);
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(4 - self.state))
    }
}

impl<Ix> FusedIterator for Neighbors<Ix> where Ix: IndexType {}

impl<'a, N, E, Ix> IntoNeighbors for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
{
    type Neighbors = Neighbors<Ix>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            node: a,
            state: 0,
            h: self.horizontal_node_count(),
            v: self.vertical_node_count(),
        }
    }
}
