#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::WrapUSIZE;
use std::marker::PhantomData;

use crate::{
    hex::shapes::*,
    lattice_abstract::{Axis, Coordinate, Offset, Shape},
};

/// Axial based coordinates for hex graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HexAxial<S = ()> {
    pub(crate) r: isize,
    pub(crate) q: isize,
    s: S,
}

impl<S> HexAxial<S>
where
    S: Default,
{
    pub fn new(r: isize, q: isize) -> Self {
        Self {
            q,
            r,
            s: S::default(),
        }
    }
}

impl<S: Copy + Eq> Coordinate for HexAxial<S> {}

/// Defines wheter the hex graph is `flat-top` or `point-top` and is odd or even.
pub trait HexAxialShapeBase: OE + RQ {
    type Axis: Axis;
    unsafe fn move_coord_unchecked(
        coord: HexAxial,
        dir: <Self::Axis as Axis>::Direction,
    ) -> HexAxial;
}

impl HexAxialShapeBase for OddR {
    type Axis = AxisR;

    unsafe fn move_coord_unchecked(coord: HexAxial, dir: AxisDR) -> HexAxial {
        move_coord_r(coord, dir)
    }
}

impl HexAxialShapeBase for EvenR {
    type Axis = AxisR;

    unsafe fn move_coord_unchecked(coord: HexAxial, dir: AxisDR) -> HexAxial {
        move_coord_r(coord, dir)
    }
}

impl HexAxialShapeBase for OddQ {
    type Axis = AxisQ;

    unsafe fn move_coord_unchecked(coord: HexAxial, dir: AxisDQ) -> HexAxial {
        move_coord_q(coord, dir)
    }
}

impl HexAxialShapeBase for EvenQ {
    type Axis = AxisQ;

    unsafe fn move_coord_unchecked(coord: HexAxial, dir: AxisDQ) -> HexAxial {
        move_coord_q(coord, dir)
    }
}

impl<T, A> HexAxialShapeBase for DirectedMarker<T>
where
    T: HexAxialShapeBase<Axis = A>,
    A: Axis,
    A::Direction: Axis<Direction = A::Direction>,
{
    type Axis = A::Direction;

    unsafe fn move_coord_unchecked(coord: HexAxial, dir: A::Direction) -> HexAxial {
        T::move_coord_unchecked(coord, dir)
    }
}

fn move_coord_r<S>(coord: HexAxial<S>, dir: AxisDR) -> HexAxial<S>
where
    S: Default,
{
    match dir {
        AxisDR::NE => HexAxial::new(coord.r, coord.q + 1),
        AxisDR::E => HexAxial::new(coord.r + 1, coord.q),
        AxisDR::SE => HexAxial::new(coord.r + 1, coord.q - 1),
        AxisDR::SW => HexAxial::new(coord.r, coord.q - 1),
        AxisDR::W => HexAxial::new(coord.r - 1, coord.q),
        AxisDR::NW => HexAxial::new(coord.r - 1, coord.q + 1),
    }
}

fn move_coord_q<S>(coord: HexAxial<S>, dir: AxisDQ) -> HexAxial<S>
where
    S: Default,
{
    match dir {
        AxisDQ::N => HexAxial::new(coord.r, coord.q + 1),
        AxisDQ::NE => HexAxial::new(coord.r + 1, coord.q),
        AxisDQ::SE => HexAxial::new(coord.r + 1, coord.q - 1),
        AxisDQ::S => HexAxial::new(coord.r, coord.q - 1),
        AxisDQ::SW => HexAxial::new(coord.r - 1, coord.q),
        AxisDQ::NW => HexAxial::new(coord.r - 1, coord.q + 1),
    }
}

/// Shape for Axial based coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexAxialShape<ShapeBase, Loop, H = usize, V = usize> {
    h: H,
    v: V,
    l: PhantomData<fn() -> Loop>,
    t: PhantomData<fn() -> ShapeBase>,
}

impl<ShapeBase, Loop, H, V> HexAxialShape<ShapeBase, Loop, H, V> {
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            l: PhantomData,
            t: PhantomData,
        }
    }

    #[inline]
    fn convert<L2>(&self) -> HexAxialShape<ShapeBase, L2, H, V>
    where
        H: Clone,
        V: Clone,
    {
        HexAxialShape {
            h: self.h.clone(),
            v: self.v.clone(),
            l: PhantomData,
            t: PhantomData,
        }
    }
}

/// Shape for Axial based coordinates with const size. This is ZST.
#[cfg(feature = "const-generic-wrap")]
pub type ConstHexAxialShape<T, L, const H: usize, const V: usize> =
    HexAxialShape<T, L, WrapUSIZE<H>, WrapUSIZE<V>>;

#[cfg(feature = "const-generic-wrap")]
impl<T, L, const H: usize, const V: usize> Default for ConstHexAxialShape<T, L, H, V> {
    fn default() -> Self {
        Self::new(WrapUSIZE::<H>, WrapUSIZE::<V>)
    }
}

impl<B, H, V> Shape for HexAxialShape<B, (), H, V>
where
    B: HexAxialShapeBase,
    H: Clone + Into<usize>,
    V: Clone + Into<usize>,
{
    type Axis = B::Axis;
    type Coordinate = HexAxial;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h.clone().into()
    }

    fn vertical(&self) -> usize {
        self.v.clone().into()
    }

    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        if B::IS_FLAT_TOP {
            if (coord.r as usize) < self.horizontal() {
                let v = coord.q + ((coord.r as usize + B::CONVERT_OFFSET) / 2) as isize;
                if (v as usize) < self.vertical() {
                    return Ok(Offset::new(coord.r as usize, v as usize));
                }
            }
        } else {
            // coord.q < 0 => (coord.q as usize) > usize::MAX >= self.vertical()
            if (coord.q as usize) < self.vertical() {
                let h = coord.r + ((coord.q as usize + B::CONVERT_OFFSET) / 2) as isize;
                if (h as usize) < self.horizontal() {
                    return Ok(Offset::new(h as usize, coord.q as usize));
                }
            }
        }
        Err(())
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        if B::IS_FLAT_TOP {
            let v = coord.q + ((coord.r as usize + B::CONVERT_OFFSET) / 2) as isize;
            return Offset::new(coord.r as usize, v as usize);
        } else {
            let h = coord.r + ((coord.q as usize + B::CONVERT_OFFSET) / 2) as isize;
            return Offset::new(h as usize, coord.q as usize);
        }
    }

    fn from_offset(&self, offset: crate::lattice_abstract::Offset) -> Self::Coordinate {
        HexAxial::new(
            offset.horizontal() as isize - (offset.vertical() / 2) as isize,
            offset.vertical() as isize,
        )
    }

    fn horizontal_edge_size(&self, _axis: Self::Axis) -> usize {
        self.horizontal()
    }

    fn vertical_edge_size(&self, _axis: Self::Axis) -> usize {
        self.vertical()
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        let c = unsafe { B::move_coord_unchecked(coord, dir) };
        if self.to_offset(c).is_ok() {
            Ok(c)
        } else {
            Err(())
        }
    }

    unsafe fn move_coord_unchecked(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Self::Coordinate {
        B::move_coord_unchecked(coord, dir)
    }
}

impl<B, H, V> Shape for HexAxialShape<B, LEW, H, V>
where
    B: HexAxialShapeBase,
    H: Clone + Into<usize>,
    V: Clone + Into<usize>,
{
    type Axis = B::Axis;
    type Coordinate = HexAxial;
    type OffsetConvertError = ();
    type CoordinateMoveError = ();

    fn horizontal(&self) -> usize {
        self.h.clone().into()
    }

    fn vertical(&self) -> usize {
        self.v.clone().into()
    }

    #[inline]
    fn to_offset(&self, coord: Self::Coordinate) -> Result<Offset, Self::OffsetConvertError> {
        self.convert::<()>().to_offset(coord)
    }

    #[inline]
    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        self.convert::<()>().to_offset_unchecked(coord)
    }

    #[inline]
    fn from_offset(&self, offset: crate::lattice_abstract::Offset) -> Self::Coordinate {
        self.convert::<()>().from_offset(offset)
    }

    fn horizontal_edge_size(&self, _axis: Self::Axis) -> usize {
        self.horizontal()
    }

    fn vertical_edge_size(&self, _axis: Self::Axis) -> usize {
        self.vertical()
    }

    fn move_coord(
        &self,
        coord: Self::Coordinate,
        dir: <Self::Axis as Axis>::Direction,
    ) -> Result<Self::Coordinate, Self::CoordinateMoveError> {
        let mut c = unsafe { B::move_coord_unchecked(coord, dir) };
        let q = c.q;
        if (q as usize) >= self.vertical() {
            return Err(());
        }
        let min = -((q + B::CONVERT_OFFSET as isize) / 2);
        if c.r < min {
            while {
                c.r += self.horizontal() as isize;
                c.r < min
            } {}
        } else {
            let max = self.horizontal() as isize - min;
            while c.r >= max {
                c.r -= self.horizontal() as isize;
            }
        }

        Ok(c)
    }
}
