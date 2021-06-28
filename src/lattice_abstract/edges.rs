use petgraph::visit::EdgeRef;

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct EdgeReference<'a, C, E, D> {
    pub(crate) source_id: C,
    pub(crate) target_id: C,
    pub(crate) edge_weight: &'a E,
    pub(crate) direction: D,
}

impl<'a, C: Clone, E, D: Clone> Clone for EdgeReference<'a, C, E, D> {
    fn clone(&self) -> Self {
        Self {
            source_id: self.source_id.clone(),
            target_id: self.target_id.clone(),
            edge_weight: self.edge_weight,
            direction: self.direction.clone(),
        }
    }
}

impl<'a, C: Copy, E, D: Copy> Copy for EdgeReference<'a, C, E, D> {}

impl<'a, C, E, D> EdgeRef for EdgeReference<'a, C, E, D>
where
    C: Copy,
    D: AxisDirection + Copy,
{
    type NodeId = C;

    type EdgeId = (C, D);

    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.source_id
    }

    fn target(&self) -> Self::NodeId {
        self.target_id
    }

    fn weight(&self) -> &Self::Weight {
        self.edge_weight
    }

    fn id(&self) -> Self::EdgeId {
        (self.source_id, self.direction)
    }
}

pub struct Edges<'a, N, E, S, C> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    offset: Offset,
    state: usize,
}

impl<'a, N, E, S, C, D> Iterator for Edges<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type Item = EdgeReference<'a, C, E, D>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.state < S::Axis::DIRECTED_COUNT {
            unsafe {
                let d = D::from_index_unchecked(self.state);
                let n = self.graph.s.move_coord(self.node, d.clone());
                self.state += 1;
                if let Ok(target) = n {
                    let nx = if d.clone().is_forward() {
                        self.offset
                    } else {
                        self.graph.s.to_offset_unchecked(target)
                    };
                    let ea = S::Axis::from_direction(d.clone()).to_index();
                    let e = &self
                        .graph
                        .edges
                        .get_unchecked(ea)
                        .ref_2d()
                        .get_unchecked(nx.0)
                        .get_unchecked(nx.1);
                    return Some(EdgeReference {
                        source_id: self.node,
                        target_id: target,
                        edge_weight: e,
                        direction: d,
                    });
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
