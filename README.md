# lattice-graph

Extention library for [petgraph](https://crates.io/crates/petgraph).
This adds a specialized graph for lattice (or grid) based graph structures for petgraph.
This probides a smaller and faster graph than the general purpose `petgraph::Graph` struct.
It can be used for passfinding in tilemap based game.
This is for developing game, but it can be used for other purposes as well.

# Feature Status
- [x] Square grid graph.
  - [ ] Loop support for square grid.
- [ ] Hex grid graph.
- [ ] Hierarchical graph structure.
- [ ] (Virtual graph?)
- [ ] (Cubic graph?)
