use crate::{lattice_abstract::*, unreachable_debug_checked};

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
    type Direction = AxisDR;

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

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => AxisR::NE,
            1 => AxisR::E,
            2 => AxisR::SE,
            _ => unreachable_debug_checked(),
        }
    }

    fn foward(self) -> Self::Direction {
        unsafe { AxisDR::from_index_unchecked(self.to_index()) }
    }

    fn backward(self) -> Self::Direction {
        unsafe { AxisDR::from_index_unchecked(self.to_index() + Self::COUNT) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        let i = dir.to_index();
        unsafe { Self::from_index_unchecked(if i >= Self::COUNT { i - Self::COUNT } else { i }) }
    }

    fn is_forward_direction(dir: &Self::Direction) -> bool {
        dir.dir_to_index() < Self::COUNT
    }
}

impl Direction<AxisR> {
    pub const NE: Self = Direction::Foward(AxisR::NE);
    pub const E: Self = Direction::Foward(AxisR::E);
    pub const SE: Self = Direction::Foward(AxisR::SE);
    pub const SW: Self = Direction::Backward(AxisR::NE);
    pub const W: Self = Direction::Backward(AxisR::E);
    pub const NW: Self = Direction::Backward(AxisR::SE);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisDR {
    NE = 0,
    E = 1,
    SE = 2,
    SW = 3,
    W = 4,
    NW = 5,
}

impl Axis for AxisDR {
    const COUNT: usize = 6;
    const DIRECTED: bool = true;
    type Direction = Self;

    fn to_index(&self) -> usize {
        match self {
            AxisDR::NE => 0,
            AxisDR::E => 1,
            AxisDR::SE => 2,
            AxisDR::SW => 3,
            AxisDR::W => 4,
            AxisDR::NW => 5,
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match index {
            0 => AxisDR::NE,
            1 => AxisDR::E,
            2 => AxisDR::SE,
            3 => AxisDR::SW,
            4 => AxisDR::W,
            5 => AxisDR::NW,
            _ => return None,
        })
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => AxisDR::NE,
            1 => AxisDR::E,
            2 => AxisDR::SE,
            3 => AxisDR::SW,
            4 => AxisDR::W,
            5 => AxisDR::NW,
            _ => unreachable_debug_checked(),
        }
    }

    fn foward(self) -> Self::Direction {
        self
    }

    fn backward(self) -> Self::Direction {
        match self {
            AxisDR::NE => AxisDR::SW,
            AxisDR::E => AxisDR::W,
            AxisDR::SE => AxisDR::NW,
            AxisDR::SW => AxisDR::NE,
            AxisDR::W => AxisDR::E,
            AxisDR::NW => AxisDR::SE,
        }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        dir
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
    type Direction = AxisDQ;

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

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => AxisQ::N,
            1 => AxisQ::NE,
            2 => AxisQ::SE,
            _ => unreachable_debug_checked(),
        }
    }

    fn foward(self) -> Self::Direction {
        unsafe { AxisDQ::from_index_unchecked(self.to_index()) }
    }

    fn backward(self) -> Self::Direction {
        unsafe { AxisDQ::from_index_unchecked(self.to_index() + 3) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        let i = dir.dir_to_index();
        unsafe { Self::from_index_unchecked(if i < Self::COUNT { i } else { i - Self::COUNT }) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AxisDQ {
    N = 0,
    NE = 1,
    SE = 2,
    S = 3,
    SW = 4,
    NW = 5,
}

impl Axis for AxisDQ {
    const COUNT: usize = 6;
    const DIRECTED: bool = true;
    type Direction = Self;

    fn to_index(&self) -> usize {
        match self {
            AxisDQ::N => 0,
            AxisDQ::NE => 1,
            AxisDQ::SE => 2,
            AxisDQ::S => 3,
            AxisDQ::SW => 4,
            AxisDQ::NW => 5,
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match index {
            0 => AxisDQ::N,
            1 => AxisDQ::NE,
            2 => AxisDQ::SE,
            3 => AxisDQ::S,
            4 => AxisDQ::SW,
            5 => AxisDQ::NW,
            _ => return None,
        })
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => AxisDQ::N,
            1 => AxisDQ::NE,
            2 => AxisDQ::SE,
            3 => AxisDQ::S,
            4 => AxisDQ::SW,
            5 => AxisDQ::NW,
            _ => unreachable_debug_checked(),
        }
    }

    fn foward(self) -> Self::Direction {
        self
    }

    fn backward(self) -> Self::Direction {
        match self {
            AxisDQ::N => AxisDQ::S,
            AxisDQ::NE => AxisDQ::SW,
            AxisDQ::SE => AxisDQ::NW,
            AxisDQ::S => AxisDQ::N,
            AxisDQ::SW => AxisDQ::NE,
            AxisDQ::NW => AxisDQ::NE,
        }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        dir
    }
}

pub trait OE {
    const IS_EVEN: bool;
    const CONVERT_OFFSET: usize = if Self::IS_EVEN { 1 } else { 0 };
}
pub trait RQ {
    const IS_FLAT_TOP: bool;
}

/// Point-top + Odd Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddR {}
impl OE for OddR {
    const IS_EVEN: bool = false;
}
impl RQ for OddR {
    const IS_FLAT_TOP: bool = false;
}

/// Point-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenR {}
impl OE for EvenR {
    const IS_EVEN: bool = true;
}
impl RQ for EvenR {
    const IS_FLAT_TOP: bool = false;
}

/// Odd-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddQ {}
impl OE for OddQ {
    const IS_EVEN: bool = false;
}
impl RQ for OddQ {
    const IS_FLAT_TOP: bool = false;
}

/// Flat-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenQ {}
impl OE for EvenQ {
    const IS_EVEN: bool = true;
}
impl RQ for EvenQ {
    const IS_FLAT_TOP: bool = false;
}

pub trait LoopMarker {}
impl LoopMarker for () {}

///Marker for E-W direction Loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LEW {}
impl LoopMarker for LEW {}
