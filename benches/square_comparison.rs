use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petgraph::algo::astar;
use petgraph::data::DataMap;
use petgraph::visit::*;
use rand::prelude::*;
use std::hint::black_box;

// Import abstract square implementation
use lattice_graph::lattice_abstract::square::{UndirectedSquareGraph, SquareOffset, SquareShape};

fn creation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_creation_comparison");

    for (h, v) in [(10, 10), (50, 50), (100, 100)] {
        group.bench_with_input(
            BenchmarkId::new("abstract_square", format!("{}x{}", h, v)),
            &(h, v),
            |b, &(h, v)| {
                b.iter(|| {
                    UndirectedSquareGraph::<f32, i32>::new_with(
                        SquareShape::new(black_box(h), black_box(v)),
                        |coord: SquareOffset| {
                            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                            x * v + y
                        } as f32,
                        |_, _| thread_rng().gen_range(1..=10),
                    )
                })
            },
        );
    }

    group.finish();
}

fn node_access_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_node_access_comparison");

    let h = 100;
    let v = 100;

    let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * v + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("abstract_square_node_weight", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| abstract_graph.node_weight(black_box(coord)))
    });

    group.bench_function("abstract_square_node_iter", |b| {
        b.iter(|| {
            for node_ref in abstract_graph.node_references() {
                black_box(node_ref.weight());
            }
        })
    });

    group.finish();
}

fn edge_access_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_edge_access_comparison");

    let h = 100;
    let v = 100;

    let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * v + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("abstract_square_edge_iter", |b| {
        b.iter(|| {
            for edge_ref in abstract_graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    group.bench_function("abstract_square_edges_from_node", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| {
            for edge_ref in abstract_graph.edges(black_box(coord)) {
                black_box(edge_ref.weight());
            }
        })
    });

    group.finish();
}

fn neighbors_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_neighbors_comparison");

    let h = 100;
    let v = 100;

    let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * v + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("abstract_square_neighbors", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| {
            for neighbor in abstract_graph.neighbors(black_box(coord)) {
                black_box(neighbor);
            }
        })
    });

    group.finish();
}

fn astar_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_astar_comparison");

    for (h, v) in [(10, 10), (20, 20), (50, 50)] {
        let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                x * v + y
            } as f32,
            |_, _| thread_rng().gen_range(1..=10),
        );

        group.bench_with_input(
            BenchmarkId::new("abstract_square_astar", format!("{}x{}", h, v)),
            &(h, v),
            |b, _| {
                let start = SquareOffset::from((0, 0));
                let goal = SquareOffset::from((h - 1, v - 1));

                b.iter(|| {
                    astar(
                        &abstract_graph,
                        black_box(start),
                        |finish| finish == black_box(goal),
                        |e| *e.weight() as f32,
                        |_| 0.0,
                    )
                })
            },
        );
    }

    group.finish();
}

// Test memory usage patterns
fn memory_access_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_memory_access");

    // Small graph that fits in cache
    {
        let h = 10;
        let v = 10;

        let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                x * v + y
            } as f32,
            |_, _| 1,
        );

        group.bench_function("abstract_10x10_random_access", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let coord = SquareOffset::from((x, y));
                    black_box(abstract_graph.node_weight(coord));
                }
            })
        });
    }

    // Large graph that doesn't fit in cache
    {
        let h = 500;
        let v = 500;

        let abstract_graph = UndirectedSquareGraph::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                x * v + y
            } as f32,
            |_, _| 1,
        );

        group.bench_function("abstract_500x500_random_access", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let coord = SquareOffset::from((x, y));
                    black_box(abstract_graph.node_weight(coord));
                }
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    creation_comparison,
    node_access_comparison,
    edge_access_comparison,
    neighbors_comparison,
    astar_comparison,
    memory_access_comparison
);
criterion_main!(benches);
