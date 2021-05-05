//! # FPGA Generator
//!
//! This module aims at generating Verilog code automatically.

#![allow(non_snake_case)]  // allow non snake case for variables in Verilog code

use super::union_find_decoder;
use super::distributed_uf_decoder;
use super::ftqec;
use super::types::QubitType;
use super::pyo3::prelude::*;
use super::pyo3::types::{IntoPyDict};
use std::collections::{HashMap};
use lazy_static::lazy_static;
use std::sync::RwLock;
use super::util::getFileContentFromMultiplePlaces;

lazy_static! {
    static ref PY_SEARCH_DIRECTORIES: RwLock<Vec<String>> = RwLock::new(vec![
        format!("./pylib/"),
        format!("../../pylib/"),
    ]);
}

pub fn run_matched_fpga_generator(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("perfect_measurement_distributed_union_find", Some(matches)) => {
            let d = value_t!(matches, "d", usize).expect("required");
            perfect_measurement_distributed_union_find(d);
        }
        ("fault_tolerant_distributed_union_find", Some(matches)) => {
            let d = value_t!(matches, "d", usize).expect("required");
            let measurement_rounds = value_t!(matches, "measurement_rounds", usize).expect("required");
            let p = value_t!(matches, "p", f64).unwrap_or(0.001);  // not required if all weights are identical
            let autotune = matches.is_present("autotune");  // default autotune is disabled
            let fast_channel_interval = value_t!(matches, "fast_channel_interval", usize).unwrap_or(1);
            fault_tolerant_distributed_union_find(d, measurement_rounds, p, autotune, fast_channel_interval);
        }
        _ => unreachable!()
    }
}

pub struct DistributedUnionFind<N: DufNode> {
    pub module_name: String,
    pub distance_solver_name: String,
    /// 2 for perfect measurement or 3 for imperfect measurement
    pub dimension: usize,
    pub nodes: Vec<N>,
    pub neighbors: Vec<DufNeighbor>,
    pub fast_channels: Vec<DufFastChannel>,
}

pub trait DufNode {
    fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny>;
}

impl<N: DufNode> DistributedUnionFind<N> {
    pub fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny> {
        let mut nodes: Vec<&PyAny> = Vec::new();
        for node in self.nodes.iter() {
            nodes.push(node.build_py(module, py)?);
        }
        let mut neighbors: Vec<&PyAny> = Vec::new();
        for neighbor in self.neighbors.iter() {
            neighbors.push(neighbor.build_py(module, py)?);
        }
        let mut fast_channels: Vec<&PyAny> = Vec::new();
        for fast_channel in self.fast_channels.iter() {
            fast_channels.push(fast_channel.build_py(module, py)?);
        }
        module.getattr("DistributedUnionFind")?.call((), Some([
            ("module_name", self.module_name.clone().into_py(py)),
            ("distance_solver_name", self.distance_solver_name.clone().into_py(py)),
            ("dimension", self.dimension.clone().into_py(py)),
            ("nodes", nodes.into_py(py)),
            ("neighbors", neighbors.into_py(py)),
            ("fast_channels", fast_channels.into_py(py)),
        ].into_py_dict(py)))
    }
    pub fn generate_code(&self) -> String {
        Python::with_gil(|py| {
            (|py: Python| -> PyResult<String> {
                // get source file
                let py_module_name = format!("distributed_union_find_FPGA_generator");
                let lib_filename = format!("{}.py", py_module_name);
                let lib_source = getFileContentFromMultiplePlaces(&PY_SEARCH_DIRECTORIES.read().unwrap(), &lib_filename).unwrap();
                let module = PyModule::from_code(py, lib_source.as_str(), lib_filename.as_str(), py_module_name.as_str())?;
                // generate code
                let duf = self.build_py(module, py)?;
                let ret: String = duf.call_method0("generate_code")?.extract()?;
                Ok(ret)
            })(py).map_err(|e| {
                e.print_and_set_sys_last_vars(py);
            })
        }).expect("python run failed")
    }
}

pub struct DufNeighbor {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
    /// the total length of this edge
    pub length: usize,
}

impl DufNeighbor {
    fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny> {
        module.getattr("DufNeighbor")?.call((), Some([
            ("a", self.a.into_py(py)),
            ("b", self.b.into_py(py)),
            ("length", self.length.into_py(py)),
        ].into_py_dict(py)))
    }
}

pub struct DufFastChannel {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
}

impl DufFastChannel {
    fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny> {
        module.getattr("DufFastChannel")?.call((), Some([
            ("a", self.a.into_py(py)),
            ("b", self.b.into_py(py)),
        ].into_py_dict(py)))
    }
}

pub struct DufNode2d {
    pub address: (usize, usize),  // compressed address, can reduce 1 bit for each axis
    pub origin: (usize, usize),  // original address
    pub boundary_cost: Option<usize>,
    pub is_error_syndrome: bool,
    pub channel_index: HashMap<usize, usize>,
    pub index_channel: Vec<usize>,
    pub neighbor_count: usize,
}

impl DufNode for DufNode2d {
    fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny> {
        module.getattr("DufNode")?.call((), Some([
            ("address", [self.address.0, self.address.1].into_py(py)),
            ("origin", [self.origin.0, self.origin.1].into_py(py)),
            ("boundary_cost", self.boundary_cost.clone().into_py(py)),
            ("is_error_syndrome", self.is_error_syndrome.into_py(py)),
            ("channel_index", self.channel_index.clone().into_py(py)),
            ("index_channel", self.index_channel.clone().into_py(py)),
            ("neighbor_count", self.neighbor_count.into_py(py)),
        ].into_py_dict(py)))
    }
}

pub struct DufNode3d {
    pub address: (usize, usize, usize),  // compressed address, can reduce 1 bit for each axis
    pub origin: (usize, usize, usize),  // original address
    pub boundary_cost: Option<usize>,
    pub is_error_syndrome: bool,
    pub channel_index: HashMap<usize, usize>,
    pub index_channel: Vec<usize>,
    pub neighbor_count: usize,
}

impl DufNode for DufNode3d {
    fn build_py<'a>(&self, module: &'a PyModule, py: Python) -> PyResult<&'a PyAny> {
        module.getattr("DufNode")?.call((), Some([
            ("address", [self.address.0, self.address.1, self.address.2].into_py(py)),
            ("origin", [self.origin.0, self.origin.1, self.origin.2].into_py(py)),
            ("boundary_cost", self.boundary_cost.clone().into_py(py)),
            ("is_error_syndrome", self.is_error_syndrome.into_py(py)),
            ("channel_index", self.channel_index.clone().into_py(py)),
            ("index_channel", self.index_channel.clone().into_py(py)),
            ("neighbor_count", self.neighbor_count.into_py(py)),
        ].into_py_dict(py)))
    }
}

impl DistributedUnionFind<DufNode2d> {
    pub fn from_union_find_decoder(decoder: &union_find_decoder::UnionFindDecoder<(usize, usize)>) -> Self {
        let mut coordinates_parity = None;
        let mut nodes: Vec<_> = decoder.nodes.iter().map(|node| {
            let origin = node.node.user_data;
            let my_coordinates_parity = Some((origin.0 % 2, origin.1 % 2));
            if coordinates_parity.is_none() {
                coordinates_parity = my_coordinates_parity;
            }
            assert_eq!(coordinates_parity, my_coordinates_parity, "coordinate parity must be same for the address compression technique to work");
            let address = (origin.0 >> 1, origin.1 >> 1);
            DufNode2d {
                address: address,
                origin: origin,
                boundary_cost: node.node.boundary_cost,
                is_error_syndrome: node.node.is_error_syndrome,
                channel_index: HashMap::new(),
                index_channel: Vec::new(),
                neighbor_count: 0,
            }
        }).collect();
        let mut neighbors = Vec::new();
        for union_find_decoder::NeighborEdge {a, b, increased: _, length} in decoder.input_neighbors.iter() {
            neighbors.push(DufNeighbor {
                a: *a,
                b: *b,
                length: *length,
            });
            let a_idx = nodes[*a].index_channel.len();
            nodes[*a].channel_index.insert(*b, a_idx);
            nodes[*a].index_channel.push(*b);
            let b_idx = nodes[*b].index_channel.len();
            nodes[*b].channel_index.insert(*a, b_idx);
            nodes[*b].index_channel.push(*a);
        }
        // update neighbor_count
        for node in nodes.iter_mut() {
            node.neighbor_count = node.index_channel.len();
        }
        // since `union_find_decoder` don't consider fast channels, simply ignore here. otherwise just do similar thing as above
        let fast_channels = Vec::new();
        Self {
            module_name: format!("module_name"),
            distance_solver_name: format!("tree_distance_2d_solver"),
            dimension: 2,
            nodes: nodes,
            neighbors: neighbors,
            fast_channels: fast_channels,
        }
    }
}

impl DistributedUnionFind<DufNode3d> {
    pub fn from_distributed_union_find_decoder(decoder: &distributed_uf_decoder::DistributedUnionFind<(usize, usize, usize)>) -> Self {
        let mut coordinates_parity = None;
        let mut minimum_t = usize::MAX;
        assert!(decoder.nodes.len() > 0, "should at least contain 1 node");
        for node in decoder.nodes.iter() {
            let origin = node.user_data;
            minimum_t = std::cmp::min(origin.0, minimum_t);
        }
        let mut nodes: Vec<_> = decoder.nodes.iter().map(|node| {
            let origin = node.user_data;
            let my_coordinates_parity = Some((origin.0 % 6, origin.1 % 2, origin.2 % 2));
            if coordinates_parity.is_none() {
                coordinates_parity = my_coordinates_parity;
            }
            assert_eq!(coordinates_parity, my_coordinates_parity, "coordinate parity must be same for the address compression technique to work");
            let address = ((origin.0 - minimum_t) / 6, origin.1 >> 1, origin.2 >> 1);
            DufNode3d {
                address: address,
                origin: origin,
                boundary_cost: node.boundary_cost,
                is_error_syndrome: node.is_error_syndrome,
                channel_index: HashMap::new(),
                index_channel: Vec::new(),
                neighbor_count: 0,
            }
        }).collect();
        let mut neighbors = Vec::new();
        for distributed_uf_decoder::InputNeighbor {a, b, increased: _, length, latency: _} in decoder.input_neighbors.iter() {
            neighbors.push(DufNeighbor {
                a: *a,
                b: *b,
                length: *length,
            });
            let a_idx = nodes[*a].index_channel.len();
            nodes[*a].channel_index.insert(*b, a_idx);
            nodes[*a].index_channel.push(*b);
            let b_idx = nodes[*b].index_channel.len();
            nodes[*b].channel_index.insert(*a, b_idx);
            nodes[*b].index_channel.push(*a);
        }
        let mut fast_channels = Vec::new();
        for distributed_uf_decoder::InputFastChannel {a, b, latency: _} in decoder.input_fast_channels.iter() {
            fast_channels.push(DufFastChannel {
                a: *a,
                b: *b,
            });
            let a_idx = nodes[*a].index_channel.len();
            nodes[*a].channel_index.insert(*b, a_idx);
            nodes[*a].index_channel.push(*b);
            let b_idx = nodes[*b].index_channel.len();
            nodes[*b].channel_index.insert(*a, b_idx);
            nodes[*b].index_channel.push(*a);
        }
        // update neighbor_count
        for node in nodes.iter_mut() {
            node.neighbor_count = node.index_channel.len();
        }
        Self {
            module_name: format!("module_name"),
            distance_solver_name: format!("tree_distance_2d_solver"),
            dimension: 3,
            nodes: nodes,
            neighbors: neighbors,
            fast_channels: fast_channels,
        }
    }
}

/**
default example:
`cargo run -- fpga_generator perfect_measurement_distributed_union_find 3`
**/
fn perfect_measurement_distributed_union_find(d: usize) {
    let (nodes, _position_to_index, neighbors) = union_find_decoder::make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
    let decoder = union_find_decoder::UnionFindDecoder::new(nodes, neighbors);
    let duf = DistributedUnionFind::<DufNode2d>::from_union_find_decoder(&decoder);
    println!("{}", duf.generate_code());
}

/**
default example:
`cargo run --release -- fpga_generator fault_tolerant_distributed_union_find 5`
**/
fn fault_tolerant_distributed_union_find(d: usize, measurement_rounds: usize, p: f64, autotune: bool, fast_channel_interval: usize) {
    let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(measurement_rounds, d);
    if autotune {
        model.set_depolarizing_error_with_perfect_initialization(p);
    } else {
        model.set_phenomenological_error_with_perfect_initialization(p);
    }
    model.build_graph();
    let (nodes, _position_to_index, neighbors, fast_channels) = distributed_uf_decoder::make_decoder_given_ftqec_model(&model, QubitType::StabZ, fast_channel_interval);
    let decoder = distributed_uf_decoder::DistributedUnionFind::new(nodes, neighbors, fast_channels,
        distributed_uf_decoder::manhattan_distance_standard_planar_code_3d_nodes, distributed_uf_decoder::compare_standard_planar_code_3d_nodes);
    let duf = DistributedUnionFind::<DufNode3d>::from_distributed_union_find_decoder(&decoder);
    println!("{}", duf.generate_code());
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::pyo3::types::{PyDict};

    // use `cargo test fpga_generator_test_basic_python_call -- --nocapture` to run specific test

    #[test]
    fn fpga_generator_test_basic_python_call() {
        Python::with_gil(|py| {
            (|py: Python| -> PyResult<()> {
                // find source file first
                let py_module_name = format!("test_call_python_from_rust");
                let lib_filename = format!("{}.py", py_module_name);
                let lib_source = getFileContentFromMultiplePlaces(&PY_SEARCH_DIRECTORIES.read().unwrap(), &lib_filename).unwrap();
                let module = PyModule::from_code(py, lib_source.as_str(), lib_filename.as_str(), py_module_name.as_str())?;
                // run it as module
                let hello_world_ret: String = module.getattr("hello_world")?.call0()?.extract()?;
                println!("hello_world_ret: {}", hello_world_ret);
                let is_main: bool = module.getattr("is_main")?.extract()?;
                println!("is_main: {}", is_main);
                // or run it as main
                let locals = PyDict::new(py);
                py.run(&lib_source, None, Some(locals))?;
                let hello_world = locals.get_item("hello_world").expect("exist");
                let hello_world_ret: String = hello_world.call0()?.extract()?;
                println!("hello_world_ret: {}", hello_world_ret);
                let is_main: bool = locals.get_item("is_main").expect("exist").extract()?;
                println!("is_main: {}", is_main);
                Ok(())
            })(py).map_err(|e| {
                e.print_and_set_sys_last_vars(py);
            })
        }).expect("python run failed")
    }

}
