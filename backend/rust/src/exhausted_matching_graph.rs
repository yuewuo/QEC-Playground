//! # Build Exhausted Graph from Nearest-Neighbor Connected Graph
//!
//!
//! ## Note
//!
//! Since the decoding graph of open-boundary surface code has some special "virtual boundary"
//! , building fully-connected graph needs some special handling of those boundaries.
//! I did that in `ftqec.rs` but since tailored surface code decoding needs another two graphs (arXiv:1907.02554v2)
//! , I decide to abstract this functionality and make a more efficient implementation and better interface.
//!
//! This improved efficient comes from
//! 1. change to Floyd algorithm from Dijkstra algorithm, finding all shortest paths at once
//! 2. taking care of boundary: if two nodes's shortest path is worse than the sum of their individual path to boundary, no need to connect (`use_reduced_graph`)
//!

