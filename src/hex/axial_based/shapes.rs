#[cfg(feature = "const-generic-wrap")]
use const_generic_wrap::WrapUSIZE;
use std::marker::PhantomData;

use crate::{
    hex::shapes::*,
    lattice_abstract::{Axis, Coordinate, Offset, Shape},
};

pub trait AxialCoord<I = isize>: Clone + Coordinate {
    ///Creates a new Coordinate.
    fn new(r: I, q: I) -> Self;
    /// Get a reference to the hex axial's r.
    fn r(&self) -> I;
    /// Get a reference to the hex axial's q.
    fn q(&self) -> I;
}

/// Axial based coordinates for hex graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HexAxial {
    pub(crate) r: isize,
    pub(crate) q: isize,
}

impl HexAxial {
    /// Get a reference to the hex axial's r.
    pub fn r(&self) -> isize {
        self.r
    }

    /// Get a reference to the hex axial's q.
    pub fn q(&self) -> isize {
        self.q
    }
}

impl HexAxial {
    ///Creates a new Coordinate.
    pub fn new(r: isize, q: isize) -> Self {
        Self { q, r }
    }
}

impl Coordinate for HexAxial {}
impl AxialCoord for HexAxial {
    #[inline]
    fn new(r: isize, q: isize) -> Self {
        Self::new(r, q)
    }

    #[inline]
    fn r(&self) -> isize {
        self.r
    }

    #[inline]
    fn q(&self) -> isize {
        self.q
    }
}

/// Defines wheter the hex graph is `flat-top` or `point-top` and is odd or even.
pub trait HexAxialShapeBase<HA: AxialCoord>: OE + RQ + Clone {
    type Axis: Axis;
    unsafe fn move_coord_unchecked(coord: HA, dir: <Self::Axis as Axis>::Direction) -> HA;
}

impl<HA: AxialCoord> HexAxialShapeBase<HA> for OddR {
    type Axis = AxisR;

    unsafe fn move_coord_unchecked(coord: HA, dir: AxisDR) -> HA {
        move_coord_r(coord, dir)
    }
}

impl<HA: AxialCoord> HexAxialShapeBase<HA> for EvenR {
    type Axis = AxisR;

    unsafe fn move_coord_unchecked(coord: HA, dir: AxisDR) -> HA {
        move_coord_r(coord, dir)
    }
}

impl<HA: AxialCoord> HexAxialShapeBase<HA> for OddQ {
    type Axis = AxisQ;

    unsafe fn move_coord_unchecked(coord: HA, dir: AxisDQ) -> HA {
        move_coord_q(coord, dir)
    }
}

impl<HA: AxialCoord> HexAxialShapeBase<HA> for EvenQ {
    type Axis = AxisQ;

    unsafe fn move_coord_unchecked(coord: HA, dir: AxisDQ) -> HA {
        move_coord_q(coord, dir)
    }
}

impl<T, A, HA: AxialCoord> HexAxialShapeBase<HA> for DirectedMarker<T>
where
    T: HexAxialShapeBase<HA, Axis = A>,
    A: Axis,
    A::Direction: Axis<Direction = A::Direction>,
{
    type Axis = A::Direction;

    unsafe fn move_coord_unchecked(coord: HA, dir: A::Direction) -> HA {
        T::move_coord_unchecked(coord, dir)
    }
}

fn move_coord_r<HA: AxialCoord>(coord: HA, dir: AxisDR) -> HA {
    match dir {
        AxisDR::NE => HA::new(coord.r(), coord.q() + 1),
        AxisDR::E => HA::new(coord.r() + 1, coord.q()),
        AxisDR::SE => HA::new(coord.r() + 1, coord.q() - 1),
        AxisDR::SW => HA::new(coord.r(), coord.q() - 1),
        AxisDR::W => HA::new(coord.r() - 1, coord.q()),
        AxisDR::NW => HA::new(coord.r() - 1, coord.q() + 1),
    }
}

fn move_coord_q<HA: AxialCoord>(coord: HA, dir: AxisDQ) -> HA {
    match dir {
        AxisDQ::N => HA::new(coord.r(), coord.q() + 1),
        AxisDQ::NE => HA::new(coord.r() + 1, coord.q()),
        AxisDQ::SE => HA::new(coord.r() + 1, coord.q() - 1),
        AxisDQ::S => HA::new(coord.r(), coord.q() - 1),
        AxisDQ::SW => HA::new(coord.r() - 1, coord.q()),
        AxisDQ::NW => HA::new(coord.r() - 1, coord.q() + 1),
    }
}

/// Shape for Axial based coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexAxialShape<ShapeBase, Loop, H = usize, V = usize, HA = HexAxial> {
    h: H,
    v: V,
    l: PhantomData<fn() -> Loop>,
    t: PhantomData<fn() -> ShapeBase>,
    ha: PhantomData<fn() -> HA>,
}

impl<ShapeBase, Loop, H, V, HA> HexAxialShape<ShapeBase, Loop, H, V, HA> {
    /// Create a new shape.
    pub fn new(h: H, v: V) -> Self {
        Self {
            h,
            v,
            l: PhantomData,
            t: PhantomData,
            ha: PhantomData,
        }
    }

    #[inline]
    fn convert<L2>(&self) -> HexAxialShape<ShapeBase, L2, H, V, HA>
    where
        H: Clone,
        V: Clone,
    {
        HexAxialShape {
            h: self.h.clone(),
            v: self.v.clone(),
            l: PhantomData,
            t: PhantomData,
            ha: PhantomData,
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

impl<B, H, V, HA> Shape for HexAxialShape<B, (), H, V, HA>
where
    HA: AxialCoord,
    B: HexAxialShapeBase<HA>,
    H: Clone + Into<usize>,
    V: Clone + Into<usize>,
{
    type Axis = B::Axis;
    type Coordinate = HA;
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
            if (coord.r() as usize) < self.horizontal() {
                let v = coord.q() + ((coord.r() as usize + B::CONVERT_OFFSET) / 2) as isize;
                if (v as usize) < self.vertical() {
                    return Ok(Offset::new(coord.r() as usize, v as usize));
                }
            }
        } else {
            // coord.q() < 0 => (coord.q() as usize) > usize::MAX >= self.vertical()
            if (coord.q() as usize) < self.vertical() {
                let h = coord.r() + ((coord.q() as usize + B::CONVERT_OFFSET) / 2) as isize;
                if (h as usize) < self.horizontal() {
                    return Ok(Offset::new(h as usize, coord.q() as usize));
                }
            }
        }
        Err(())
    }

    unsafe fn to_offset_unchecked(&self, coord: Self::Coordinate) -> Offset {
        if B::IS_FLAT_TOP {
            let v = coord.q() + ((coord.r() as usize + B::CONVERT_OFFSET) / 2) as isize;
            Offset::new(coord.r() as usize, v as usize)
        } else {
            let h = coord.r() + ((coord.q() as usize + B::CONVERT_OFFSET) / 2) as isize;
            Offset::new(h as usize, coord.q() as usize)
        }
    }

    fn offset_to_coordinate(&self, offset: crate::lattice_abstract::Offset) -> Self::Coordinate {
        if B::IS_FLAT_TOP {
            HA::new(
                offset.horizontal() as isize,
                offset.vertical() as isize
                    - ((offset.horizontal() + B::CONVERT_OFFSET) / 2) as isize,
            )
        } else {
            HA::new(
                offset.horizontal() as isize
                    - ((offset.vertical() + B::CONVERT_OFFSET) / 2) as isize,
                offset.vertical() as isize,
            )
        }
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

impl<B, H, V, HA> Shape for HexAxialShape<B, LoopEW, H, V, HA>
where
    HA: AxialCoord,
    B: HexAxialShapeBase<HA>,
    H: Clone + Into<usize>,
    V: Clone + Into<usize>,
{
    type Axis = B::Axis;
    type Coordinate = HA;
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
    fn offset_to_coordinate(&self, offset: crate::lattice_abstract::Offset) -> Self::Coordinate {
        self.convert::<()>().offset_to_coordinate(offset)
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
        let q = c.q();
        if (q as usize) >= self.vertical() {
            return Err(());
        }
        let min = -((q + B::CONVERT_OFFSET as isize) / 2);
        let h = self.horizontal() as isize;
        let mut r = c.r();
        if r < min {
            while {
                r += h;
                r < min
            } {}
        } else {
            let max = h - min;
            while r >= max {
                r -= h;
            }
        }

        Ok(HA::new(r, q))
    }
}
