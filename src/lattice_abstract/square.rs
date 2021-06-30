//! Just for testing Abstract Lattice Graph. Recommend to use specialized [`SquareGraph`](`crate::SquareGraph`).

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

    fn foward(self) -> Self::Direction {
        Direction::Foward(self.clone())
    }

    fn backward(self) -> Self::Direction {
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
pub struct SquareOffset(pub Offset);

impl PartialEq<(usize, usize)> for SquareOffset {
    fn eq(&self, other: &(usize, usize)) -> bool {
        self.0.horizontal == other.0 && self.0.vertical == other.1
    }
}

impl From<(usize, usize)> for SquareOffset {
    fn from(x: (usize, usize)) -> Self {
        SquareOffset(Offset {
            horizontal: x.0,
            vertical: x.1,
        })
    }
}

impl Coordinate for SquareOffset {}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]

pub struct SquareShape {
    h: usize,
    v: usize,
}

impl SquareShape {
    pub fn new(h: usize, v: usize) -> Self {
        Self { h, v }
    }
}

impl Shape for SquareShape {
    type Axis = SquareAxis;
    type Coordinate = SquareOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    #[inline]
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, ()> {
        if coord.0.horizontal < self.horizontal() && coord.0.vertical < self.vertical() {
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

#[cfg(test)]
mod tests {
    use std::array::IntoIter;

    use petgraph::visit::*;

    use super::*;

    type SquareGraph<N, E> = super::SquareGraphAbstract<N, E>;

    #[test]
    fn gen() {
        let sq = SquareGraph::new_with(
            SquareShape::new(4, 3),
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             })| x + 2 * y,
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             }),
             _d| Some((x + 2 * y) as i32),
        );
        assert_eq!(sq.s.horizontal(), 4);
        assert_eq!(sq.s.vertical(), 3);
        assert_eq!(sq.node_weight((0, 0).into()), Some(&0));
        assert_eq!(sq.node_weight((0, 1).into()), Some(&2));
        assert_eq!(sq.node_weight((1, 0).into()), Some(&1));
        assert_eq!(sq.node_weight((2, 0).into()), Some(&2));
        assert_eq!(sq.node_weight((3, 0).into()), Some(&3));
        assert_eq!(sq.node_weight((4, 0).into()), None);
        assert_eq!(sq.node_weight((0, 2).into()), Some(&4));
        assert_eq!(sq.node_weight((0, 3).into()), None);
        assert_eq!(
            sq.edge_weight(((0, 0).into(), SquareAxis::X).into()),
            Some(&0)
        );
        assert_eq!(
            sq.edge_weight(((0, 2).into(), SquareAxis::X).into()),
            Some(&4)
        );
        assert_eq!(sq.edge_weight(((0, 2).into(), SquareAxis::Y).into()), None);
        assert_eq!(sq.edge_weight(((3, 0).into(), SquareAxis::X).into()), None);
        assert_eq!(
            sq.edge_weight(((3, 0).into(), SquareAxis::Y).into()),
            Some(&3)
        );
    }

    #[test]
    fn node_identifiers() {
        let sq = SquareGraph::new_with(
            SquareShape::new(4, 3),
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             })| x + 2 * y,
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             }),
             _d| Some((x + 2 * y) as i32),
        );
        let mut count = 0;
        for (i, x) in sq.node_identifiers().enumerate() {
            let x = x;
            let x2 = sq.to_index(x);
            assert_eq!(x2, i);
            let x3 = sq.from_index(x2);
            assert_eq!(x, x3);
            count += 1;
        }
        assert_eq!(count, 12);
    }

    #[test]
    fn neighbors() {
        let sq = SquareGraph::new_with(
            SquareShape::new(3, 5),
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             })| x + 2 * y,
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             }),
             _d| Some((x + 2 * y) as i32),
        );

        let v00 = sq.neighbors((0, 0).into());
        debug_assert!(v00.eq(IntoIter::new([(1, 0), (0, 1)])));

        let v04 = sq.neighbors((0, 4).into());
        debug_assert!(v04.eq(IntoIter::new([(1, 4), (0, 3)])));

        let v20 = sq.neighbors((2, 0).into());
        debug_assert!(v20.eq(IntoIter::new([(2, 1), (1, 0)])));

        let v24 = sq.neighbors((2, 4).into());
        debug_assert!(v24.eq(IntoIter::new([(1, 4), (2, 3)])));

        let v12 = sq.neighbors((1, 2).into());
        debug_assert!(v12.eq(IntoIter::new([(2, 2), (1, 3), (0, 2), (1, 1)])));
    }

    #[test]
    fn edges() {
        let sq = SquareGraph::new_with(
            SquareShape::new(3, 5),
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             })| x + 2 * y,
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             }),
             _d| Some((x + 2 * y) as i32),
        );

        debug_assert!(sq
            .edges((0, 0).into())
            .map(|e| e.target())
            .eq(IntoIter::new([(1, 0), (0, 1)])));

        debug_assert!(sq.edges((0, 0).into()).map(|e| e.edge_weight).eq(&[0, 0]));
        debug_assert!(sq
            .edges((1, 1).into())
            .map(|e| e.edge_weight)
            .eq(&[3, 3, 2, 1]));

        debug_assert!(sq
            .edges((1, 2).into())
            .map(|e| e.target())
            .eq(IntoIter::new([(2, 2), (1, 3), (0, 2), (1, 1)])));
    }

    #[test]
    fn astar() {
        let sq = SquareGraph::new_with(
            SquareShape::new(4, 3),
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             })| x + 2 * y,
            |SquareOffset(Offset {
                 horizontal: x,
                 vertical: y,
             }),
             d| { Some((x + 2 * y) as i32 * if d == SquareAxis::X { 1 } else { 3 }) },
        );

        let x = petgraph::algo::astar(
            &sq,
            (0, 0).into(),
            |x| x == (2, 1),
            |e| *e.weight(),
            |x| (x.0.horizontal as i32 - 2).abs() + (x.0.vertical as i32 - 1).abs(),
        );
        assert!(x.is_some());
        let (d, p) = x.unwrap();
        assert_eq!(d, 5);
        assert_eq!(p, [(0, 0), (0, 1), (1, 1), (2, 1)]);

        let x = petgraph::algo::astar(
            &sq,
            (2, 1).into(),
            |x| x == (0, 0),
            |e| *e.weight(),
            |x| (x.0.horizontal as i32 - 0).abs() + (x.0.vertical as i32 - 0).abs(),
        );
        assert!(x.is_some());
        let (d, p) = x.unwrap();
        assert_eq!(d, 5);
        assert_eq!(p, [(2, 1), (1, 1), (0, 1), (0, 0)])
    }
}
