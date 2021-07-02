# lattice-graph

[![Doc](https://docs.rs/lattice-graph/badge.svg)](https://docs.rs/lattice-graph)
[![Crate](https://img.shields.io/crates/v/lattice-graph.svg)](https://crates.io/crates/lattice-graph)

Extention library for [petgraph](https://crates.io/crates/petgraph).
This adds a specialized graph for lattice (or grid) based graph structures for petgraph.
This probides a smaller and faster graph than the general purpose `petgraph::Graph` struct.
It can be used for path finding in tilemap based game.
This is for developing game, but it can be used for other purposes as well.

# Feature Status
- [x] Square grid graph.
  - [x] Loop support for square grid.
- [x] Hex grid graph.
- [ ] Hierarchical graph structure.
- [ ] (Virtual graph?)
- [ ] (Cubic graph?)

# MSRV
Needs const generics (rustc >= 1.51) to use `const-generice-wrap` feature to fold ZST shape info.
