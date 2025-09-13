use std::iter::FusedIterator;

use petgraph::visit::{EdgeRef, IntoEdgeReferences, IntoEdges, IntoEdgesDirected};

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

/// Edges connected to a node. See [`IntoEdges`].
// Type parameter `C` is to derive `Debug`. (I don't want to impl manually).
#[derive(Debug)]
pub struct Edges<'a, N, E, S: Shape, C = <S as Shape>::Coordinate, Dt = AxisDirMarker> {
    graph: &'a LatticeGraph<N, E, S>,
    node: C,
    offset: Offset,
    state: usize,
    directed: Dt,
}

/// Marker Used for [`Edges`] inside the [`IntoEdgeReferences`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AxisMarker;
/// Marker Used for [`Edges`] as [`IntoEdges`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AxisDirMarker;
/// Marker for [`Edges`].
pub trait DtMarker {
    /// Whether the edges are directed.
    const DIRECTED: bool;
    // trick to be used in [`IntoEdgesDirected`]
    #[inline]
    unsafe fn get_raw_id<S: Shape>(
        &self,
        s: &S,
        d: &<<S as Shape>::Axis as Axis>::Direction,
        source: Offset,
        target: <S as Shape>::Coordinate,
        st: usize,
    ) -> (Offset, usize) {
        if <S as Shape>::Axis::is_forward_direction(d) {
            (source, st)
        } else {
            (
                s.to_offset_unchecked(target),
                st - <S as Shape>::Axis::COUNT,
            )
        }
    }
    #[inline(always)]
    fn need_reverse(&self) -> bool {
        false
    }
}

impl DtMarker for AxisMarker {
    const DIRECTED: bool = true;
}
impl DtMarker for AxisDirMarker {
    const DIRECTED: bool = false;
}

/// [`petgraph::Direction`] as marker for [`Edges`] used in [`IntoEdgesDirected`].
impl DtMarker for petgraph::Direction {
    const DIRECTED: bool = false;

    unsafe fn get_raw_id<S: Shape>(
        &self,
        s: &S,
        d: &<<S as Shape>::Axis as Axis>::Direction,
        source: Offset,
        target: <S as Shape>::Coordinate,
        st: usize,
    ) -> (Offset, usize) {
        match (self, <S as Shape>::Axis::DIRECTED) {
            (petgraph::EdgeDirection::Incoming, true) => {
                let da = <S as Shape>::Axis::from_direction(d.clone())
                    .backward()
                    .dir_to_index();
                let o = s.to_offset_unchecked(target);
                (o, da)
            }
            _ => AxisDirMarker.get_raw_id(s, d, source, target, st),
        }
    }
    fn need_reverse(&self) -> bool {
        self == &petgraph::Direction::Incoming
    }
}

impl<'a, N, E, S, C, D, A, Dt> Edges<'a, N, E, S, C, Dt>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection,
{
    fn new(g: &'a LatticeGraph<N, E, S>, a: C) -> Edges<'a, N, E, S, C, Dt>
    where
        Dt: Default,
    {
        Self::new_d(g, a, Dt::default())
    }

    fn new_d(g: &'a LatticeGraph<N, E, S>, a: C, d: Dt) -> Edges<'a, N, E, S, C, Dt> {
        let offset = g.s.to_offset(a);
        Edges {
            graph: g,
            node: a,
            state: if offset.is_ok() {
                0
            } else {
                S::Axis::UNDIRECTED_COUNT
            },
            offset: offset.unwrap_or_else(|_| unsafe { unreachable_debug_checked() }),
            directed: d,
        }
    }

    unsafe fn new_unchecked_from_offset(
        g: &'a LatticeGraph<N, E, S>,
        offset: Offset,
    ) -> Edges<'a, N, E, S, C, Dt>
    where
        Dt: Default,
    {
        // Create from offset directly, avoiding coordinate conversion
        let coord = g.s.offset_to_coordinate(offset);
        Edges {
            graph: g,
            node: coord,
            state: 0,
            offset,
            directed: Dt::default(),
        }
    }
}

impl<'a, N, E, S, C, D, A, Dt> Iterator for Edges<'a, N, E, S, C, Dt>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection,
    Dt: DtMarker,
{
    type Item = EdgeReference<'a, C, E, D, A>;

    fn next(&mut self) -> Option<Self::Item> {
        let max_state = if Dt::DIRECTED {
            A::COUNT
        } else {
            A::UNDIRECTED_COUNT
        };

        while self.state < max_state {
            unsafe {
                let d = D::dir_from_index_unchecked(self.state);
                let st = self.state;
                self.state += 1;

                // Try to move offset directly without coordinate conversion
                if let Ok(target_offset) = self.graph.s.move_offset(self.offset, &d) {
                    // Get target coordinate only when needed
                    let target = self.graph.s.offset_to_coordinate(target_offset);

                    let (nx, ne) =
                        self.directed
                            .get_raw_id(&self.graph.s, &d, self.offset, target, st);
                    debug_assert_eq!(A::from_direction(d.clone()).to_index(), ne);

                    let e = self.graph.edge_weight_unchecked_raw((nx, ne));
                    let (source_id, target_id) = if self.directed.need_reverse() {
                        (target, self.node)
                    } else {
                        (self.node, target)
                    };
                    return Some(EdgeReference {
                        source_id,
                        target_id,
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
        let x = if Dt::DIRECTED {
            A::COUNT
        } else {
            A::UNDIRECTED_COUNT
        } - self.state;
        (0, Some(x))
    }
}

impl<'a, N, E, S, C, Dt> FusedIterator for Edges<'a, N, E, S, C, Dt>
where
    Self: Iterator,
    S: Shape,
{
}

impl<'a, N, E, S, C, D, A> IntoEdges for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    type Edges = Edges<'a, N, E, S>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        Edges::new(self, a)
    }
}

/// Edges connected to a node with [`Direction`](`petgraph::Direction`). See [`IntoEdgesDirected`].
pub type EdgesDirected<'a, N, E, S> =
    Edges<'a, N, E, S, <S as Shape>::Coordinate, petgraph::Direction>;
impl<'a, N, E, S, C, D, A> IntoEdgesDirected for &'a LatticeGraph<N, E, S>
where
    C: Copy,
    S: Shape<Coordinate = C, Axis = A>,
    A: Axis<Direction = D>,
    D: AxisDirection + Copy,
{
    type EdgesDirected = EdgesDirected<'a, N, E, S>;

    fn edges_directed(self, a: Self::NodeId, dir: petgraph::Direction) -> Self::EdgesDirected {
        Edges::new_d(self, a, dir)
    }
}

/// Iterator for all edges of [`LatticeGraph`]. See [`IntoEdgeReferences`](`IntoEdgeReferences::edge_references`).
// Type parameter `C` is to derive `Debug`. (I don't want to impl manually).
#[derive(Debug)]
pub struct EdgeReferences<'a, N, E, S: Shape, C = <S as Shape>::Coordinate> {
    g: &'a LatticeGraph<N, E, S>,
    e: Option<Edges<'a, N, E, S, C, AxisMarker>>,
    current_offset: shapes::Offset,
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
            if self.current_offset.horizontal < self.g.s.horizontal() {
                // Use the current offset directly
                let current = self.current_offset;
                // Move to next position (row-major order for cache efficiency)
                self.current_offset.vertical += 1;
                if self.current_offset.vertical >= self.g.s.vertical() {
                    self.current_offset.vertical = 0;
                    self.current_offset.horizontal += 1;
                }
                // Create Edges directly from offset
                self.e = Some(unsafe { Edges::new_unchecked_from_offset(self.g, current) });
            } else {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_nodes = if self.current_offset.horizontal < self.g.s.horizontal() {
            let remaining_in_current_row = self.g.s.vertical() - self.current_offset.vertical;
            let remaining_rows = self.g.s.horizontal() - self.current_offset.horizontal - 1;
            remaining_in_current_row + remaining_rows * self.g.s.vertical()
        } else {
            0
        };
        let maxlen = remaining_nodes * S::Axis::UNDIRECTED_COUNT
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
    type EdgeReferences = EdgeReferences<'a, N, E, S>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            g: self,
            e: None,
            current_offset: shapes::Offset::new(0, 0),
        }
    }
}
