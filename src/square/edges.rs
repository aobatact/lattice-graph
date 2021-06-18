use super::*;
use petgraph::{
    graph::IndexType,
    visit::{EdgeRef, IntoEdgeReferences, IntoEdges},
};
use std::{iter::FusedIterator, ops::Range};

impl<'a, N, E, Ix> IntoEdgeReferences for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    E: Copy,
    Range<Ix>: Iterator<Item = Ix>,
{
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences::new(&self)
    }
}

/// Reference of Edge data (EdgeIndex, EdgeWeight, direction) in [`SquareGraph`].
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeReference<'a, E, Ix: IndexType> {
    pub(crate) edge_id: EdgeIndex<Ix>,
    pub(crate) edge_weight: &'a E,
    pub(crate) direction: bool,
}

impl<'a, E, Ix: IndexType> Clone for EdgeReference<'a, E, Ix> {
    fn clone(&self) -> Self {
        Self {
            edge_id: self.edge_id,
            edge_weight: self.edge_weight,
            direction: self.direction,
        }
    }
}

impl<'a, E, Ix: IndexType> Copy for EdgeReference<'a, E, Ix> {}

impl<'a, E, Ix: IndexType> EdgeReference<'a, E, Ix> {
    #[inline]
    fn get_node(&self, is_source: bool) -> NodeIndex<Ix> {
        if is_source {
            (self.edge_id).node
        } else {
            match (self.edge_id).axis {
                Axis::Horizontal => (self.edge_id).node.right(),
                Axis::Vertical => (self.edge_id).node.up(),
            }
        }
    }
}

impl<'a, E: Copy, Ix: IndexType> EdgeRef for EdgeReference<'a, E, Ix> {
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
    type Weight = E;

    #[inline]
    fn source(&self) -> Self::NodeId {
        self.get_node(self.direction)
    }

    #[inline]
    fn target(&self) -> Self::NodeId {
        self.get_node(!self.direction)
    }

    fn weight(&self) -> &Self::Weight {
        self.edge_weight
    }

    fn id(&self) -> Self::EdgeId {
        self.edge_id
    }
}

/// Iterator for all edges of [`SquareGraph`].
#[derive(Clone, Debug)]
pub struct EdgeReferences<'a, E, Ix: IndexType> {
    horizontal: &'a FixedVec2D<E>,
    vertical: &'a FixedVec2D<E>,
    nodes: NodeIndices<Ix>,
    prv: Option<EdgeIndex<Ix>>,
}

impl<'a, E, Ix: IndexType> EdgeReferences<'a, E, Ix> {
    fn new<N>(graph: &'a SquareGraph<N, E, Ix>) -> Self {
        Self {
            horizontal: &graph.horizontal,
            vertical: &graph.vertical,
            nodes: NodeIndices::new(graph.horizontal_node_count(), graph.vertical_node_count()),
            prv: None,
        }
    }
}

impl<'a, E, Ix> Iterator for EdgeReferences<'a, E, Ix>
where
    Ix: IndexType,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut e) = self.prv {
                if e.axis == Axis::Horizontal {
                    let item = self.vertical.ref_2d()[e.node.horizontal.index()]
                        .get(e.node.vertical.index());
                    if let Some(item) = item {
                        e.axis = Axis::Vertical;
                        return Some(EdgeReference {
                            edge_id: *e,
                            edge_weight: item,
                            direction: true,
                        });
                    }
                }
            }
            if let Some(next) = self.nodes.next() {
                let item = self
                    .horizontal
                    .ref_2d()
                    .get(next.horizontal.index())
                    .map(|x| x.get(next.vertical.index()))
                    .flatten();
                let edge_id = EdgeIndex {
                    node: next,
                    axis: Axis::Horizontal,
                };
                self.prv = Some(edge_id);
                if let Some(edge_weight) = item {
                    return Some(EdgeReference {
                        edge_id,
                        edge_weight,
                        direction: true,
                    });
                }
            } else {
                return None;
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.nodes.size_hint();
        (
            lo.saturating_sub(self.horizontal.size() + self.vertical.size()),
            hi.map(|x| x * 2),
        )
    }
}

pub struct Edges<'a, N, E, Ix: IndexType> {
    g: &'a SquareGraph<N, E, Ix>,
    node: NodeIndex<Ix>,
    state: usize,
}

impl<'a, N, E, Ix> Iterator for Edges<'a, N, E, Ix>
where
    Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        let g = self.g;
        let n = self.node;
        loop {
            'inner: loop {
                let er = match self.state {
                    0 => {
                        if n.horizontal.index() == 0 {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex {
                                node: n.left(),
                                axis: Axis::Horizontal,
                            },
                            edge_weight: unsafe {
                                g.horizontal
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index() - 1)
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: false,
                        }
                    }
                    1 => {
                        if n.horizontal.index() + 1 >= g.horizontal_node_count() {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex {
                                node: n,
                                axis: Axis::Horizontal,
                            },
                            edge_weight: unsafe {
                                g.horizontal
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: true,
                        }
                    }
                    2 => {
                        if n.vertical.index() == 0 {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex {
                                node: n.down(),
                                axis: Axis::Vertical,
                            },
                            edge_weight: unsafe {
                                g.vertical
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index() - 1)
                            },
                            direction: false,
                        }
                    }
                    3 => {
                        if n.vertical.index() + 1 >= g.vertical_node_count() {
                            break 'inner;
                        }
                        EdgeReference {
                            edge_id: EdgeIndex {
                                node: n,
                                axis: Axis::Vertical,
                            },
                            edge_weight: unsafe {
                                g.vertical
                                    .ref_2d()
                                    .get_unchecked(n.horizontal.index())
                                    .get_unchecked(n.vertical.index())
                            },
                            direction: true,
                        }
                    }
                    _ => return None,
                };
                self.state += 1;
                return Some(er);
            }
            self.state += 1;
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(4 - self.state))
    }
}

impl<'a, N, E, Ix> FusedIterator for Edges<'a, N, E, Ix> where Ix: IndexType {}

impl<'a, N, E, Ix> IntoEdges for &'a SquareGraph<N, E, Ix>
where
    Ix: IndexType,
    E: Copy,
    Range<Ix>: Iterator<Item = Ix>,
{
    type Edges = Edges<'a, N, E, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        Edges {
            g: &self,
            node: a,
            state: 0,
        }
    }
}
