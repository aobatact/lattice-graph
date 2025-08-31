use criterion::{criterion_group, criterion_main, Criterion};
use lattice_graph::lattice_abstract::square::{SquareGraphAbstract, SquareOffset, SquareShape};
use ndarray::Array2;
use petgraph::algo::astar;
use petgraph::data::DataMap;
use petgraph::visit::*;
use rand::prelude::*;
use std::hint::black_box;

type Graph = SquareGraphAbstract<f32, i32>;

fn astar_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("astar");

    for (h, v) in [(10, 10), (20, 20), (50, 50)] {
        let graph = Graph::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                x * v + y
            } as f32,
            |_, _| thread_rng().gen_range(1..=10),
        );

        let start = SquareOffset::from((0, 0));
        let goal = SquareOffset::from((h - 1, v - 1));

        group.bench_function(&format!("{}x{}", h, v), |b| {
            b.iter(|| {
                astar(
                    &graph,
                    black_box(start),
                    |finish| finish == black_box(goal),
                    |e| *e.weight() as f32,
                    |_| 0.0,
                )
            })
        });
    }

    group.finish();
}

fn graph_creation_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("graph_creation");

    for (h, v) in [(10, 10), (50, 50), (100, 100)] {
        group.bench_function(&format!("new_{}x{}", h, v), |b| {
            b.iter(|| {
                Graph::new_with(
                    SquareShape::new(black_box(h), black_box(v)),
                    |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
                    |_, _| thread_rng().gen_range(1..=10),
                )
            })
        });

        group.bench_function(&format!("default_{}x{}", h, v), |b| {
            b.iter(|| Graph::new(SquareShape::new(black_box(h), black_box(v))))
        });
    }

    group.finish();
}

fn node_access_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_access");

    let graph = Graph::new_with(
        SquareShape::new(100, 100),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * 100 + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("node_weight", |b| {
        let coord = SquareOffset::from((50, 0));
        b.iter(|| graph.node_weight(black_box(coord)))
    });

    group.bench_function("node_references_iter", |b| {
        b.iter(|| {
            for node_ref in graph.node_references() {
                black_box(node_ref.weight());
            }
        })
    });

    group.finish();
}

fn edge_access_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_access");

    let graph = Graph::new_with(
        SquareShape::new(100, 100),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * 100 + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("edge_references_iter", |b| {
        b.iter(|| {
            for edge_ref in graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    let coord = SquareOffset::from((50, 0));
    group.bench_function("edges_from_node", |b| {
        b.iter(|| {
            for edge_ref in graph.edges(black_box(coord)) {
                black_box(edge_ref.weight());
            }
        })
    });

    group.finish();
}

fn neighbors_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbors");

    let graph = Graph::new_with(
        SquareShape::new(100, 100),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * 100 + y
        } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    let coord = SquareOffset::from((50, 0));

    group.bench_function("neighbors_iter", |b| {
        b.iter(|| {
            for neighbor in graph.neighbors(black_box(coord)) {
                black_box(neighbor);
            }
        })
    });

    group.finish();
}

fn array2_vs_abstract_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("array2_vs_abstract");

    let h = 100;
    let v = 100;

    // Array2 creation and access
    group.bench_function("array2_creation", |b| {
        b.iter(|| {
            let mut data = Vec::with_capacity(h * v);
            for i in 0..h {
                for j in 0..v {
                    data.push((i * v + j) as f32);
                }
            }
            Array2::from_shape_vec((h, v), data).unwrap()
        })
    });

    let array = {
        let mut data = Vec::with_capacity(h * v);
        for i in 0..h {
            for j in 0..v {
                data.push((i * v + j) as f32);
            }
        }
        Array2::from_shape_vec((h, v), data).unwrap()
    };

    group.bench_function("array2_random_access", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let i = thread_rng().gen_range(0..h);
                let j = thread_rng().gen_range(0..v);
                black_box(array.get((i, j)));
            }
        })
    });

    group.bench_function("array2_sequential_access", |b| {
        b.iter(|| {
            for elem in array.iter() {
                black_box(elem);
            }
        })
    });

    // Abstract graph creation and access for comparison
    let abstract_graph = Graph::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
            let (x, y) = (coord.0.horizontal(), coord.0.vertical());
            x * v + y
        } as f32,
        |_, _| 1,
    );

    group.bench_function("abstract_creation", |b| {
        b.iter(|| {
            Graph::new_with(
                SquareShape::new(black_box(h), black_box(v)),
                |coord: SquareOffset| {
                    let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                    x * v + y
                } as f32,
                |_, _| 1,
            )
        })
    });

    group.bench_function("abstract_random_access", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let x = thread_rng().gen_range(0..h);
                let y = thread_rng().gen_range(0..v);
                let coord = SquareOffset::from((x, y));
                black_box(abstract_graph.node_weight(coord));
            }
        })
    });

    group.bench_function("abstract_sequential_access", |b| {
        b.iter(|| {
            for node_ref in abstract_graph.node_references() {
                black_box(node_ref.weight());
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    astar_bench,
    graph_creation_bench,
    node_access_bench,
    edge_access_bench,
    neighbors_bench,
    array2_vs_abstract_bench
);
criterion_main!(benches);