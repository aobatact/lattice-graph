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
    NE,
    SE,
    S,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddR {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenR {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddQ {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenQ {}

impl<H, V> Shape for HexOffsetShape<OddR, H, V>
where
    H: Into<usize> + Copy,
    V: Into<usize> + Copy,
{
    type Axis = AxisR;
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

    fn horizontal_edge_size(&self, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => self.horizontal(),
            _ => self.horizontal() - 1,
        }
    }

    fn vertical_edge_size(&self, axis: Self::Axis) -> usize {
        match axis {
            AxisR::SE => self.vertical() - 1,
            _ => self.vertical(),
        }
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: Direction<AxisR>,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        let o = coord.0;
        match (dir, o.vertical() & 1 == 0) {
            (Direction::Foward(AxisR::E), _) => o.add_x(1).check_x(self.horizontal()),
            (Direction::Backward(AxisR::E), _) => o.sub_x(1),
            (Direction::Foward(AxisR::NE), true) | (Direction::Backward(AxisR::SE), false) => {
                o.add_y(1).check_y(self.vertical())
            }
            (Direction::Foward(AxisR::SE), true) | (Direction::Backward(AxisR::NE), false) => {
                o.sub_y(1)
            }
            (Direction::Foward(AxisR::NE), false) => o.add_x(1).add_y(1).check_y(self.vertical()),
            (Direction::Foward(AxisR::SE), false) => o
                .add_x(1)
                .sub_y(1)
                .map(|o| o.check_x(self.horizontal()))
                .flatten(),
            (Direction::Backward(AxisR::NE), true) => o.sub_x(1).map(|o| o.sub_y(1)).flatten(),
            (Direction::Backward(AxisR::SE), true) => o
                .sub_x(1)
                .map(|o| o.add_y(1).check_y(self.vertical()))
                .flatten(),
        }
        .map(|o| HexOffset(o))
        .ok_or(())
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
    }
}
