#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::*;
use std::marker::{Copy, PhantomData};

use crate::lattice_abstract::shapes::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HexOffset(Offset);

impl HexOffset {
    pub fn new(x: usize, y: usize) -> Self {
        Self(Offset {
            horizontal: x,
            vertical: y,
        })
    }
}

impl Coordinate for HexOffset {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Point top Hex Direcion.
pub enum AxisR {
    NE = 0,
    E = 1,
    SE = 2,
}

impl Axis for AxisR {
    const COUNT: usize = 3;
    const DIRECTED: bool = false;
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction = Direction<Self>;

    fn to_index(&self) -> usize {
        match self {
            AxisR::NE => 0,
            AxisR::E => 1,
            AxisR::SE => 2,
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match index {
            0 => AxisR::NE,
            1 => AxisR::E,
            2 => AxisR::SE,
            _ => return None,
        })
    }

    fn foward(self) -> Self::Direction {
        Direction::Foward(self)
    }

    fn backward(self) -> Self::Direction {
        Direction::Backward(self)
    }

    fn from_direction(dir: Self::Direction) -> Self {
        match dir {
            Direction::Foward(a) | Direction::Backward(a) => a,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Flat top Hex Direction.
pub enum AxisQ {
    N = 0,
    NE = 1,
    SE = 2,
}

impl Axis for AxisQ {
    const COUNT: usize = 3;
    const DIRECTED: bool = false;
    const DIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction = Direction<Self>;

    fn to_index(&self) -> usize {
        match self {
            AxisQ::N => 0,
            AxisQ::NE => 1,
            AxisQ::SE => 2,
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match index {
            0 => AxisQ::N,
            1 => AxisQ::NE,
            2 => AxisQ::SE,
            _ => return None,
        })
    }

    fn foward(self) -> Self::Direction {
        Direction::Foward(self)
    }

    fn backward(self) -> Self::Direction {
        Direction::Backward(self)
    }

    fn from_direction(dir: Self::Direction) -> Self {
        match dir {
            Direction::Foward(a) | Direction::Backward(a) => a,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexOffsetShape<T, H = usize, V = usize>
where
    H: Into<usize> + Copy,
    V: Into<usize> + Copy,
{
    h: H,
    v: V,
    t: PhantomData<fn() -> T>,
}

impl<T, H, V> HexOffsetShape<T, H, V>
where
    H: Into<usize> + Copy,
    V: Into<usize> + Copy,
{
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            t: PhantomData,
        }
    }
}

#[cfg(feature = "const-generic-wrap")]
pub type ConstHexOffsetShape<T, const H: usize, const V: usize> =
    HexOffsetShape<T, WrapUSIZE<H>, WrapUSIZE<V>>;

#[cfg(feature = "const-generic-wrap")]
impl<T, const H: usize, const V: usize> Default for ConstHexOffsetShape<T, H, V> {
    fn default() -> Self {
        Self::new(WrapUSIZE::<H>, WrapUSIZE::<V>)
    }
}

pub trait HexOffsetShapeBase {
    type Axis: Axis;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize;
    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize;
    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Direction<Self::Axis>,
    ) -> Result<HexOffset, ()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddR {}
impl HexOffsetShapeBase for OddR {
    type Axis = AxisR;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => horizontal,
            _ => horizontal - 1,
        }
    }

    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => vertical - 1,
            _ => vertical,
        }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Direction<AxisR>,
    ) -> Result<HexOffset, ()> {
        move_coord_r(horizontal, vertical, coord, dir, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenR {}

impl HexOffsetShapeBase for EvenR {
    type Axis = AxisR;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => horizontal,
            _ => horizontal - 1,
        }
    }

    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => vertical - 1,
            _ => vertical,
        }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Direction<AxisR>,
    ) -> Result<HexOffset, ()> {
        move_coord_r(horizontal, vertical, coord, dir, 1)
    }
}

fn move_coord_r(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: Direction<AxisR>,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 == flag) {
        (Direction::Foward(AxisR::E), _) => o.add_x(1).check_x(horizontal),
        (Direction::Backward(AxisR::E), _) => o.sub_x(1),
        (Direction::Foward(AxisR::NE), true) | (Direction::Backward(AxisR::SE), false) => {
            o.add_y(1).check_y(vertical)
        }
        (Direction::Foward(AxisR::SE), true) | (Direction::Backward(AxisR::NE), false) => {
            o.sub_y(1)
        }
        (Direction::Foward(AxisR::NE), false) => o.add_x(1).add_y(1).check_y(vertical),
        (Direction::Foward(AxisR::SE), false) => {
            o.add_x(1).sub_y(1).map(|o| o.check_x(horizontal)).flatten()
        }
        (Direction::Backward(AxisR::NE), true) => o.sub_x(1).map(|o| o.sub_y(1)).flatten(),
        (Direction::Backward(AxisR::SE), true) => {
            o.sub_x(1).map(|o| o.add_y(1).check_y(vertical)).flatten()
        }
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddQ {}
impl HexOffsetShapeBase for OddQ {
    type Axis = AxisQ;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisQ::N => horizontal,
            _ => horizontal - 1,
        }
    }

    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisQ::N => vertical - 1,
            _ => vertical,
        }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Direction<AxisQ>,
    ) -> Result<HexOffset, ()> {
        move_coord_q(horizontal, vertical, coord, dir, 0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenQ {}
impl HexOffsetShapeBase for EvenQ {
    type Axis = AxisQ;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisQ::N => horizontal,
            _ => horizontal - 1,
        }
    }

    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize {
        match axis {
            AxisQ::N => vertical - 1,
            _ => vertical,
        }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Direction<AxisQ>,
    ) -> Result<HexOffset, ()> {
        move_coord_q(horizontal, vertical, coord, dir, 1)
    }
}

fn move_coord_q(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: Direction<AxisQ>,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 != flag) {
        (Direction::Foward(AxisQ::N), _) => o.add_y(1).check_y(vertical),
        (Direction::Backward(AxisQ::N), _) => o.sub_y(1),
        (Direction::Foward(AxisQ::NE), true) | (Direction::Backward(AxisQ::SE), false) => {
            o.add_x(1).check_x(horizontal)
        }
        (Direction::Foward(AxisQ::SE), true) | (Direction::Backward(AxisQ::NE), false) => {
            o.sub_x(1)
        }
        (Direction::Foward(AxisQ::NE), false) => o.add_y(1).add_x(1).check_x(horizontal),
        (Direction::Foward(AxisQ::SE), false) => {
            o.add_y(1).sub_x(1).map(|o| o.check_y(vertical)).flatten()
        }
        (Direction::Backward(AxisQ::NE), true) => o.sub_y(1).map(|o| o.sub_x(1)).flatten(),
        (Direction::Backward(AxisQ::SE), true) => {
            o.sub_y(1).map(|o| o.add_x(1).check_x(horizontal)).flatten()
        }
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

impl<B, H, V> Shape for HexOffsetShape<B, H, V>
where
    B: HexOffsetShapeBase,
    B::Axis: Axis<Direction = Direction<B::Axis>>,
    H: Into<usize> + Copy,
    V: Into<usize> + Copy,
{
    type Axis = B::Axis;
    type Coordinate = HexOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h.into()
    }

    fn vertical(&self) -> usize {
        self.v.into()
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        let offset = coord.0;
        if offset.horizontal() < self.horizontal() && offset.vertical() < self.vertical() {
            Ok(offset)
        } else {
            Err(())
        }
    }

    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        HexOffset(offset)
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
    }

    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize {
        B::horizontal_edge_size(self.horizontal(), axis)
    }

    fn vertical_edge_size(&self, axis: Self::Axis) -> usize {
        B::vertical_edge_size(self.vertical(), axis)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        B::move_coord(self.horizontal(), self.vertical(), coord, dir)
    }
}
