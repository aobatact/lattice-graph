use std::iter::FusedIterator;

use petgraph::visit::IntoNeighbors;

use super::*;

#[derive(Debug)]
pub struct Neighbors<'a, N, E, S, C> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    state: usize,
}

impl<'a, N, E, S, C> Neighbors<'a, N, E, S, C> {
    pub(crate) fn new(graph: &'a LatticeGraph<N, E, S>, node: C) -> Self {
        Self {
            graph,
            node,
            state: 0,
        }
    }
}

impl<'a, N, E, S, C, D> Neighbors<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    #[allow(dead_code)]
    fn next_cd(&mut self) -> Option<(C, D)> {
        while self.state < S::Axis::DIRECTED_COUNT {
            unsafe {
                let d = D::from_index_unchecked(self.state);
                let n = self.graph.s.move_coord(self.node, d.clone());
                self.state += 1;
                if let Ok(target) = n {
                    return Some((target, d));
                }
            }
        }
        None
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
        // self.next_cd().map(|(c, _)| c)
        while self.state < S::Axis::DIRECTED_COUNT {
            unsafe {
                let d = D::from_index_unchecked(self.state);
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
        let x = S::Axis::DIRECTED_COUNT - self.state;
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
