use std::array::IntoIter;

use super::*;
use petgraph::visit::*;

#[test]
fn gen() {
    let sq = SquareGraph::<_, _, u32>::new_with(
        4,
        3,
        |x, y| x + 2 * y,
        |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
    );
    assert_eq!(sq.horizontal_node_count(), 4);
    assert_eq!(sq.vertical_node_count(), 3);
    assert_eq!(sq.node_weight((0, 0).into()), Some(&0));
    assert_eq!(sq.node_weight((3, 0).into()), Some(&3));
    assert_eq!(sq.node_weight((4, 0).into()), None);
    assert_eq!(sq.node_weight((0, 2).into()), Some(&4));
    assert_eq!(sq.node_weight((0, 3).into()), None);
    assert_eq!(
        sq.edge_weight(((0, 0).into(), Axis::Horizontal).into()),
        Some(&0)
    );
    assert_eq!(
        sq.edge_weight(((0, 2).into(), Axis::Horizontal).into()),
        Some(&4)
    );
    assert_eq!(sq.edge_weight(((0, 2).into(), Axis::Vertical).into()), None);
    assert_eq!(
        sq.edge_weight(((3, 0).into(), Axis::Horizontal).into()),
        None
    );
    assert_eq!(
        sq.edge_weight(((3, 0).into(), Axis::Vertical).into()),
        Some(&-3)
    );
}

#[test]
fn node_identifiers() {
    let sq = SquareGraph::<_, _, u32>::new_with(3, 5, |_x, _y| (), |_x, _y, _d| ());
    let mut count = 0;
    for (i, x) in sq.node_identifiers().enumerate() {
        let x = x;
        let x2 = sq.to_index(x);
        assert_eq!(x2, i);
        let x3 = sq.from_index(x2);
        assert_eq!(x, x3);
        count += 1;
    }
    assert_eq!(count, 15);
}

#[test]
fn neighbors() {
    let sq = SquareGraph::<_, _, u32>::new_with(
        3,
        5,
        |x, y| x + 2 * y,
        |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
    );

    let v00 = sq.neighbors((0, 0).into());
    debug_assert!(v00.eq(IntoIter::new([(1, 0), (0, 1)])));

    let v04 = sq.neighbors((0, 4).into());
    debug_assert!(v04.eq(IntoIter::new([(1, 4), (0, 3)])));

    let v20 = sq.neighbors((2, 0).into());
    debug_assert!(v20.eq(IntoIter::new([(1, 0), (2, 1)])));

    let v24 = sq.neighbors((2, 4).into());
    debug_assert!(v24.eq(IntoIter::new([(1, 4), (2, 3)])));

    let v12 = sq.neighbors((1, 2).into());
    debug_assert!(v12.eq(IntoIter::new([(0, 2), (2, 2), (1, 1), (1, 3)])));
}

#[test]
fn edges() {
    let sq = SquareGraph::<_, _, u32>::new_with(
        3,
        5,
        |x, y| x + 2 * y,
        |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
    );

    debug_assert!(sq
        .edges((0, 0).into())
        .map(|e| e.target())
        .eq(IntoIter::new([(1, 0), (0, 1)])));

    debug_assert!(sq.edges((0, 0).into()).map(|e| e.edge_weight).eq(&[0, 0]));
    debug_assert!(sq
        .edges((1, 1).into())
        .map(|e| e.edge_weight)
        .eq(&[2, 3, -1, -3]));

    debug_assert!(sq
        .edges((1, 2).into())
        .map(|e| e.target())
        .eq(IntoIter::new([(0, 2), (2, 2), (1, 1), (1, 3)])));
}

#[test]
fn edge_references() {
    let sq = SquareGraph::<_, _, u32>::new_with(
        3,
        5,
        |x, y| x + 2 * y,
        |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { -1 }),
    );

    let mut i = 0;
    let mut x = -1;
    for e in sq
        .edge_references()
        .filter(|x| x.id().axis == Axis::Horizontal)
    {
        let y = sq.to_index(e.edge_id.node) as i32;
        assert!(x < y);
        x = y;
        i += 1;
    }
    assert_eq!(i, 10);
    x = -1;
    i = 0;
    for e in sq
        .edge_references()
        .filter(|x| x.id().axis == Axis::Vertical)
    {
        let y = sq.to_index(e.edge_id.node) as i32;
        assert!(x < y);
        x = y;
        i += 1;
    }
    assert_eq!(i, 12);
}

#[test]
fn astar() {
    let sq = SquareGraph::<_, _, u32>::new_with(
        3,
        4,
        |_, _| (),
        |x, y, d| (x + 2 * y) as i32 * (if d.is_horizontal() { 1 } else { 3 }),
    );

    let x = petgraph::algo::astar(
        &sq,
        (0, 0).into(),
        |x| x == (2, 1),
        |e| *e.weight(),
        |x| x.distance((2, 1)) as i32,
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
        |x| x.distance((0, 0)) as i32,
    );
    assert!(x.is_some());
    let (d, p) = x.unwrap();
    assert_eq!(d, 5);
    assert_eq!(p, [(2, 1), (1, 1), (0, 1), (0, 0)])
}
