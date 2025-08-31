use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use petgraph::algo::astar;
use petgraph::data::DataMap;
use petgraph::visit::*;
use petgraph::{Graph, Undirected};
use rand::prelude::*;
use std::collections::HashMap;
use std::hint::black_box;

// Import abstract square implementation
use lattice_graph::lattice_abstract::square::{UndirectedSquareGraph, SquareOffset, SquareShape};

type PetGraph = Graph<f32, i32, Undirected>;

// Helper function to create a petgraph UnGraph equivalent to the square grid
fn create_petgraph_square(h: usize, v: usize) -> (PetGraph, HashMap<(usize, usize), petgraph::graph::NodeIndex>) {
    let mut graph = Graph::new_undirected();
    let mut coord_to_node = HashMap::new();
    
    // Pre-allocate capacity for better performance
    graph.reserve_nodes(h * v);
    graph.reserve_edges((h - 1) * v + h * (v - 1));
    coord_to_node.reserve(h * v);
    
    // Create all nodes first
    for x in 0..h {
        for y in 0..v {
            let weight = (x * v + y) as f32;
            let node_idx = graph.add_node(weight);
            coord_to_node.insert((x, y), node_idx);
        }
    }
    
    // Add edges between adjacent nodes
    for x in 0..h {
        for y in 0..v {
            let current_node = coord_to_node[&(x, y)];
            
            // Right edge
            if x + 1 < h {
                let right_node = coord_to_node[&(x + 1, y)];
                // Use deterministic weights matching the abstract graph
                let edge_weight = ((x + y) % 10 + 1) as i32;
                graph.add_edge(current_node, right_node, edge_weight);
            }
            
            // Down edge
            if y + 1 < v {
                let down_node = coord_to_node[&(x, y + 1)];
                // Use deterministic weights matching the abstract graph  
                let edge_weight = ((x + y) % 10 + 1) as i32;
                graph.add_edge(current_node, down_node, edge_weight);
            }
        }
    }
    
    (graph, coord_to_node)
}

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

        group.bench_with_input(
            BenchmarkId::new("petgraph_ungraph", format!("{}x{}", h, v)),
            &(h, v),
            |b, &(h, v)| {
                b.iter(|| {
                    create_petgraph_square(black_box(h), black_box(v))
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

    let (petgraph, coord_to_node) = create_petgraph_square(h, v);

    group.bench_function("abstract_square_node_weight", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| abstract_graph.node_weight(black_box(coord)))
    });

    group.bench_function("petgraph_node_weight", |b| {
        let node_idx = coord_to_node[&(50, 50)];
        b.iter(|| petgraph.node_weight(black_box(node_idx)))
    });

    group.bench_function("abstract_square_node_iter", |b| {
        b.iter(|| {
            for node_ref in abstract_graph.node_references() {
                black_box(node_ref.weight());
            }
        })
    });

    group.bench_function("petgraph_node_iter", |b| {
        b.iter(|| {
            for node_weight in petgraph.node_weights() {
                black_box(node_weight);
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

    let (petgraph, coord_to_node) = create_petgraph_square(h, v);

    group.bench_function("abstract_square_edge_iter", |b| {
        b.iter(|| {
            for edge_ref in abstract_graph.edge_references() {
                black_box(edge_ref.weight());
            }
        })
    });

    group.bench_function("petgraph_edge_iter", |b| {
        b.iter(|| {
            for edge_weight in petgraph.edge_weights() {
                black_box(edge_weight);
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

    group.bench_function("petgraph_edges_from_node", |b| {
        let node_idx = coord_to_node[&(50, 50)];
        b.iter(|| {
            for edge_ref in petgraph.edges(black_box(node_idx)) {
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

    let (petgraph, coord_to_node) = create_petgraph_square(h, v);

    group.bench_function("abstract_square_neighbors", |b| {
        let coord = SquareOffset::from((50, 50));
        b.iter(|| {
            for neighbor in abstract_graph.neighbors(black_box(coord)) {
                black_box(neighbor);
            }
        })
    });

    group.bench_function("petgraph_neighbors", |b| {
        let node_idx = coord_to_node[&(50, 50)];
        b.iter(|| {
            for neighbor in petgraph.neighbors(black_box(node_idx)) {
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
                (x * v + y) as f32
            },
            |coord: SquareOffset, _| {
                // Use deterministic weights based on coordinate for consistent benchmarking
                let (x, y) = (coord.0.horizontal(), coord.0.vertical());
                ((x + y) % 10 + 1) as i32
            },
        );

        let (petgraph, coord_to_node) = create_petgraph_square(h, v);

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
                        |node| {
                            // Manhattan distance heuristic
                            let dx = (node.0.horizontal() as i32 - goal.0.horizontal() as i32).abs();
                            let dy = (node.0.vertical() as i32 - goal.0.vertical() as i32).abs();
                            (dx + dy) as f32
                        },
                    )
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("petgraph_astar", format!("{}x{}", h, v)),
            &(h, v),
            |b, _| {
                let start_node = coord_to_node[&(0, 0)];
                let goal_node = coord_to_node[&(h - 1, v - 1)];

                b.iter(|| {
                    astar(
                        &petgraph,
                        black_box(start_node),
                        |finish| finish == black_box(goal_node),
                        |e| *e.weight() as f32,
                        |node_idx| {
                            // Same Manhattan distance heuristic as abstract square
                            // Convert node index back to coordinates (same layout as abstract square)
                            let index = node_idx.index();
                            let x = index / v;
                            let y = index % v;
                            let dx = (x as i32 - (h - 1) as i32).abs();
                            let dy = (y as i32 - (v - 1) as i32).abs();
                            (dx + dy) as f32
                        },
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

        let (petgraph, coord_to_node) = create_petgraph_square(h, v);

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

        group.bench_function("petgraph_10x10_random_access", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let node_idx = coord_to_node[&(x, y)];
                    black_box(petgraph.node_weight(node_idx));
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

        let (petgraph, coord_to_node) = create_petgraph_square(h, v);

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

        group.bench_function("petgraph_500x500_random_access", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let x = thread_rng().gen_range(0..h);
                    let y = thread_rng().gen_range(0..v);
                    let node_idx = coord_to_node[&(x, y)];
                    black_box(petgraph.node_weight(node_idx));
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
