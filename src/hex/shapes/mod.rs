use crate::lattice_abstract::*;

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

impl Direction<AxisQ> {
    pub const N: Self = Direction::Foward(AxisQ::N);
    pub const NE: Self = Direction::Foward(AxisQ::NE);
    pub const SE: Self = Direction::Foward(AxisQ::SE);
    pub const S: Self = Direction::Backward(AxisQ::N);
    pub const SW: Self = Direction::Backward(AxisQ::NE);
    pub const NW: Self = Direction::Backward(AxisQ::SE);
}

/// Point-top + Odd Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddR {}

/// Point-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenR {}

/// Odd-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OddQ {}

/// Flat-top + Even Shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvenQ {}

pub trait LoopMarker {}
impl LoopMarker for () {}

///Marker for E-W direction Loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LEW {}
impl LoopMarker for LEW {}
