use std::{convert::Infallible, marker::PhantomData, num::NonZeroUsize, usize};

use petgraph::{
    data::{DataMap, DataMapMut},
    visit::{Data, GraphBase, GraphProp},
    EdgeType,
};

use crate::fixedvec2d::*;

pub trait Shape {
    type Axis: Axis;
    type Coordinate: Coordinate;
    type OffsetConvertError;

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError>;
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset;
    fn from_offset(&self, offset: Offset) -> Self::Coordinate;
    fn horizontal(&self) -> usize;
    fn vertical(&self) -> usize;
    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize;
    fn vertical_edge_size(&self, axis: Self::Axis) -> usize;
}

pub trait Axis: Copy + PartialEq {
    const COUNT: usize;
    const DIRECTED: bool;
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction;
    fn to_index(&self) -> usize;
    unsafe fn from_index_unchecked(index: usize) -> Self;
    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SquareAxis {
    X = 0,
    Y = 1,
}

impl Axis for SquareAxis {
    const COUNT: usize = 2;
    const DIRECTED: bool = false;
    type Direction = Direction<Self>;

    fn to_index(&self) -> usize {
        match self {
            SquareAxis::X => 0,
            SquareAxis::Y => 1,
        }
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => SquareAxis::X,
            1 => SquareAxis::Y,
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        match index {
            0 => Some(SquareAxis::X),
            1 => Some(SquareAxis::Y),
            _ => None,
        }
    }
}

pub enum Direction<T> {
    Foward(T),
    Backward(T),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset(usize, usize);

impl<T: Into<usize>> From<(T, T)> for Offset {
    fn from(x: (T, T)) -> Self {
        Offset(x.0.into(), x.1.into())
    }
}

impl<T: From<usize>> From<Offset> for (T, T) {
    fn from(x: Offset) -> Self {
        (x.0.into(), x.1.into())
    }
}

pub trait Coordinate: Copy + PartialEq {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SquareOffset(pub(crate) Offset);

impl Coordinate for SquareOffset {}

pub struct SquareShape {
    h: usize,
    v: usize,
}

impl Shape for SquareShape {
    type Axis = SquareAxis;

    type Coordinate = SquareOffset;

    type OffsetConvertError = ();

    #[inline]
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
    }

    #[inline]
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, ()> {
        if coord.0 .0 < self.horizontal() && coord.0 .1 < self.vertical() {
            Ok(coord.0)
        } else {
            Err(())
        }
    }

    #[inline]
    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        SquareOffset(offset)
    }

    #[inline]
    fn horizontal(&self) -> usize {
        self.h
    }

    #[inline]
    fn vertical(&self) -> usize {
        self.v
    }

    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize {
        let h = self.horizontal();
        match axis {
            SquareAxis::X => h - 1,
            SquareAxis::Y => h,
        }
    }

    fn vertical_edge_size(&self, axis: Self::Axis) -> usize {
        let v = self.vertical();
        match axis {
            SquareAxis::X => v,
            SquareAxis::Y => v - 1,
        }
    }
}

impl EdgeType for SquareShape {
    fn is_directed() -> bool {
        <Self as Shape>::Axis::DIRECTED
    }
}

pub struct LatticeGraph<N, E, S> {
    nodes: FixedVec2D<N>,
    edges: Vec<FixedVec2D<E>>,
    s: S,
}

impl<N, E, S: Shape> LatticeGraph<N, E, S> {
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
        if let Ok(offset) = offset {
            self.nodes.ref_2d().get(offset.0)?.get(offset.1)
        } else {
            None
        }
    }

    fn edge_weight(self: &Self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            unsafe {
                self.edges
                    .get_unchecked(ax)
                    .ref_2d()
                    .get(offset.0)?
                    .get(offset.1)
            }
        } else {
            None
        }
    }
}

impl<N, E, S: Shape> DataMapMut for LatticeGraph<N, E, S> {
    fn node_weight_mut(self: &mut Self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        let offset = self.s.to_offset(id);
        if let Ok(offset) = offset {
            self.nodes.mut_2d().get_mut(offset.0)?.get_mut(offset.1)
        } else {
            None
        }
    }

    fn edge_weight_mut(self: &mut Self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        let offset = self.s.to_offset(id.0);
        let ax = id.1.to_index();
        if let Ok(offset) = offset {
            unsafe {
                self.edges
                    .get_unchecked_mut(ax)
                    .mut_2d()
                    .get_mut(offset.0)?
                    .get_mut(offset.1)
            }
        } else {
            None
        }
    }
}

impl<N, E, S: Shape + EdgeType> GraphProp for LatticeGraph<N, E, S> {
    type EdgeType = S;
}
