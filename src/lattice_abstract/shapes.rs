//! Shapes to define the behavior of the [`LatticeGraph`](`crate::lattice_abstract::LatticeGraph`)
//!
//! If you want to create your own lattice based graph, use this to implement your own lattice.
//!

use crate::unreachable_debug_checked;

/// Shape of the 2d lattice.
/// It decides the behavior of the coordinate.
pub trait Shape {
    /// Axis of the lattice.
    type Axis: Axis;
    /// Coordinate of the lattice graph.
    type Coordinate: Coordinate;
    /// Error to return when [`to_offset`](`Shape::to_offset`) fails.
    /// Should set [`Infallible`](`core::convert::Infallible`) when the graph is looped and never to fail.
    type OffsetConvertError: core::fmt::Debug + Clone;
    /// Error to return when [`move_coord`](`Shape::move_coord`) fails.
    /// Should set [`Infallible`](`core::convert::Infallible`) when the graph is looped and never to fail.
    type CoordinateMoveError: core::fmt::Debug + Clone;

    /// Horizontal node count.
    fn horizontal(&self) -> usize;
    /// Vertical node count.
    fn vertical(&self) -> usize;
    /// Node count.
    fn node_count(&self) -> usize {
        self.horizontal() * self.vertical()
    }
    /// Convert coordinate to `Offset`.
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError>;
    /// Convert coordinate to Offset without a check.
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        self.to_offset(coord)
            .unwrap_or_else(|_| crate::unreachable_debug_checked())
    }
    /// Convert coordinate from `Offset`.
    fn from_offset(&self, offset: Offset) -> Self::Coordinate;

    /// Convert coordinate from index.
    fn from_index(&self, index: usize) -> Self::Coordinate {
        self.from_offset(self.index_to_offset(index))
    }
    /// Covert coordinate to index.
    fn to_index(&self, coord: Self::Coordinate) -> Option<usize> {
        let offset = self.to_offset(coord);
        offset.map(|o| self.offset_to_index(o)).ok()
    }
    /// Convert index to offset.
    fn index_to_offset(&self, index: usize) -> Offset {
        let v = index % self.vertical();
        let h = index / self.vertical();
        Offset::new(h, v)
    }
    /// Covert offset to index.
    fn offset_to_index(&self, o: Offset) -> usize {
        o.horizontal * self.vertical() + o.vertical
    }

    /// Edge count of horizontal. May differ by the axis info.
    fn horizontal_edge_size(&self, _axis: Self::Axis) -> usize {
        self.horizontal()
    }
    /// Edge count of vertical. May differ by the axis info.
    fn vertical_edge_size(&self, _axis: Self::Axis) -> usize {
        self.vertical()
    }
    /// Move coordinate to direction.
    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError>;
    /// Move coordinate to direction.
    unsafe fn move_coord_unchecked(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Self::Coordinate {
        self.move_coord(coord, dir)
            .unwrap_or_else(|_| unreachable_debug_checked())
    }
    ///Check whether coordinate is in neighbor.
    fn is_neighbor(&self, a: Self::Coordinate, b: Self::Coordinate) -> bool
    where
        Self::Coordinate: PartialEq,
    {
        self.get_direction(a, b).is_some()
    }
    ///Get direction if two coordiante is neighbor.
    fn get_direction(
        &self,
        source: Self::Coordinate,
        target: Self::Coordinate,
    ) -> Option<<Self::Axis as Axis>::Direction>
    where
        Self::Coordinate: PartialEq,
    {
        let a = source;
        let b = target;
        for i in 0..<Self::Axis as Axis>::DIRECTED_COUNT {
            let d = unsafe { <Self::Axis as Axis>::Direction::dir_from_index_unchecked(i) };
            let c = self.move_coord(a, d.clone());
            if let Ok(c) = c {
                if c == b {
                    return Some(d);
                }
            }
        }
        None
    }
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

    unsafe fn move_coord_unchecked(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Self::Coordinate {
        (*self).move_coord_unchecked(coord, dir)
    }

    fn node_count(&self) -> usize {
        (*self).node_count()
    }

    fn from_index(&self, index: usize) -> Self::Coordinate {
        (*self).from_index(index)
    }

    fn to_index(&self, coord: Self::Coordinate) -> Option<usize> {
        (*self).to_index(coord)
    }

    fn index_to_offset(&self, index: usize) -> Offset {
        (*self).index_to_offset(index)
    }

    fn offset_to_index(&self, o: Offset) -> usize {
        (*self).offset_to_index(o)
    }

    fn is_neighbor(&self, a: Self::Coordinate, b: Self::Coordinate) -> bool
    where
        Self::Coordinate: PartialEq,
    {
        (*self).is_neighbor(a, b)
    }

    fn get_direction(
        &self,
        source: Self::Coordinate,
        target: Self::Coordinate,
    ) -> Option<<Self::Axis as Axis>::Direction>
    where
        Self::Coordinate: PartialEq,
    {
        (*self).get_direction(source, target)
    }
}

/// Axis of the graph. It holds what direction of edge which node has.
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
    /// If the axis is `DIRECTED`, should set `Self`.
    type Direction: AxisDirection;
    /// Convert to index.
    fn to_index(&self) -> usize;
    /// Convert form index.
    unsafe fn from_index_unchecked(index: usize) -> Self {
        Self::from_index(index).unwrap_or_else(|| unreachable_debug_checked())
    }
    /// Convert form index.
    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized;
    /// To forward direction. It is nop when Axis is `DIRECTED`.
    fn foward(self) -> Self::Direction;
    /// To backward direction. It reverses when Axis is `DIRECTED`.
    fn backward(self) -> Self::Direction;
    /// Check the direction is forward for this axis.
    /// Returns true if the direction is `DIRECTED` is `true`, or the index of the axis and direction matches.
    fn is_forward_direction(dir: &Self::Direction) -> bool {
        Self::DIRECTED || dir.dir_to_index() == Self::from_direction(dir.clone()).to_index()
    }
    /// Convert from direction.
    fn from_direction(dir: Self::Direction) -> Self;
}

/// Direction of axis. It tells which direction is connected to node.
pub trait AxisDirection: Clone {
    /// Check this match whith [`Axis`]. It will always return true when `Axis` is directed.
    #[deprecated(note = "Use Axis::is_forward_direction instead.")]
    fn is_forward(&self) -> bool;
    /// Check this doesn't match whith [`Axis`]. It will always return false when `Axis` is directed.
    #[deprecated(note = "Use !Axis::is_forward_direction instead.")]
    #[allow(deprecated)]
    fn is_backward(&self) -> bool {
        !self.is_forward()
    }
    /// Convert to index.
    fn dir_to_index(&self) -> usize;
    /// Convert from index.
    unsafe fn dir_from_index_unchecked(index: usize) -> Self;
    /// Convert from index.
    fn dir_from_index(index: usize) -> Option<Self>
    where
        Self: Sized;
}

/// Implimention for Axis of directed graph.
impl<A> AxisDirection for A
where
    A: Axis<Direction = Self>,
{
    fn is_forward(&self) -> bool {
        true
    }
    fn dir_to_index(&self) -> usize {
        <Self as Axis>::to_index(self)
    }
    unsafe fn dir_from_index_unchecked(index: usize) -> Self {
        <Self as Axis>::from_index_unchecked(index)
    }
    fn dir_from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        <Self as Axis>::from_index(index)
    }
}

/// Default Implimention of [`AxisDirection`] when [`Axis::DIRECTED`] is false.
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

    fn dir_to_index(&self) -> usize {
        match self {
            Direction::Foward(x) => x.to_index(),
            Direction::Backward(x) => x.to_index() + T::COUNT,
        }
    }

    unsafe fn dir_from_index_unchecked(index: usize) -> Self {
        if index < T::COUNT {
            Direction::Foward(T::from_index_unchecked(index))
        } else {
            Direction::Backward(T::from_index_unchecked(index - T::COUNT))
        }
    }

    fn dir_from_index(index: usize) -> Option<Self>
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

/// Representention of where is the node in graph.
pub trait Coordinate: Copy + PartialEq {}

/// Actual postion in the stroage.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Offset {
    pub(crate) horizontal: usize,
    pub(crate) vertical: usize,
}

impl Offset {
    pub fn new(h: usize, v: usize) -> Self {
        Offset {
            horizontal: h.into(),
            vertical: v.into(),
        }
    }
    pub fn horizontal(&self) -> usize {
        self.horizontal
    }
    pub fn vertical(&self) -> usize {
        self.vertical
    }
    pub fn add_x(&self, x: usize) -> Self {
        Offset::new(self.horizontal + x, self.vertical)
    }
    pub fn add_y(&self, y: usize) -> Self {
        Offset::new(self.horizontal, self.vertical + y)
    }
    pub fn set_x(&self, x: usize) -> Self {
        Offset::new(x, self.vertical)
    }
    pub fn set_y(&self, y: usize) -> Self {
        Offset::new(self.horizontal, y)
    }
    pub fn sub_x(&self, x: usize) -> Option<Self> {
        Some(Offset::new(self.horizontal.checked_sub(x)?, self.vertical))
    }
    pub fn sub_y(&self, y: usize) -> Option<Self> {
        Some(Offset::new(self.horizontal, self.vertical.checked_sub(y)?))
    }
    pub unsafe fn sub_x_unchecked(&self, x: usize) -> Self {
        Offset::new(self.horizontal - x, self.vertical)
    }
    pub unsafe fn sub_y_unchecked(&self, y: usize) -> Self {
        Offset::new(self.horizontal, self.vertical - y)
    }
    pub fn check_x(&self, x_max: usize) -> Option<Self> {
        if self.horizontal < x_max {
            Some(*self)
        } else {
            None
        }
    }
    pub fn check_y(&self, y_max: usize) -> Option<Self> {
        if self.vertical < y_max {
            Some(*self)
        } else {
            None
        }
    }
}

impl<T: Into<usize>> From<(T, T)> for Offset {
    fn from(offset: (T, T)) -> Self {
        Offset {
            horizontal: offset.0.into(),
            vertical: offset.1.into(),
        }
    }
}

impl<T: From<usize>> From<Offset> for (T, T) {
    fn from(x: Offset) -> Self {
        (x.horizontal.into(), x.vertical.into())
    }
}
