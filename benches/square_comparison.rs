use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use petgraph::algo::astar;
use petgraph::data::DataMap;
use petgraph::visit::*;
use rand::prelude::*;

// Import both square implementations
use lattice_graph::SquareGraph;
use lattice_graph::lattice_abstract::square::{SquareGraphAbstract, SquareShape, SquareOffset};

fn creation_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("square_creation_comparison");

    for (h, v) in [(10, 10), (50, 50), (100, 100)] {
        group.bench_with_input(
            BenchmarkId::new("direct_square", format!("{}x{}", h, v)),
            &(h, v),
            |b, &(h, v)| {
                b.iter(|| {
                    SquareGraph::<f32, i32>::new_with(
                        black_box(h),
                        black_box(v),
                        |x, y| (x * v + y) as f32,
                        |_, _, _| thread_rng().gen_range(1..=10),
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("abstract_square", format!("{}x{}", h, v)),
            &(h, v),
            |b, &(h, v)| {
                b.iter(|| {
                    SquareGraphAbstract::<f32, i32>::new_with(
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
    
    // Create both graph types
    let direct_graph = SquareGraph::<f32, i32>::new_with(
        h,
        v,
        |x, y| (x * v + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    // Test node weight access
    group.bench_function("direct_square_node_weight", |b| {
        use lattice_graph::square::NodeIndex;
        let node_id = NodeIndex::new(50usize.into(), 50usize.into());
        b.iter(|| direct_graph.node_weight(black_box(node_id)))
    });

    group.bench_function("abstract_square_node_weight", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| abstract_graph.node_weight(black_box(coord)))
    });

    // Test node iteration
    group.bench_function("direct_square_node_iter", |b| {
        b.iter(|| {
            for node_ref in direct_graph.node_references() {
                black_box(node_ref.weight());
            }
        })
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
    
    let direct_graph = SquareGraph::<f32, i32>::new_with(
        h,
        v,
        |x, y| (x * v + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    // Test edge iteration
    group.bench_function("direct_square_edge_iter", |b| {
        b.iter(|| {
            for edge_ref in direct_graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    group.bench_function("abstract_square_edge_iter", |b| {
        b.iter(|| {
            for edge_ref in abstract_graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    // Test edges from a node
    group.bench_function("direct_square_edges_from_node", |b| {
        use lattice_graph::square::NodeIndex;
        let node_id = NodeIndex::new(50usize.into(), 50usize.into());
        b.iter(|| {
            for edge_ref in direct_graph.edges(black_box(node_id)) {
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
    
    let direct_graph = SquareGraph::<f32, i32>::new_with(
        h,
        v,
        |x, y| (x * v + y) as f32,
        |_, _, _| thread_rng().gen_range(1..=10),
    );

    let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
        SquareShape::new(h, v),
        |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
        |_, _| thread_rng().gen_range(1..=10),
    );

    group.bench_function("direct_square_neighbors", |b| {
        use lattice_graph::square::NodeIndex;
        let node_id = NodeIndex::new(50usize.into(), 50usize.into());
        b.iter(|| {
            for neighbor in direct_graph.neighbors(black_box(node_id)) {
                black_box(neighbor);
            }
        })
    });

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
        let direct_graph = SquareGraph::<f32, i32>::new_with(
            h,
            v,
            |x, y| (x * v + y) as f32,
            |_, _, _| thread_rng().gen_range(1..=10),
        );

        let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
            |_, _| thread_rng().gen_range(1..=10),
        );

        group.bench_with_input(
            BenchmarkId::new("direct_square_astar", format!("{}x{}", h, v)),
            &(h, v),
            |b, _| {
                use lattice_graph::square::NodeIndex;
                let start = NodeIndex::new(0usize.into(), 0usize.into());
                let goal = NodeIndex::new((h - 1).into(), (v - 1).into());
                
                b.iter(|| {
                    astar(
                        &direct_graph,
                        black_box(start),
                        |finish| finish == black_box(goal),
                        |e| *e.weight() as f32,
                        |_| 0.0,
                    )
                })
            },
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
        
        let direct_graph = SquareGraph::<f32, i32>::new_with(
            h,
            v,
            |x, y| (x * v + y) as f32,
            |_, _, _| 1,
        );

        let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
            |_, _| 1,
        );

        group.bench_function("direct_10x10_random_access", |b| {
            use lattice_graph::square::NodeIndex;
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let node_id = NodeIndex::new(x.into(), y.into());
                    black_box(direct_graph.node_weight(node_id));
                }
            })
        });

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
        
        let direct_graph = SquareGraph::<f32, i32>::new_with(
            h,
            v,
            |x, y| (x * v + y) as f32,
            |_, _, _| 1,
        );

        let abstract_graph = SquareGraphAbstract::<f32, i32>::new_with(
            SquareShape::new(h, v),
            |coord: SquareOffset| {
                        let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                        x * v + y
                    } as f32,
            |_, _| 1,
        );

        group.bench_function("direct_500x500_random_access", |b| {
            use lattice_graph::square::NodeIndex;
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let node_id = NodeIndex::new(x.into(), y.into());
                    black_box(direct_graph.node_weight(node_id));
                }
            })
        });

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