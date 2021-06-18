use super::*;
use crate::SquareGraph;
use petgraph::{graph::IndexType, visit::IntoNeighbors};
use std::iter::FusedIterator;

pub struct Neighbors<Ix: IndexType, S> {
    node: NodeIndex<Ix>,
    state: usize,
    h: usize,
    v: usize,
    s: PhantomData<S>,
}

impl<Ix, S> Iterator for Neighbors<Ix, S>
where
    Ix: IndexType,
    S: IsLoop,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let n = self.node;
            let x = match self.state {
                0 if n.horizontal.index() != 0 => n.left(),
                0 if <S as IsLoop>::HORIZONTAL && n.horizontal.index() == 0 => NodeIndex {
                    horizontal: Ix::new(self.h - 1),
                    vertical: n.vertical,
                },
                1 if n.horizontal.index() + 1 < self.h => n.right(),
                1 if <S as IsLoop>::HORIZONTAL && n.horizontal.index() + 1 == self.h => NodeIndex {
                    horizontal: Ix::new(0),
                    vertical: n.vertical,
                },
                2 if n.vertical.index() != 0 => n.down(),
                2 if <S as IsLoop>::VERTICAL && n.vertical.index() == 0 => NodeIndex {
                    horizontal: n.vertical,
                    vertical: Ix::new(self.v - 1),
                },
                3 if n.vertical.index() + 1 < self.v => n.up(),
                3 if <S as IsLoop>::HORIZONTAL && n.horizontal.index() + 1 == self.h => NodeIndex {
                    horizontal: n.vertical,
                    vertical: Ix::new(0),
                },
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

impl<Ix, S> FusedIterator for Neighbors<Ix, S>
where
    Ix: IndexType,
    S: IsLoop,
{
}

impl<'a, N, E, Ix, S> IntoNeighbors for &'a SquareGraph<N, E, Ix, S>
where
    Ix: IndexType,
    S: IsLoop,
{
    type Neighbors = Neighbors<Ix, S>;

    fn neighbors(self: Self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            node: a,
            state: 0,
            h: self.horizontal_node_count(),
            v: self.vertical_node_count(),
            s: PhantomData,
        }
    }
}
