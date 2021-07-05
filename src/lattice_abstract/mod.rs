//! Module for Abstract 2D Lattice Graph. It is used inside by other lattice graph in other modules like [`hex`](`crate::hex`).
//! Use it when you want to define your own lattice graph, or to use the concreate visit iterator structs for traits in [`visit`](`petgraph::visit`).

use crate::{fixedvec2d::*, unreachable_debug_checked};
use fixedbitset::FixedBitSet;
use petgraph::{
    data::{DataMap, DataMapMut},
    visit::{Data, GraphBase, GraphProp, NodeCount, VisitMap, Visitable},
    EdgeType,
};
use std::{marker::PhantomData, num::NonZeroUsize, usize};
mod edges;
pub use edges::*;
mod neighbors;
pub use neighbors::*;
mod nodes;
pub use nodes::*;
pub mod shapes;
pub(crate) use shapes::*;
pub mod square;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Abstract Lattice Graph.
/// It holds the node and edge weight data.
/// The actural behaviour is dependent on [`Shape`](`shapes::Shape`).
pub struct LatticeGraph<N, E, S> {
    nodes: FixedVec2D<N>,
    edges: Vec<FixedVec2D<E>>,
    s: S,
}

impl<N, E, S: Shape> LatticeGraph<N, E, S> {
    /// Creates a graph from raw data.
    pub unsafe fn new_raw(nodes: FixedVec2D<N>, edges: Vec<FixedVec2D<E>>, s: S) -> Self {
        Self { nodes, edges, s }
    }

    /// Creates a graph with uninitalized node and edge weight data.
    pub unsafe fn new_uninit(s: S) -> Self {
        let nodes =
            FixedVec2D::<N>::new_uninit(NonZeroUsize::new(s.horizontal()).unwrap(), s.vertical());
        let ac = S::Axis::COUNT;
        let mut edges = Vec::with_capacity(ac);
        for i in 0..ac {
            let a = S::Axis::from_index_unchecked(i);
            edges.push(FixedVec2D::<E>::new_uninit(
                NonZeroUsize::new(s.horizontal_edge_size(a.clone())).unwrap(),
                s.vertical_edge_size(a),
            ))
        }
        Self { nodes, edges, s }
    }

    /// Creates a graph with node and edge weight data set to [`default`](`Default::default`).
    pub fn new(s: S) -> Self
    where
        S: Clone,
        N: Default,
        E: Default,
    {
        Self::new_with(s.clone(), |_| N::default(), |_, _| Some(E::default()))
    }

    /// Creates a graph with node and edge weight data from the coordinate.
    pub fn new_with<FN, FE>(s: S, mut n: FN, mut e: FE) -> Self
    where
        S: Clone,
        FN: FnMut(S::Coordinate) -> N,
        FE: FnMut(S::Coordinate, S::Axis) -> Option<E>,
    {
        let mut uninit = unsafe { Self::new_uninit(s.clone()) };
        let nodes = uninit.nodes.mut_1d();
        let edges = &mut uninit.edges;
        for i in 0..s.node_count() {
            let offset = s.index_to_offset(i);
            let c = s.from_offset(offset);
            nodes[i] = n(c);
            for j in 0..S::Axis::COUNT {
                let a = unsafe { <S::Axis as Axis>::from_index_unchecked(j) };
                if s.move_coord(c, a.foward()).is_err() {
                    continue;
                }
                if let Some(ex) = e(c, a) {
                    edges[j].mut_2d()[offset.horizontal][offset.vertical] = ex;
                }
            }
        }
        uninit
    }

    /// Get a reference to the lattice graph's s.
    pub fn shape(&self) -> &S {
        &self.s
    }
}

impl<N, E, S> Default for LatticeGraph<N, E, S>
where
    N: Default,
    E: Default,
    S: Shape + Default + Clone,
{
    fn default() -> Self {
        Self::new(S::default())
    }
}

impl<N, E, S: Shape> GraphBase for LatticeGraph<N, E, S> {
    type NodeId = S::Coordinate;
    type EdgeId = (S::Coordinate, S::Axis);
}

impl<N, E, S: Shape> Data for LatticeGraph<N, E, S> {
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<N, E, S: Shape> DataMap for LatticeGraph<N, E, S> {
    fn node_weight(self: &Self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        let offset = self.s.to_offset(id);
        // SAFETY : offset must be checked in `to_offset`
        offset
            .map(move |offset| unsafe {
                let nodes = self.nodes.ref_2d();
                if cfg!(debug_assert) {
                    nodes
                        .get(offset.horizontal)
                        .unwrap()
                        .get(offset.vertical)
                        .unwrap()
                } else {
                    nodes
                        .get_unchecked(offset.horizontal)
                        .get_unchecked(offset.vertical)
                }
            })
            .ok()
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            unsafe {
                self.edges
                    .get_unchecked(ax)
                    .ref_2d()
                    .get(offset.horizontal)?
                    .get(offset.vertical)
            }
        } else {
            None
        }
    }
}

impl<N, E, S: Shape> DataMapMut for LatticeGraph<N, E, S> {
    fn node_weight_mut(self: &mut Self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        let offset = self.s.to_offset(id);

        // SAFETY : offset must be checked in `to_offset`
        offset
            .map(move |offset| unsafe {
                let nodes = self.nodes.mut_2d();
                if cfg!(debug_assert) {
                    nodes
                        .get_mut(offset.horizontal)
                        .unwrap()
                        .get_mut(offset.vertical)
                        .unwrap()
                } else {
                    nodes
                        .get_unchecked_mut(offset.horizontal)
                        .get_unchecked_mut(offset.vertical)
                }
            })
            .ok()
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            unsafe {
                self.edges
                    .get_unchecked_mut(ax)
                    .mut_2d()
                    .get_mut(offset.horizontal)?
                    .get_mut(offset.vertical)
            }
        } else {
            None
        }
    }
}

///Wrapper for [`Axis`] to be [`EdgeType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeTypeWrap<A>(PhantomData<A>);
impl<A: Axis> EdgeType for EdgeTypeWrap<A> {
    fn is_directed() -> bool {
        A::DIRECTED
    }
}

impl<N, E, S: Shape> GraphProp for LatticeGraph<N, E, S> {
    type EdgeType = EdgeTypeWrap<S::Axis>;
}

/// [`VisitMap`] of [`LatticeGraph`].
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VisMap<S> {
    v: Vec<FixedBitSet>,
    s: S,
}

impl<S: Shape> VisMap<S> {
    pub fn new(s: S) -> Self {
        let h = s.horizontal();
        let v = s.vertical();
        let mut vec = Vec::with_capacity(h);
        for _ in 0..h {
            vec.push(FixedBitSet::with_capacity(v));
        }
        Self { v: vec, s }
    }
}

impl<S: Shape> VisitMap<S::Coordinate> for VisMap<S> {
    fn visit(&mut self, a: S::Coordinate) -> bool {
        let offset = self.s.to_offset(a);
        if let Ok(a) = offset {
            !self.v[a.horizontal].put(a.vertical)
        } else {
            false
        }
    }

    fn is_visited(&self, a: &S::Coordinate) -> bool {
        let offset = self.s.to_offset(a.clone());
        if let Ok(a) = offset {
            self.v[a.horizontal].contains(a.vertical)
        } else {
            false
        }
    }
}

impl<N, E, S: Shape + Clone> Visitable for LatticeGraph<N, E, S> {
    type Map = VisMap<S>;

    fn visit_map(self: &Self) -> Self::Map {
        VisMap::new(self.s.clone())
    }

    fn reset_map(self: &Self, map: &mut Self::Map) {
        map.v.iter_mut().for_each(|x| x.clear())
    }
}
