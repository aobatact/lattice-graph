use criterion::{criterion_group, criterion_main, Criterion};
use lattice_graph::SquareGraph;
use ndarray::Array2;
use petgraph::algo::astar;
use petgraph::data::DataMap;
use petgraph::visit::*;
use rand::prelude::*;
use std::hint::black_box;

type Graph = SquareGraph<f32, i32>;

fn astar_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("astar");

    for (h, v) in [(10, 10), (20, 20), (50, 50)] {
        let graph = Graph::new_with(
            h,
            v,
            |x, y| (x * v + y) as f32,
            |_x, _y, _axis| thread_rng().gen_range(1..=10),
        );

        use lattice_graph::square::NodeIndex;
        let start = NodeIndex::new(0usize.into(), 0usize.into());
        let goal = NodeIndex::new((h - 1).into(), (v - 1).into());

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
                    black_box(h),
                    black_box(v),
                    |x, y| (x * v + y) as f32,
                    |_, _, _| thread_rng().gen_range(1..=10),
                )
            })
        });

        group.bench_function(&format!("default_{}x{}", h, v), |b| {
            b.iter(|| Graph::new(black_box(h), black_box(v)))
        });
    }

    group.finish();
}

fn node_access_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_access");

    let graph = Graph::new_with(
        100,
        100,
        |x, y| (x * 100 + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("node_weight", |b| {
        use lattice_graph::square::NodeIndex;
        let node_id = NodeIndex::new(50usize.into(), 0usize.into()); // 50*100 + 0 = 5000
        b.iter(|| graph.node_weight(black_box(node_id)))
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
        100,
        100,
        |x, y| (x * 100 + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("edge_references_iter", |b| {
        b.iter(|| {
            for edge_ref in graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    use lattice_graph::square::NodeIndex;
    let node_id = NodeIndex::new(50usize.into(), 0usize.into()); // 50*100 + 0 = 5000
    group.bench_function("edges_from_node", |b| {
        b.iter(|| {
            for edge_ref in graph.edges(black_box(node_id)) {
                black_box(edge_ref.weight());
            }
        })
    });

    group.finish();
}

fn neighbors_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbors");

    let graph = Graph::new_with(
        100,
        100,
        |x, y| (x * 100 + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    use lattice_graph::square::NodeIndex;
    let node_id = NodeIndex::new(50usize.into(), 0usize.into()); // 50*100 + 0 = 5000

    group.bench_function("neighbors_iter", |b| {
        b.iter(|| {
            for neighbor in graph.neighbors(black_box(node_id)) {
                black_box(neighbor);
            }
        })
    });

    group.finish();
}

fn array2_vs_fixedvec2d_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("array2_operations");

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

    group.finish();
}

criterion_group!(
    benches,
    astar_bench,
    graph_creation_bench,
    node_access_bench,
    edge_access_bench,
    neighbors_bench,
    array2_vs_fixedvec2d_bench
);
criterion_main!(benches);
