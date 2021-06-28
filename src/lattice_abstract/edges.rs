use petgraph::visit::EdgeRef;

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct EdgeReference<'a, C, E, S, D> {
    pub(crate) node_id: C,
    pub(crate) edge_weight: &'a E,
    pub(crate) direction: D,
    pub(crate) shape: S,
}

impl<'a, C, E, S, D> EdgeReference<'a, C, E, S, D>
where
    C: Copy,
    S: Shape<Coordinate = C> + Copy,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    pub(crate) fn get_node(&self, target: bool) -> <Self as EdgeRef>::NodeId {
        if target ^ self.direction.is_forward() {
            self.node_id
        } else {
            self.shape
                .move_coord(self.node_id, self.direction)
                .unwrap_or_else(|_| unsafe { crate::unreachable_debug_checked() })
        }
    }
}

impl<'a, C: Clone, E, S: Clone, D: Clone> Clone for EdgeReference<'a, C, E, S, D> {
    fn clone(&self) -> Self {
        Self {
            node_id: self.node_id.clone(),
            edge_weight: self.edge_weight,
            direction: self.direction.clone(),
            shape: self.shape.clone(),
        }
    }
}

impl<'a, C: Copy, E, S: Copy, D: Copy> Copy for EdgeReference<'a, C, E, S, D> {}

impl<'a, C, E, S, D> EdgeRef for EdgeReference<'a, C, E, S, D>
where
    C: Copy,
    S: Shape<Coordinate = C> + Copy,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    type NodeId = C;

    type EdgeId = (C, D);

    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.get_node(false)
    }

    fn target(&self) -> Self::NodeId {
        self.get_node(true)
    }

    fn weight(&self) -> &Self::Weight {
        self.edge_weight
    }

    fn id(&self) -> Self::EdgeId {
        (self.node_id, self.direction)
    }
}
