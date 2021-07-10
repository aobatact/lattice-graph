use std::iter::FusedIterator;

use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoEdges};

use super::*;

/// Edge reference data. See [`IntoEdgeReferences`] or [`IntoEdges`].
#[derive(Debug, PartialEq, Eq)]
pub struct EdgeReference<'a, C, E, D, A> {
    pub(crate) source_id: C,
    pub(crate) target_id: C,
    pub(crate) edge_weight: &'a E,
    pub(crate) direction: D,
    pub(crate) axis: PhantomData<fn() -> A>,
}

impl<'a, C, E, D, A> EdgeReference<'a, C, E, D, A> {
    /// Get a reference to the edge reference's direction.
    pub fn direction(&self) -> &D {
        &self.direction
    }
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

/// Edges connected to a node. See [`edges`][`IntoEdges::edges`].
#[derive(Debug)]
pub struct Edges<'a, N, E, S, C> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    offset: Offset,
    state: usize,
}

impl<'a, N, E, S, C, D, A> Edges<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Clone,
{
    fn new(g: &'a LatticeGraph<N, E, S>, a: C) -> Edges<N, E, S, C> {
        let offset = g.s.to_offset(a);
        Edges {
            graph: g,
            node: a,
            state: if offset.is_ok() {
                0
            } else {
                S::Axis::DIRECTED_COUNT
            },
            offset: offset.unwrap_or_else(|_| unsafe { unreachable_debug_checked() }),
        }
    }

    unsafe fn new_unchecked(g: &'a LatticeGraph<N, E, S>, a: C) -> Edges<N, E, S, C> {
        let offset = g.s.to_offset(a);
        Edges {
            graph: g,
            node: a,
            state: 0,
            offset: offset.unwrap_or_else(|_| unreachable_debug_checked()),
        }
    }
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
        while self.state < A::DIRECTED_COUNT {
            unsafe {
                let d = D::dir_from_index_unchecked(self.state);
                let n = self.graph.s.move_coord(self.node, d.clone());
                let st = self.state;
                self.state += 1;
                if let Ok(target) = n {
                    let (nx, ne) = if A::is_forward_direction(&d) {
                        (self.offset, st)
                    } else {
                        (self.graph.s.to_offset_unchecked(target), st - A::COUNT)
                    };
                    debug_assert_eq!(A::from_direction(d.clone()).to_index(), ne);
                    //let ne = S::Axis::from_direction(d.clone()).to_index();
                    let e = &self
                        .graph
                        .edges
                        .get_unchecked(ne)
                        .ref_2d()
                        .get_unchecked(nx.horizontal)
                        .get_unchecked(nx.vertical);
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
        Edges::new(self, a)
    }
}

/// Iterator for all edges of [`LatticeGraph`]. See [`IntoEdgeReferences`](`IntoEdgeReferences::edge_references`).
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
            if self.index < self.g.s.node_count() {
                let x = self.g.s.from_index(self.index);
                self.index += 1;
                //self.e = Some(self.g.edges(x));
                self.e = Some(unsafe { Edges::new_unchecked(self.g, x) });
            } else {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let node_len = self.g.node_count() - self.index;
        let maxlen = node_len * S::Axis::DIRECTED_COUNT
            + self
                .e
                .as_ref()
                .map(|x| x.size_hint().1.unwrap_or(0))
                .unwrap_or(0);
        (0, Some(maxlen))
    }
}

impl<'a, N, E, S, C, D, A> FusedIterator for EdgeReferences<'a, N, E, S, C>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    D: AxisDirection + Copy,
    A: Axis<Direction = D>,
{
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
