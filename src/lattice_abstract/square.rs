use super::*;

pub type SquareGraphAbstract<N, E> = LatticeGraph<N, E, SquareShape>;

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

    #[allow(unused_unsafe)]
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
