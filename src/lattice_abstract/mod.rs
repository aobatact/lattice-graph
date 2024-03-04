//! Module for Abstract 2D Lattice Graph. It is used inside by other lattice graph in other modules like [`hex`](`crate::hex`).
//! Use it when you want to define your own lattice graph, or to use the concreate visit iterator structs for traits in [`visit`](`petgraph::visit`).

use crate::{fixedvec2d::*, unreachable_debug_checked};
use fixedbitset::FixedBitSet;
use petgraph::{
    data::{DataMap, DataMapMut},
    visit::{Data, GraphBase, GraphProp, IntoNodeIdentifiers, NodeCount, VisitMap, Visitable},
    EdgeType,
};
use std::{marker::PhantomData, mem::MaybeUninit, num::NonZeroUsize, ptr::drop_in_place};
mod edges;
pub use edges::{EdgeReference, EdgeReferences, Edges, EdgesDirected};
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
pub struct LatticeGraph<N, E, S: Shape> {
    nodes: FixedVec2D<N>,
    edges: Vec<FixedVec2D<E>>,
    s: S,
}

impl<N, E, S: Shape> LatticeGraph<N, E, S> {
    /// Creates a graph from raw data. This api might change.
    #[doc(hidden)]
    pub unsafe fn new_raw(nodes: FixedVec2D<N>, edges: Vec<FixedVec2D<E>>, s: S) -> Self {
        Self { nodes, edges, s }
    }

    /// Creates a graph with uninitalized node and edge weight data.
    /// It is extremely unsafe so should use with [`MaybeUninit`](`core::mem::MaybeUninit`) and use [`assume_init`](`Self::assume_init`).
    pub unsafe fn new_uninit(s: S) -> Self {
        let nodes =
            FixedVec2D::<N>::new_uninit(NonZeroUsize::new(s.horizontal()).unwrap(), s.vertical());
        let ac = S::Axis::COUNT;
        let mut edges = Vec::with_capacity(ac);
        for _i in 0..ac {
            edges.push(FixedVec2D::<E>::new_uninit(
                NonZeroUsize::new(s.horizontal()).unwrap(),
                s.vertical(),
            ))
        }
        Self { nodes, edges, s }
    }

    /// Creates a graph with node and edge weight data set to [`default`](`Default::default`).
    pub fn new(s: S) -> Self
    where
        N: Default,
        E: Default,
    {
        Self::new_with(s, |_| N::default(), |_, _| E::default())
    }

    /// Creates a graph with node and edge weight data from the coordinate.
    pub fn new_with<FN, FE>(s: S, mut n: FN, mut e: FE) -> Self
    where
        FN: FnMut(S::Coordinate) -> N,
        FE: FnMut(S::Coordinate, S::Axis) -> E, // change to E ?
    {
        let mut uninit = unsafe { Self::new_uninit(s) };
        let s = &uninit.s;
        let nodes = uninit.nodes.mut_1d();
        let edges = &mut uninit.edges;
        for i in 0..s.node_count() {
            let offset = s.index_to_offset(i);
            let c = s.offset_to_coordinate(offset);
            unsafe { std::ptr::write(nodes.get_unchecked_mut(i), n(c)) }
            for j in 0..S::Axis::COUNT {
                let a = unsafe { <S::Axis as Axis>::from_index_unchecked(j) };
                if s.move_coord(c, a.foward()).is_err() {
                    continue;
                }
                let ex = e(c, a);
                let t = edges[j]
                    .mut_2d()
                    .get_mut(offset.horizontal)
                    .and_then(|x| x.get_mut(offset.vertical));
                if let Some(x) = t {
                    unsafe { std::ptr::write(x, ex) };
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

impl<N, E, S: Shape + Default> LatticeGraph<N, E, S> {
    /// Creates a graph with node and edge weight data set to [`default`](`Default::default`) with [`Shape`] from default.
    pub fn new_s() -> Self
    where
        N: Default,
        E: Default,
    {
        Self::new(S::default())
    }

    /// Creates a graph with uninitalized node and edge weight data with [`Shape`] from default.
    /// It is extremely unsafe so should use with [`MaybeUninit`](`core::mem::MaybeUninit`) and use [`assume_init`](`Self::assume_init`).
    pub unsafe fn new_uninit_s() -> Self {
        Self::new_uninit(S::default())
    }

    /// Creates a graph with node and edge weight data from the coordinate with [`Shape`] from default.
    pub fn new_with_s<FN, FE>(n: FN, e: FE) -> Self
    where
        FN: FnMut(S::Coordinate) -> N,
        FE: FnMut(S::Coordinate, S::Axis) -> E,
    {
        Self::new_with(S::default(), n, e)
    }
}

impl<N, E, S: Shape> LatticeGraph<MaybeUninit<N>, MaybeUninit<E>, S> {
    /**
    Assume the underlying nodes and edges to be initialized.
    ```
    # use lattice_graph::hex::axial_based::*;
    # use core::mem::MaybeUninit;
    # use petgraph::data::*;
    let mut hex = unsafe { HexGraphConst::<MaybeUninit<f32>, MaybeUninit<()>, OddR, 5, 5>::new_uninit_s() };
    for i in 0..5{
        for j in 0..5{
            let offset = Offset::new(i, j);
            let coord = hex.shape().offset_to_coordinate(offset);
            if let Some(ref mut n) = hex.node_weight_mut(coord){
                **n = MaybeUninit::new((i + j) as f32);
            }
        }
    }
    let hex_init = unsafe{ hex.assume_init() };
    ```
    */
    pub unsafe fn assume_init(self) -> LatticeGraph<N, E, S> {
        let md = std::mem::ManuallyDrop::new(self);
        LatticeGraph {
            nodes: core::ptr::read(&md.nodes).assume_init(),
            edges: core::ptr::read(&md.edges)
                .into_iter()
                .map(|e| e.assume_init())
                .collect(),
            s: core::ptr::read(&md.s),
        }
    }
}

impl<N, E, S: Shape> Drop for LatticeGraph<N, E, S> {
    fn drop(&mut self) {
        // if e is drop type, drop manually to prevent dropping for invalid (uninitialized) edges.
        if std::mem::needs_drop::<E>() {
            let ni = self.node_identifiers();
            let s = &self.s;
            let e = &mut self.edges;
            unsafe {
                for (di, edges) in e.drain(..).enumerate() {
                    let dir = S::Axis::from_index_unchecked(di).foward();
                    for (coord, mut e) in ni.clone().zip(edges.into_raw()) {
                        if s.move_coord(coord, dir.clone()).is_ok() {
                            drop_in_place(&mut e);
                        }
                    }
                }
            }
        }
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
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
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

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            if self.s.move_coord(id.0, id.1.foward()).is_err() {
                return None;
            }
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
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
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

    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            if self.s.move_coord(id.0, id.1.foward()).is_err() {
                return None;
            }
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

impl<N, E, S: Shape> LatticeGraph<N, E, S> {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn node_weight_unchecked(
        &self,
        id: <LatticeGraph<N, E, S> as GraphBase>::NodeId,
    ) -> &<LatticeGraph<N, E, S> as Data>::NodeWeight {
        let offset = self.s.to_offset_unchecked(id);
        // SAFETY : offset must be checked in `to_offset`
        self.node_weight_unchecked_raw(offset)
    }

    #[doc(hidden)]
    pub unsafe fn node_weight_unchecked_raw(
        &self,
        offset: Offset,
    ) -> &<LatticeGraph<N, E, S> as Data>::NodeWeight {
        self.nodes
            .ref_2d()
            .get_unchecked(offset.horizontal)
            .get_unchecked(offset.vertical)
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn edge_weight_unchecked(
        &self,
        id: <LatticeGraph<N, E, S> as GraphBase>::EdgeId,
    ) -> &<LatticeGraph<N, E, S> as Data>::EdgeWeight {
        let offset = self.s.to_offset_unchecked(id.0);
        let ax = id.1.to_index();
        self.edge_weight_unchecked_raw((offset, ax))
    }

    #[doc(hidden)]
    pub unsafe fn edge_weight_unchecked_raw(
        &self,
        (offset, ax): (Offset, usize),
    ) -> &<LatticeGraph<N, E, S> as Data>::EdgeWeight {
        self.edges
            .get_unchecked(ax)
            .ref_2d()
            .get_unchecked(offset.horizontal)
            .get_unchecked(offset.vertical)
    }

    #[doc(hidden)]
    pub unsafe fn node_weight_mut_unchecked(
        &mut self,
        id: <LatticeGraph<N, E, S> as GraphBase>::NodeId,
    ) -> &mut <LatticeGraph<N, E, S> as Data>::NodeWeight {
        let offset = self.s.to_offset_unchecked(id);
        // SAFETY : offset must be checked in `to_offset`
        let nodes = self.nodes.mut_2d();

        nodes
            .get_unchecked_mut(offset.horizontal)
            .get_unchecked_mut(offset.vertical)
    }

    #[doc(hidden)]
    pub unsafe fn edge_weight_mut_unchecked(
        &mut self,
        id: <LatticeGraph<N, E, S> as GraphBase>::EdgeId,
    ) -> &mut <LatticeGraph<N, E, S> as Data>::EdgeWeight {
        let offset = self.s.to_offset_unchecked(id.0);
        let ax = id.1.to_index();
        self.edges
            .get_unchecked_mut(ax)
            .mut_2d()
            .get_unchecked_mut(offset.horizontal)
            .get_unchecked_mut(offset.vertical)
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
    pub(crate) fn new(s: S) -> Self {
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
        let offset = self.s.to_offset(*a);
        if let Ok(a) = offset {
            self.v[a.horizontal].contains(a.vertical)
        } else {
            false
        }
    }
}

impl<N, E, S: Shape + Clone> Visitable for LatticeGraph<N, E, S> {
    type Map = VisMap<S>;

    fn visit_map(&self) -> Self::Map {
        VisMap::new(self.s.clone())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.v.iter_mut().for_each(|x| x.clear())
    }
}
