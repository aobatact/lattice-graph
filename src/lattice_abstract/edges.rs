use std::iter::FusedIterator;

use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoEdges};

use super::*;

#[derive(Debug, PartialEq, Eq)]
pub struct EdgeReference<'a, C, E, D, A> {
    pub(crate) source_id: C,
    pub(crate) target_id: C,
    pub(crate) edge_weight: &'a E,
    pub(crate) direction: D,
    pub(crate) axis: PhantomData<fn() -> A>,
}

impl<'a, C: Clone, E, D: Clone, A> Clone for EdgeReference<'a, C, E, D, A> {
    fn clone(&self) -> Self {
        Self {
            source_id: self.source_id.clone(),
            target_id: self.target_id.clone(),
            edge_weight: self.edge_weight,
            direction: self.direction.clone(),
            axis: PhantomData,
        }
    }
}

impl<'a, C: Copy, E, D: Copy, A> Copy for EdgeReference<'a, C, E, D, A> {}

impl<'a, C, E, D, A> EdgeRef for EdgeReference<'a, C, E, D, A>
where
    C: Copy,
    D: AxisDirection + Copy,
    A: Axis<Direction = D>,
{
    type NodeId = C;

    type EdgeId = (C, A);

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
        (self.source_id, A::from_direction(self.direction))
    }
}

#[derive(Debug)]
pub struct Edges<'a, N, E, S, C> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    offset: Offset,
    state: usize,
}

impl<'a, N, E, S, C, D, A> Iterator for Edges<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    type Item = EdgeReference<'a, C, E, D, A>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.state < S::Axis::DIRECTED_COUNT {
            unsafe {
                let d = D::from_index_unchecked(self.state);
                let n = self.graph.s.move_coord(self.node, d.clone());
                self.state += 1;
                if let Ok(target) = n {
                    let (nx, ne) = if d.clone().is_forward() {
                        (self.offset, self.state)
                    } else {
                        (
                            self.graph.s.to_offset_unchecked(target),
                            self.state - S::Axis::COUNT,
                        )
                    };
                    debug_assert_eq!(S::Axis::from_direction(d.clone()).to_index(), ne);
                    let e = &self
                        .graph
                        .edges
                        .get_unchecked(ne)
                        .ref_2d()
                        .get_unchecked(nx.0)
                        .get_unchecked(nx.1);
                    return Some(EdgeReference {
                        source_id: self.node,
                        target_id: target,
                        edge_weight: e,
                        direction: d,
                        axis: PhantomData,
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

impl<'a, N, E, S, C, D> FusedIterator for Edges<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C>,
    S::Axis: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
}

impl<'a, N, E, S, C, D, A> IntoEdges for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    type Edges = Edges<'a, N, E, S, C>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        let offset = self.s.to_offset(a);

        Edges {
            graph: self,
            node: a,
            state: 0,
            offset: offset.unwrap_or_else(|_| unsafe { unreachable_debug_checked() }),
        }
    }
}

pub struct EdgeReferences<'a, N, E, S, C> {
    g: &'a LatticeGraph<N, E, S>,
    e: Option<Edges<'a, N, E, S, C>>,
    index: usize,
}

impl<'a, N, E, S, C, D, A> Iterator for EdgeReferences<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    D: AxisDirection + Copy,
    A: Axis<Direction = D>,
{
    type Item = EdgeReference<'a, C, E, D, A>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut e) = self.e {
                let next = e.next();
                if next.is_some() {
                    return next;
                }
            }
            if self.index < self.g.s.node_counts() {
                let x = self.g.s.from_index(self.index);
                self.index += 1;
                self.e = Some(self.g.edges(x));
            } else {
                return None;
            }
        }
    }
}

impl<'a, N, E, S, C, D, A> IntoEdgeReferences for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    type EdgeRef = EdgeReference<'a, C, E, D, A>;
    type EdgeReferences = EdgeReferences<'a, N, E, S, C>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            g: self,
            e: None,
            index: 0,
        }
    }
}
