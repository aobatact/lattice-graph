use crate::unreachable_debug_checked;

/// Shape of the 2d lattice.
pub trait Shape {
    type Axis: Axis;
    type Coordinate: Coordinate;
    type OffsetConvertError: core::fmt::Debug + Clone + Default;
    type CoordinateMoveError: core::fmt::Debug + Clone + Default;

    /// Horizontal node count.
    fn horizontal(&self) -> usize;
    /// Vertical node count.
    fn vertical(&self) -> usize;
    fn node_count(&self) -> usize {
        self.horizontal() * self.vertical()
    }
    /// Convert coordinate to Offset.
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError>;
    /// Convert coordinate to Offset without a check.
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        self.to_offset(coord)
            .unwrap_or_else(|_| crate::unreachable_debug_checked())
    }
    /// Convert coordinate from Offset.
    fn from_offset(&self, offset: Offset) -> Self::Coordinate;
    fn from_index(&self, index: usize) -> Self::Coordinate {
        let v = index / self.horizontal();
        let h = index % self.horizontal();
        self.from_offset(Offset(h, v))
    }
    fn to_index(&self, coord: Self::Coordinate) -> Option<usize> {
        let offset = self.to_offset(coord);
        offset.map(|o| o.1 * self.horizontal() + o.0).ok()
    }
    fn index_to_offset(&self, index: usize) -> Offset {
        let v = index / self.horizontal();
        let h = index % self.horizontal();
        Offset(h, v)
    }
    fn offset_to_index(&self, o: Offset) -> usize {
        o.1 * self.horizontal() + o.0
    }
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
    /// Number of axis.
    const COUNT: usize;
    /// Whether it is Directed or not.
    const DIRECTED: bool;
    /// If this axis is not directed, it is doubled, otherwise same as `COUNT`.
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    /// For tricks to optimize for undirected graph, and not to regress performance of directed graph.
    type Direction: AxisDirection;
    fn to_index(&self) -> usize;
    unsafe fn from_index_unchecked(index: usize) -> Self {
        Self::from_index(index).unwrap_or_else(|| unreachable_debug_checked())
    }
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction<T> {
    Foward(T),
    Backward(T),
}

impl<T: Axis> AxisDirection for Direction<T> {
    fn is_forward(&self) -> bool {
        match self {
            Direction::Foward(_) => true,
            Direction::Backward(_) => false,
        }
    }

    fn to_index(&self) -> usize {
        match self {
            Direction::Foward(x) => x.to_index(),
            Direction::Backward(x) => x.to_index() + T::COUNT,
        }
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        if index < T::COUNT {
            Direction::Foward(T::from_index_unchecked(index))
        } else {
            Direction::Backward(T::from_index_unchecked(index - T::COUNT))
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        if index < T::COUNT {
            Some(unsafe { Direction::Foward(T::from_index_unchecked(index)) })
        } else {
            T::from_index(index - T::COUNT).map(|x| Direction::Backward(x))
        }
    }
}

pub trait Coordinate: Copy + PartialEq {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset(pub(crate) usize, pub(crate) usize);

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
