use petgraph::graph::IndexType;

/// Axis of the Square grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    /// Check whether axis is horizontal.
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Axis::Horizontal)
    }
    /// Check whether axis is vertical.
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Axis::Vertical)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SquareDirection {
    Foward(Axis),
    Backward(Axis),
}

impl SquareDirection {
    /// Foward Horizontal
    pub const fn up() -> Self {
        Self::Foward(Axis::Vertical)
    }
    /// Backward Horizontal
    pub const fn down() -> Self {
        Self::Backward(Axis::Vertical)
    }
    /// Backward Vertical
    pub const fn left() -> Self {
        Self::Backward(Axis::Horizontal)
    }
    /// Foward Vertical
    pub const fn right() -> Self {
        Self::Foward(Axis::Horizontal)
    }
    pub const fn is_horizontal(&self) -> bool {
        match self {
            SquareDirection::Foward(x) | SquareDirection::Backward(x) => x.is_horizontal(),
        }
    }
    pub const fn is_vertical(&self) -> bool {
        match self {
            SquareDirection::Foward(x) | SquareDirection::Backward(x) => x.is_vertical(),
        }
    }
}

impl From<(Axis, bool)> for SquareDirection {
    fn from((axis, dir): (Axis, bool)) -> Self {
        if dir {
            SquareDirection::Foward(axis)
        } else {
            SquareDirection::Backward(axis)
        }
    }
}

impl From<SquareDirection> for (Axis, bool) {
    fn from(dir: SquareDirection) -> Self {
        match dir {
            SquareDirection::Backward(x) => (x, false),
            SquareDirection::Foward(x) => (x, true),
        }
    }
}

/// Node index for [`SquareGraph`]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex<Ix: IndexType> {
    pub horizontal: Ix,
    pub vertical: Ix,
}

impl<Ix: IndexType> NodeIndex<Ix> {
    /// Create a Index from horizontal and vertical.
    pub fn new(horizontal: Ix, vertical: Ix) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Returns the manhattan distance
    pub fn distance<T: Into<(usize, usize)>>(&self, target: T) -> usize {
        let target: (usize, usize) = target.into();
        (self.horizontal.index() as isize - target.0 as isize).abs() as usize
            + (self.vertical.index() as isize - target.1 as isize).abs() as usize
    }

    /// Get the edge from this node. This does not check whether the node is valid in graph.
    pub unsafe fn get_edge_id_unchecked(&self, dir: SquareDirection) -> EdgeIndex<Ix> {
        match dir {
            SquareDirection::Foward(x) => (*self, x),
            SquareDirection::Backward(a @ Axis::Vertical) => (
                Self::new(self.horizontal, Ix::new(self.vertical.index() - 1)),
                a,
            ),
            SquareDirection::Backward(a @ Axis::Horizontal) => (
                Self::new(Ix::new(self.horizontal.index() - 1), self.vertical),
                a,
            ),
        }
        .into()
    }

    #[inline]
    pub fn up(self) -> Self {
        Self {
            vertical: Ix::new(self.vertical.index() + 1),
            ..self
        }
    }

    #[inline]
    pub fn down(self) -> Self {
        Self {
            vertical: Ix::new(self.vertical.index() - 1),
            ..self
        }
    }

    #[inline]
    pub fn right(self) -> Self {
        Self {
            horizontal: Ix::new(self.horizontal.index() + 1),
            ..self
        }
    }

    #[inline]
    pub fn left(self) -> Self {
        Self {
            horizontal: Ix::new(self.horizontal.index() - 1),
            ..self
        }
    }
}

impl<Ix: IndexType> PartialEq<(usize, usize)> for NodeIndex<Ix> {
    fn eq(&self, value: &(usize, usize)) -> bool {
        &(self.horizontal.index(), self.vertical.index()) == value
    }
}

impl<Ix: IndexType> From<(usize, usize)> for NodeIndex<Ix> {
    fn from(value: (usize, usize)) -> Self {
        NodeIndex::new(Ix::new(value.0), Ix::new(value.1))
    }
}

impl<Ix: IndexType> From<NodeIndex<Ix>> for (usize, usize) {
    fn from(value: NodeIndex<Ix>) -> Self {
        (value.horizontal.index(), value.vertical.index())
    }
}

/// Edge Index of [`SquareGraph`]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeIndex<Ix: IndexType> {
    pub node: NodeIndex<Ix>,
    pub axis: Axis,
}

impl<Ix: IndexType> From<(NodeIndex<Ix>, Axis)> for EdgeIndex<Ix> {
    fn from((n, a): (NodeIndex<Ix>, Axis)) -> Self {
        Self { node: n, axis: a }
    }
}

impl<Ix: IndexType> From<(NodeIndex<Ix>, SquareDirection)> for EdgeIndex<Ix> {
    fn from((n, a): (NodeIndex<Ix>, SquareDirection)) -> Self {
        unsafe { n.get_edge_id_unchecked(a) }
    }
}
