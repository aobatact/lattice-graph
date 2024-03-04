//! Square graph using Abstract Lattice Graph. If you want a undirected square graph, it is recommended to use specialized [`SquareGraph`](`crate::SquareGraph`).
//! Use this for directed graph or square with diagonal direction graph.

use super::*;
use petgraph::{Directed, Undirected};

/// Undirected Square Graph based on [`LatticeGraph`], recommended to use [`SquareGraph`](`crate::SquareGraph`) instead.
pub type SquareGraphAbstract<N, E> = LatticeGraph<N, E, SquareShape>;
/// Directed Square Graph based on [`LatticeGraph`].
pub type DirectedSquareGraph<N, E> = LatticeGraph<N, E, SquareShape<Directed>>;
/// Undirected Square Graph with edge to diagonal direction.
pub type DiagonalSquareGraph<N, E> = LatticeGraph<N, E, SquareDiagonalShape>;
/// Directed Square Graph with edge to diagonal direction.
pub type DirectedDiagonalSquareGraph<N, E> = LatticeGraph<N, E, SquareDiagonalShape<Directed>>;

/// Axis for square graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SquareAxis {
    X = 0,
    Y = 1,
}

impl Axis for SquareAxis {
    const COUNT: usize = 2;
    const DIRECTED: bool = false;
    type Direction = DirectedSquareAxis;
    const UNDIRECTED_COUNT: usize = if Self::DIRECTED {
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
        unsafe { DirectedSquareAxis::from_index_unchecked(self.to_index()) }
    }

    fn backward(self) -> Self::Direction {
        unsafe { DirectedSquareAxis::from_index_unchecked(self.to_index() + 2) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        let i = dir.dir_to_index();
        unsafe { Self::from_index_unchecked(if i >= 2 { i - 2 } else { i }) }
    }
}

/// Offset for square lattice graph.
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
/// Shape for Square Graph.
pub struct SquareShape<E = Undirected> {
    h: usize,
    v: usize,
    e: PhantomData<E>,
}

impl<E> SquareShape<E> {
    /// Create a new graph.
    pub fn new(h: usize, v: usize) -> Self {
        Self {
            h,
            v,
            e: PhantomData,
        }
    }
}

fn range_check<S: Shape>(s: S, coord: SquareOffset) -> Result<Offset, ()> {
    if coord.0.horizontal < s.horizontal() && coord.0.vertical < s.vertical() {
        Ok(coord.0)
    } else {
        Err(())
    }
}

fn move_coord<S: Shape>(
    s: S,
    coord: SquareOffset,
    dir: DirectedSquareAxis,
) -> Result<SquareOffset, ()> {
    let o = match dir {
        DirectedSquareAxis::X => coord.0.add_x(1).check_x(s.horizontal()),
        DirectedSquareAxis::Y => coord.0.add_y(1).check_y(s.vertical()),
        DirectedSquareAxis::RX => coord.0.sub_x(1),
        DirectedSquareAxis::RY => coord.0.sub_y(1),
    };
    o.map(|s| SquareOffset(s)).ok_or_else(|| ())
}

impl Shape for SquareShape {
    type Axis = SquareAxis;
    type Coordinate = SquareOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    #[inline]
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, ()> {
        range_check(self, coord)
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

    fn move_coord(&self, coord: SquareOffset, dir: DirectedSquareAxis) -> Result<SquareOffset, ()> {
        move_coord(self, coord, dir)
    }
}

/// Axis for directed square graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectedSquareAxis {
    X = 0,
    Y = 1,
    RX = 2,
    RY = 3,
}

impl Axis for DirectedSquareAxis {
    const COUNT: usize = 4;
    const DIRECTED: bool = true;
    type Direction = Self;

    fn to_index(&self) -> usize {
        match self {
            DirectedSquareAxis::X => 0,
            DirectedSquareAxis::Y => 1,
            DirectedSquareAxis::RX => 2,
            DirectedSquareAxis::RY => 3,
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => DirectedSquareAxis::X,
            1 => DirectedSquareAxis::Y,
            2 => DirectedSquareAxis::RX,
            3 => DirectedSquareAxis::RY,
            _ => return None,
        })
    }

    fn foward(self) -> Self::Direction {
        self
    }

    fn backward(self) -> Self::Direction {
        let x = self.to_index();
        let x2 = if x > 2 { x - 2 } else { x + 2 };
        unsafe { Self::from_index_unchecked(x2) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        dir
    }
}

impl Shape for SquareShape<petgraph::Directed> {
    type Axis = DirectedSquareAxis;
    type Coordinate = SquareOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h
    }

    fn vertical(&self) -> usize {
        self.v
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        range_check(self, coord)
    }

    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        SquareOffset(offset)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: DirectedSquareAxis,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        move_coord(self, coord, dir)
    }
}

/// Axis for lattice graph with Square and Diagonal Edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SquareDiagonalAxis {
    N,
    NE,
    E,
    SE,
}

impl Axis for SquareDiagonalAxis {
    const COUNT: usize = 4;
    const DIRECTED: bool = false;
    const UNDIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction = DirectedSquareDiagonalAxis;

    fn to_index(&self) -> usize {
        match self {
            SquareDiagonalAxis::N => 0,
            SquareDiagonalAxis::NE => 1,
            SquareDiagonalAxis::E => 2,
            SquareDiagonalAxis::SE => 3,
        }
    }

    fn from_index(index: usize) -> Option<Self>
    where
        Self: Sized,
    {
        Some(match index {
            0 => SquareDiagonalAxis::N,
            1 => SquareDiagonalAxis::NE,
            2 => SquareDiagonalAxis::E,
            3 => SquareDiagonalAxis::SE,
            _ => return None,
        })
    }

    fn foward(self) -> Self::Direction {
        unsafe { DirectedSquareDiagonalAxis::from_index_unchecked(self.to_index()) }
    }

    fn backward(self) -> Self::Direction {
        unsafe { DirectedSquareDiagonalAxis::from_index_unchecked(self.to_index() + 4) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        unsafe {
            let i = dir.to_index();
            Self::from_index_unchecked(if i < 4 { i } else { i - 4 })
        }
    }
}

/// Axis for lattice graph with Square and Diagonal Edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DirectedSquareDiagonalAxis {
    N,
    NE,
    E,
    SE,
    S,
    SW,
    W,
    NW,
}

impl Axis for DirectedSquareDiagonalAxis {
    const COUNT: usize = 8;
    const DIRECTED: bool = true;
    const UNDIRECTED_COUNT: usize = if Self::DIRECTED {
        Self::COUNT
    } else {
        Self::COUNT * 2
    };
    type Direction = Self;

    fn to_index(&self) -> usize {
        match self {
            DirectedSquareDiagonalAxis::N => 0,
            DirectedSquareDiagonalAxis::NE => 1,
            DirectedSquareDiagonalAxis::E => 2,
            DirectedSquareDiagonalAxis::SE => 3,
            DirectedSquareDiagonalAxis::S => 4,
            DirectedSquareDiagonalAxis::SW => 5,
            DirectedSquareDiagonalAxis::W => 6,
            DirectedSquareDiagonalAxis::NW => 7,
        }
    }

    unsafe fn from_index_unchecked(index: usize) -> Self {
        match index {
            0 => DirectedSquareDiagonalAxis::N,
            1 => DirectedSquareDiagonalAxis::NE,
            2 => DirectedSquareDiagonalAxis::E,
            3 => DirectedSquareDiagonalAxis::SE,
            4 => DirectedSquareDiagonalAxis::S,
            5 => DirectedSquareDiagonalAxis::SW,
            6 => DirectedSquareDiagonalAxis::W,
            7 => DirectedSquareDiagonalAxis::NW,
            _ => core::hint::unreachable_unchecked(),
        }
    }

    fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => DirectedSquareDiagonalAxis::N,
            1 => DirectedSquareDiagonalAxis::NE,
            2 => DirectedSquareDiagonalAxis::E,
            3 => DirectedSquareDiagonalAxis::SE,
            4 => DirectedSquareDiagonalAxis::S,
            5 => DirectedSquareDiagonalAxis::SW,
            6 => DirectedSquareDiagonalAxis::W,
            7 => DirectedSquareDiagonalAxis::NW,
            _ => return None,
        })
    }

    fn foward(self) -> Self::Direction {
        self
    }

    fn backward(self) -> Self::Direction {
        let i = self.to_index();
        unsafe { Self::from_index_unchecked(if i < 4 { i + 4 } else { i - 4 }) }
    }

    fn from_direction(dir: Self::Direction) -> Self {
        dir
    }
}

/// Shape for lattice graph with Square and Diagonal Edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SquareDiagonalShape<E = Undirected> {
    h: usize,
    v: usize,
    e: PhantomData<E>,
}

impl<E> SquareDiagonalShape<E> {
    /// Create a new graph.
    pub fn new(h: usize, v: usize) -> Self {
        Self {
            h,
            v,
            e: PhantomData,
        }
    }
}

impl Shape for SquareDiagonalShape {
    type Axis = SquareDiagonalAxis;
    type Coordinate = SquareOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h
    }

    fn vertical(&self) -> usize {
        self.v
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        if coord.0.horizontal < self.horizontal() && coord.0.vertical < self.vertical() {
            Ok(coord.0)
        } else {
            Err(())
        }
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
    }

    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        SquareOffset(offset)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: DirectedSquareDiagonalAxis,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        let offset = coord.0;
        match dir {
            DirectedSquareDiagonalAxis::N => offset.add_y(1).check_y(self.vertical()),
            DirectedSquareDiagonalAxis::NE => offset
                .add_x(1)
                .check_x(self.horizontal())
                .map(|o| o.add_y(1).check_y(self.vertical()))
                .flatten(),
            DirectedSquareDiagonalAxis::E => offset.add_x(1).check_x(self.horizontal()),
            DirectedSquareDiagonalAxis::SE => offset
                .add_x(1)
                .check_x(self.horizontal())
                .map(|o| o.sub_y(1))
                .flatten(),

            DirectedSquareDiagonalAxis::S => offset.sub_y(1),
            DirectedSquareDiagonalAxis::SW => offset.sub_x(1).map(|o| o.sub_y(1)).flatten(),
            DirectedSquareDiagonalAxis::W => offset.sub_x(1),
            DirectedSquareDiagonalAxis::NW => offset
                .sub_x(1)
                .map(|o| o.add_y(1).check_y(self.vertical()))
                .flatten(),
        }
        .map(|x| SquareOffset(x))
        .ok_or(())
    }
}

impl Shape for SquareDiagonalShape<Directed> {
    type Axis = DirectedSquareDiagonalAxis;
    type Coordinate = SquareOffset;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h
    }

    fn vertical(&self) -> usize {
        self.v
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        if coord.0.horizontal < self.horizontal() && coord.0.vertical < self.vertical() {
            Ok(coord.0)
        } else {
            Err(())
        }
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        coord.0
    }

    fn from_offset(&self, offset: Offset) -> Self::Coordinate {
        SquareOffset(offset)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: DirectedSquareDiagonalAxis,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        let offset = coord.0;
        match dir {
            DirectedSquareDiagonalAxis::N => offset.add_y(1).check_y(self.vertical()),
            DirectedSquareDiagonalAxis::NE => offset
                .add_x(1)
                .check_x(self.horizontal())
                .map(|o| o.add_y(1).check_y(self.vertical()))
                .flatten(),
            DirectedSquareDiagonalAxis::E => offset.add_x(1).check_x(self.horizontal()),
            DirectedSquareDiagonalAxis::SE => offset
                .add_x(1)
                .check_x(self.horizontal())
                .map(|o| o.sub_y(1))
                .flatten(),

            DirectedSquareDiagonalAxis::S => offset.sub_y(1),
            DirectedSquareDiagonalAxis::SW => offset.sub_x(1).map(|o| o.sub_y(1)).flatten(),
            DirectedSquareDiagonalAxis::W => offset.sub_x(1),
            DirectedSquareDiagonalAxis::NW => offset
                .sub_x(1)
                .map(|o| o.add_y(1).check_y(self.vertical()))
                .flatten(),
        }
        .map(|x| SquareOffset(x))
        .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::visit::*;

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
             _d| (x + 2 * y) as i32,
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
        debug_assert!(v00.eq([(1, 0), (0, 1)]));

        let v04 = sq.neighbors((0, 4).into());
        debug_assert!(v04.eq([(1, 4), (0, 3)]));

        let v20 = sq.neighbors((2, 0).into());
        debug_assert!(v20.eq([(2, 1), (1, 0)]));

        let v24 = sq.neighbors((2, 4).into());
        debug_assert!(v24.eq([(1, 4), (2, 3)]));

        let v12 = sq.neighbors((1, 2).into());
        debug_assert!(v12.eq([(2, 2), (1, 3), (0, 2), (1, 1)]));
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
             _d| (x + 2 * y) as i32,
        );

        debug_assert!(sq
            .edges((0, 0).into())
            .map(|e| e.target())
            .eq([(1, 0), (0, 1)]));

        debug_assert!(sq.edges((0, 0).into()).map(|e| e.edge_weight).eq(&[0, 0]));
        debug_assert!(sq
            .edges((1, 1).into())
            .map(|e| e.edge_weight)
            .eq(&[3, 3, 2, 1]));

        debug_assert!(sq.edges((1, 2).into()).map(|e| e.target()).eq([
            (2, 2),
            (1, 3),
            (0, 2),
            (1, 1)
        ]));
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
             d| { (x + 2 * y) as i32 * if d == SquareAxis::X { 1 } else { 3 } },
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
