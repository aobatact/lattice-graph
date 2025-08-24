use crate::hex::shapes::*;
use crate::lattice_abstract::shapes::*;
#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::WrapUSIZE;
use std::marker::PhantomData;

/// Double coordinate based coordinates for hex graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoubleCoord {
    pub(crate) h: usize,
    pub(crate) v: usize,
}

impl DoubleCoord {
    /// Create a new coordinate.
    pub fn new(h: usize, v: usize) -> Self {
        Self { h, v }
    }

    /// Get a double coord's h.
    pub fn h(&self) -> usize {
        self.h
    }

    /// Get a double coord's v.
    pub fn v(&self) -> usize {
        self.v
    }
}

impl Coordinate for DoubleCoord {}

/// A trait to make a type in [`crate::hex::shapes`] to be a shape for double coord.
pub trait DoubleCoordShapeBase: OE + RQ + Clone {
    type Axis: Axis;
    fn move_coord(
        coord: DoubleCoord,
        dir: <Self::Axis as Axis>::Direction,
        h_max: usize,
        v_max: usize,
    ) -> Option<DoubleCoord>;
}

impl DoubleCoordShapeBase for OddR {
    type Axis = AxisR;

    fn move_coord(
        coord: DoubleCoord,
        dir: AxisDR,
        h_max: usize,
        v_max: usize,
    ) -> Option<DoubleCoord> {
        move_coord_r(coord, dir, h_max, v_max)
    }
}

impl DoubleCoordShapeBase for OddQ {
    type Axis = AxisQ;

    fn move_coord(
        coord: DoubleCoord,
        dir: AxisDQ,
        h_max: usize,
        v_max: usize,
    ) -> Option<DoubleCoord> {
        move_coord_q(coord, dir, h_max, v_max)
    }
}

fn move_coord_r(
    coord: DoubleCoord,
    dir: AxisDR,
    h_max: usize,
    v_max: usize,
) -> Option<DoubleCoord> {
    'outer: {
        let mut coord = coord;
        match dir {
            AxisDR::NE => {
                coord.h += 1;
                if coord.h >= h_max {
                    break 'outer;
                }
                coord.v += 1;
                if coord.v >= v_max {
                    break 'outer;
                }
            }
            AxisDR::E => {
                coord.h += 2;
                if coord.h >= h_max {
                    break 'outer;
                }
            }
            AxisDR::SE => {
                coord.h += 1;
                if coord.h >= h_max {
                    break 'outer;
                }
                if let Some(x) = coord.v.checked_sub(1) {
                    coord.v = x;
                } else {
                    break 'outer;
                }
            }
            AxisDR::SW => {
                if let Some(x) = coord.h.checked_sub(1) {
                    coord.h = x;
                } else {
                    break 'outer;
                }
                if let Some(x) = coord.v.checked_sub(1) {
                    coord.v = x;
                } else {
                    break 'outer;
                }
            }
            AxisDR::W => {
                if let Some(x) = coord.h.checked_sub(2) {
                    coord.h = x;
                } else {
                    break 'outer;
                }
            }
            AxisDR::NW => {
                if let Some(x) = coord.h.checked_sub(1) {
                    coord.h = x;
                } else {
                    break 'outer;
                }
                coord.v += 1;
                if coord.v >= v_max {
                    break 'outer;
                }
            }
        }
        return Some(coord);
    }
    None
}

fn move_coord_q(
    coord: DoubleCoord,
    dir: AxisDQ,
    h_max: usize,
    v_max: usize,
) -> Option<DoubleCoord> {
    'block: {
        let mut coord = coord;
        match dir {
            AxisDQ::N => {
                coord.v += 2;
                if coord.v >= v_max {
                    break 'block;
                }
            }
            AxisDQ::NE => {
                coord.h += 1;
                if coord.h >= h_max {
                    break 'block;
                }
                coord.v += 1;
                if coord.v >= v_max {
                    break 'block;
                }
            }
            AxisDQ::SE => {
                coord.h += 1;
                if coord.h >= h_max {
                    break 'block;
                }
                if let Some(x) = coord.v.checked_sub(1) {
                    coord.v = x;
                } else {
                    break 'block;
                }
            }
            AxisDQ::S => {
                if let Some(x) = coord.v.checked_sub(2) {
                    coord.v = x;
                } else {
                    break 'block;
                }
            }
            AxisDQ::SW => {
                if let Some(x) = coord.h.checked_sub(1) {
                    coord.h = x;
                } else {
                    break 'block;
                }
                if let Some(x) = coord.v.checked_sub(1) {
                    coord.v = x;
                } else {
                    break 'block;
                }
            }
            AxisDQ::NW => {
                if let Some(x) = coord.h.checked_sub(1) {
                    coord.h = x;
                } else {
                    break 'block;
                }
                coord.v += 1;
                if coord.v >= v_max {
                    break 'block;
                }
            }
        }
        return Some(coord);
    }
    None
}

///Shape for double coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DoubleCoordShape<
    ShapeBase,
    Loop,
    H = usize,
    V = usize,
    Axis = <ShapeBase as DoubleCoordShapeBase>::Axis,
> {
    h: H,
    v: V,
    l: PhantomData<fn() -> Loop>,
    t: PhantomData<fn() -> ShapeBase>,
    a: PhantomData<fn() -> Axis>,
}

impl<ShapeBase, Loop, H, V, Axis> DoubleCoordShape<ShapeBase, Loop, H, V, Axis> {
    /// Create a new DoubleCoord
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            l: PhantomData,
            t: PhantomData,
            a: PhantomData,
        }
    }
}

/// Shape for double coordinates with const size. This is ZST.
#[cfg(feature = "const-generic-wrap")]
pub type ConstDoubleCoordShape<T, L, const H: usize, const V: usize> =
    DoubleCoordShape<T, L, WrapUSIZE<H>, WrapUSIZE<V>>;

#[cfg(feature = "const-generic-wrap")]
impl<T: DoubleCoordShapeBase, L, const H: usize, const V: usize> Default
    for ConstDoubleCoordShape<T, L, H, V>
{
    fn default() -> Self {
        Self::new(WrapUSIZE::<H>, WrapUSIZE::<V>)
    }
}

impl<B, H, V> Shape for DoubleCoordShape<B, (), H, V, AxisR>
where
    B: DoubleCoordShapeBase<Axis = AxisR>,
    H: Clone + Into<usize>,
    V: Clone + Into<usize>,
{
    type Axis = B::Axis;

    type Coordinate = DoubleCoord;

    type OffsetConvertError = ();

    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h.clone().into()
    }

    fn vertical(&self) -> usize {
        self.v.clone().into()
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        let h = coord.h / 2;
        let v = coord.v;
        if h < self.horizontal() && v < self.vertical() {
            Ok(Offset::new(h, v))
        } else {
            Err(())
        }
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        let h = coord.h / 2;
        let v = coord.v;
        Offset::new(h, v)
    }

    fn offset_to_coordinate(&self, offset: Offset) -> Self::Coordinate {
        let v = offset.vertical;
        let h = offset.horizontal * 2 + (v & 1);
        DoubleCoord::new(h, v)
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        B::move_coord(coord, dir, self.horizontal(), self.vertical()).ok_or(())
    }

    fn is_neighbor(&self, a: Self::Coordinate, b: Self::Coordinate) -> bool {
        let dif_v = a.v.abs_diff(b.v);
        if dif_v > 1 {
            return false;
        }
        let dif_h = a.h.abs_diff(b.h);
        // safety : 0 <= dif_v < 2 so only (2, 0) and (1, 1) is true. (not (0, 2))
        dif_h + dif_v == 2
        // match (dif_h, dif_v) {
        //     (2, 0) | (1, 1) => true,
        //     (_, 0) | (_, 1) => false,
        //     (_, _) => unsafe { crate::unreachable_debug_checked() },
        // }
    }
}
