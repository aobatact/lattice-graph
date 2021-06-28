use std::{convert::Infallible, marker::PhantomData, num::NonZeroUsize, usize};

use petgraph::{
    data::{DataMap, DataMapMut},
    visit::{Data, GraphBase, GraphProp},
    EdgeType,
};
mod edges;

use crate::fixedvec2d::*;

/// Shape of the 2d lattice.
pub trait Shape {
    type Axis: Axis;
    type Coordinate: Coordinate;
    type OffsetConvertError;
    type CoordinateMoveError;

    /// Convert coordinate to Offset.
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError>;
    /// Convert coordinate to Offset without a check.
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset;
    /// Convert coordinate from Offset.
    fn from_offset(&self, offset: Offset) -> Self::Coordinate;
    /// Horizontal node count.
    fn horizontal(&self) -> usize;
    /// Vertical node count.
    fn vertical(&self) -> usize;
    /// Edge count of horizontal. May differ by the axis info.
    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize;
    /// Edge count of vertical. May differ by the axis info.
    fn vertical_edge_size(&self, axis: Self::Axis) -> usize;
    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError>;
    // fn normalize_coord(&self,
    //     coord: Self::Coordinate,
    //     dir: <Self::Axis as Axis>::Direction,
    // ) -> Result<(Self::Coordinate, Self::Axis), Self::CoordinateMoveError>;
}

impl<S: Shape> Shape for &S {
    type Axis = S::Axis;

    type Coordinate = S::Coordinate;

    type OffsetConvertError = S::OffsetConvertError;

    type CoordinateMoveError = S::CoordinateMoveError;

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        (*self).to_offset(coord)
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        (*self).to_offset_unchecked(coord)
    }

    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        (*self).from_offset(offset)
    }

    fn horizontal(&self) -> usize {
        (*self).horizontal()
    }

    fn vertical(&self) -> usize {
        (*self).vertical()
    }

    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize {
        (*self).horizontal_edge_size(axis)
    }

    fn vertical_edge_size(&self, axis: Self::Axis) -> usize {
        (*self).vertical_edge_size(axis)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        (*self).move_coord(coord, dir)
    }
}

pub trait Axis: Copy + PartialEq {
    const COUNT: usize;
    const DIRECTED: bool;
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction: AxisDirection;
    fn to_index(&self) -> usize;
    unsafe fn from_index_unchecked(index: usize) -> Self;
    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized;
    fn foward(&self) -> Self::Direction;
    fn backward(&self) -> Self::Direction;
    fn from_direction(dir: Self::Direction) -> Self;
}

pub trait AxisDirection {
    fn is_forward(&self) -> bool;
    fn is_backward(&self) -> bool {
        !self.is_forward()
    }
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
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };

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

    fn foward(&self) -> Self::Direction {
        Direction::Foward(self.clone())
    }

    fn backward(&self) -> Self::Direction {
        Direction::Backward(self.clone())
    }

    fn from_direction(dir: Self::Direction) -> Self {
        match dir {
            Direction::Foward(a) | Direction::Backward(a) => a,
        }
    }
}

pub enum Direction<T> {
    Foward(T),
    Backward(T),
}

impl<T> AxisDirection for Direction<T> {
    fn is_forward(&self) -> bool {
        match self {
            Direction::Foward(_) => true,
            Direction::Backward(_) => false,
        }
    }

    fn to_index(&self) -> usize {
        todo!()
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        todo!()
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset(usize, usize);

impl Offset {
    pub fn add_x(&self, x: usize) -> Self {
        Offset(self.0 + x, self.1)
    }
    pub fn add_y(&self, y: usize) -> Self {
        Offset(self.0, self.1 + y)
    }
    pub fn sub_x(&self, x: usize) -> Option<Self> {
        Some(Offset(self.0.checked_sub(x)?, self.1))
    }
    pub fn sub_y(&self, y: usize) -> Option<Self> {
        Some(Offset(self.0, self.1.checked_sub(y)?))
    }
    pub unsafe fn sub_x_unchecked(&self, x: usize) -> Self {
        Offset(self.0 - x, self.1)
    }
    pub unsafe fn sub_y_unchecked(&self, y: usize) -> Self {
        Offset(self.0, self.1 - y)
    }
    pub fn check_x(&self, x_max: usize) -> Option<Self> {
        if self.0 < x_max {
            Some(*self)
        } else {
            None
        }
    }
    pub fn check_y(&self, y_max: usize) -> Option<Self> {
        if self.1 < y_max {
            Some(*self)
        } else {
            None
        }
    }
}

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
    type CoordinateMoveError = ();

    #[inline]
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, ()> {
        if coord.0 .0 < self.horizontal() && coord.0 .1 < self.vertical() {
            Ok(coord.0)
        } else {
            Err(())
        }
    }

    #[inline]
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
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

    fn move_coord(
        &self,
        coord: SquareOffset,
        dir: Direction<SquareAxis>,
    ) -> Result<SquareOffset, ()> {
        let o = match dir {
            Direction::Foward(SquareAxis::X) => coord.0.add_x(1).check_x(self.h),
            Direction::Foward(SquareAxis::Y) => coord.0.add_y(1).check_y(self.v),
            Direction::Backward(SquareAxis::X) => coord.0.sub_x(1),
            Direction::Backward(SquareAxis::Y) => coord.0.sub_y(1),
        };
        o.map(|s| SquareOffset(s)).ok_or_else(|| ())
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
        // SAFETY : offset must be checked in `to_offset`
        offset
            .map(move |offset| unsafe {
                let nodes = self.nodes.ref_2d();
                if cfg!(debug_assert) {
                    nodes.get(offset.0).unwrap().get(offset.1).unwrap()
                } else {
                    nodes.get_unchecked(offset.0).get_unchecked(offset.1)
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

        // SAFETY : offset must be checked in `to_offset`
        offset
            .map(move |offset| unsafe {
                let nodes = self.nodes.mut_2d();
                if cfg!(debug_assert) {
                    nodes.get_mut(offset.0).unwrap().get_mut(offset.1).unwrap()
                } else {
                    nodes
                        .get_unchecked_mut(offset.0)
                        .get_unchecked_mut(offset.1)
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
