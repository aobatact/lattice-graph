#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::*;
use std::marker::{Copy, PhantomData};

use crate::{hex::shapes::*, lattice_abstract::shapes::*};

/// Offset based coordinates for hex graph.
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
/// Defines wheter the hex graph is `flat-top` or `point-top` and is odd or even.
pub trait HexOffsetShapeBase {
    type Axis: Axis;
    fn horizontal_edge_size(horizontal: usize, axis: Self::Axis) -> usize;
    fn vertical_edge_size(vertical: usize, axis: Self::Axis) -> usize;
    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<HexOffset, ()>;
}

/// Defines wheter the hex graph, which is looped in E-W Direction, is `flat-top` or `point-top` and is odd or even.
pub trait HexOffsetShapeBaseLEW: HexOffsetShapeBase {
    fn move_coord_lew(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<HexOffset, ()>;
}

impl HexOffsetShapeBase for OddR {
    type Axis = AxisR;
    // TODO : give Shape Data
    fn horizontal_edge_size(horizontal: usize, _axis: Self::Axis) -> usize {
        horizontal
        // match axis {
        //     AxisR::SE => horizontal,
        //     _ => horizontal - 1,
        // }
    }

    fn vertical_edge_size(vertical: usize, _axis: Self::Axis) -> usize {
        vertical
        // match axis {
        //     AxisR::SE => vertical - 1,
        //     _ => vertical,
        // }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDR,
    ) -> Result<HexOffset, ()> {
        move_coord_r(horizontal, vertical, coord, dir, 0)
    }
}

impl HexOffsetShapeBaseLEW for OddR {
    fn move_coord_lew(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDR,
    ) -> Result<HexOffset, ()> {
        move_coord_r_lew(horizontal, vertical, coord, dir, 0)
    }
}

impl HexOffsetShapeBase for EvenR {
    type Axis = AxisR;
    fn horizontal_edge_size(horizontal: usize, _axis: Self::Axis) -> usize {
        horizontal
        // match axis {
        //     AxisR::SE => horizontal,
        //     _ => horizontal - 1,
        // }
    }

    fn vertical_edge_size(vertical: usize, _axis: Self::Axis) -> usize {
        vertical
        // match axis {
        //     AxisR::SE => vertical - 1,
        //     _ => vertical,
        // }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDR,
    ) -> Result<HexOffset, ()> {
        move_coord_r(horizontal, vertical, coord, dir, 1)
    }
}

impl HexOffsetShapeBaseLEW for EvenR {
    fn move_coord_lew(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDR,
    ) -> Result<HexOffset, ()> {
        move_coord_r_lew(horizontal, vertical, coord, dir, 1)
    }
}

fn move_coord_r(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: AxisDR,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 == flag) {
        (AxisDR::E, _) => o.add_x(1).check_x(horizontal),
        (AxisDR::W, _) => o.sub_x(1),
        (AxisDR::NE, true) | (AxisDR::NW, false) => o.add_y(1).check_y(vertical),
        (AxisDR::SE, true) | (AxisDR::SW, false) => o.sub_y(1),
        (AxisDR::NE, false) => o
            .add_x(1)
            .check_x(horizontal)
            .map(|o| o.add_y(1).check_y(vertical))
            .flatten(),
        (AxisDR::SE, false) => o.add_x(1).check_x(horizontal).map(|o| o.sub_y(1)).flatten(),
        (AxisDR::SW, true) => o.sub_x(1).map(|o| o.sub_y(1)).flatten(),
        (AxisDR::NW, true) => o.sub_x(1).map(|o| o.add_y(1).check_y(vertical)).flatten(),
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

fn move_coord_r_lew(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: AxisDR,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 == flag) {
        (AxisDR::E, _) => Some(o.add_x(1).check_x(horizontal).unwrap_or_else(|| o.set_x(0))),
        (AxisDR::W, _) => Some(o.sub_x(1).unwrap_or_else(|| o.set_x(horizontal - 1))),
        (AxisDR::NE, true) | (AxisDR::NW, false) => o.add_y(1).check_y(vertical),
        (AxisDR::SE, true) | (AxisDR::SW, false) => o.sub_y(1),
        (AxisDR::NE, false) => o
            .add_x(1)
            .check_x(horizontal)
            .unwrap_or_else(|| o.set_x(0))
            .add_y(1)
            .check_y(vertical),
        (AxisDR::SE, false) => o
            .add_x(1)
            .check_x(horizontal)
            .unwrap_or_else(|| o.set_x(0))
            .sub_y(1),
        (AxisDR::SW, true) => o
            .sub_x(1)
            .unwrap_or_else(|| o.set_x(horizontal - 1))
            .sub_y(1),
        (AxisDR::NW, true) => o
            .sub_x(1)
            .unwrap_or_else(|| o.set_x(horizontal - 1))
            .add_y(1)
            .check_y(vertical),
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

impl HexOffsetShapeBase for OddQ {
    type Axis = AxisQ;
    fn horizontal_edge_size(horizontal: usize, _axis: Self::Axis) -> usize {
        horizontal
        // match axis {
        //     AxisQ::N => horizontal,
        //     _ => horizontal - 1,
        // }
    }

    fn vertical_edge_size(vertical: usize, _axis: Self::Axis) -> usize {
        vertical
        // match axis {
        //     AxisQ::N => vertical - 1,
        //     _ => vertical,
        // }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDQ,
    ) -> Result<HexOffset, ()> {
        move_coord_q(horizontal, vertical, coord, dir, 0)
    }
}

impl HexOffsetShapeBaseLEW for OddQ {
    fn move_coord_lew(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDQ,
    ) -> Result<HexOffset, ()> {
        move_coord_q_lew(horizontal, vertical, coord, dir, 0)
    }
}

impl HexOffsetShapeBase for EvenQ {
    type Axis = AxisQ;
    fn horizontal_edge_size(horizontal: usize, _axis: Self::Axis) -> usize {
        horizontal
        // match axis {
        //     AxisQ::N => horizontal,
        //     _ => horizontal - 1,
        // }
    }

    fn vertical_edge_size(vertical: usize, _axis: Self::Axis) -> usize {
        vertical
        // match axis {
        //     AxisQ::N => vertical - 1,
        //     _ => vertical,
        // }
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDQ,
    ) -> Result<HexOffset, ()> {
        move_coord_q(horizontal, vertical, coord, dir, 1)
    }
}

impl HexOffsetShapeBaseLEW for EvenQ {
    fn move_coord_lew(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: AxisDQ,
    ) -> Result<HexOffset, ()> {
        move_coord_q_lew(horizontal, vertical, coord, dir, 1)
    }
}

impl<T, A> HexOffsetShapeBase for DirectedMarker<T>
where
    T: HexOffsetShapeBase<Axis = A>,
    A: Axis,
    A::Direction: Axis<Direction = A::Direction>,
{
    type Axis = <T::Axis as Axis>::Direction;

    fn horizontal_edge_size(horizontal: usize, _axis: Self::Axis) -> usize {
        horizontal
    }

    fn vertical_edge_size(vertical: usize, _axis: Self::Axis) -> usize {
        vertical
    }

    fn move_coord(
        horizontal: usize,
        vertical: usize,
        coord: HexOffset,
        dir: Self::Axis,
    ) -> Result<HexOffset, ()> {
        T::move_coord(horizontal, vertical, coord, dir)
    }
}

fn move_coord_q(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: AxisDQ,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 != flag) {
        (AxisDQ::N, _) => o.add_y(1).check_y(vertical),
        (AxisDQ::S, _) => o.sub_y(1),
        (AxisDQ::NE, true) | (AxisDQ::NW, false) => o.add_x(1).check_x(horizontal),
        (AxisDQ::SE, true) | (AxisDQ::SW, false) => o.sub_x(1),
        (AxisDQ::NE, false) => o.add_y(1).add_x(1).check_x(horizontal),
        (AxisDQ::SE, false) => o.add_y(1).sub_x(1).map(|o| o.check_y(vertical)).flatten(),
        (AxisDQ::SW, true) => o.sub_y(1).map(|o| o.sub_x(1)).flatten(),
        (AxisDQ::NW, true) => o.sub_y(1).map(|o| o.add_x(1).check_x(horizontal)).flatten(),
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

fn move_coord_q_lew(
    horizontal: usize,
    vertical: usize,
    coord: HexOffset,
    dir: AxisDQ,
    flag: usize,
) -> Result<HexOffset, ()> {
    let o = coord.0;
    match (dir, o.vertical() & 1 != flag) {
        (AxisDQ::N, _) => Some(o.add_x(1).check_x(horizontal).unwrap_or_else(|| o.set_x(0))),
        (AxisDQ::S, _) => Some(o.sub_x(1).unwrap_or_else(|| o.set_x(horizontal - 1))),
        (AxisDQ::NE, true) | (AxisDQ::NW, false) => o.add_x(1).check_x(horizontal),
        (AxisDQ::SE, true) | (AxisDQ::SW, false) => o.sub_x(1),
        (AxisDQ::NE, false) => o
            .add_x(1)
            .check_x(horizontal)
            .unwrap_or_else(|| o.set_x(0))
            .add_y(1)
            .check_y(vertical),
        (AxisDQ::SE, false) => o
            .add_x(1)
            .check_x(horizontal)
            .unwrap_or_else(|| o.set_x(0))
            .sub_y(1),
        (AxisDQ::SW, true) => o
            .sub_x(1)
            .unwrap_or_else(|| o.set_x(horizontal - 1))
            .sub_y(1),
        (AxisDQ::NW, true) => o
            .sub_x(1)
            .unwrap_or_else(|| o.set_x(horizontal - 1))
            .add_y(1)
            .check_y(vertical),
    }
    .map(|o| HexOffset(o))
    .ok_or(())
}

/// Shapes for hex graph with offset-based coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexOffsetShape<ShapeBase, Loop, H = usize, V = usize> {
    h: H,
    v: V,
    l: PhantomData<fn() -> Loop>,
    t: PhantomData<fn() -> ShapeBase>,
}

impl<B, L, H, V> HexOffsetShape<B, L, H, V>
where
    H: Into<usize> + Copy,
    V: Into<usize> + Copy,
{
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            l: PhantomData,
            t: PhantomData,
        }
    }
}

#[cfg(feature = "const-generic-wrap")]
pub type ConstHexOffsetShape<T, L, const H: usize, const V: usize> =
    HexOffsetShape<T, L, WrapUSIZE<H>, WrapUSIZE<V>>;

#[cfg(feature = "const-generic-wrap")]
impl<T, L, const H: usize, const V: usize> Default for ConstHexOffsetShape<T, L, H, V> {
    fn default() -> Self {
        Self::new(WrapUSIZE::<H>, WrapUSIZE::<V>)
    }
}

impl<B, H, V> Shape for HexOffsetShape<B, (), H, V>
where
    B: HexOffsetShapeBase,
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

impl<B, H, V> Shape for HexOffsetShape<B, LEW, H, V>
where
    B: HexOffsetShapeBaseLEW,
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
        B::move_coord_lew(self.horizontal(), self.vertical(), coord, dir)
    }
}
