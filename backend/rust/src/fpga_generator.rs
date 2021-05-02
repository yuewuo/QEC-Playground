//! # FPGA Generator
//!
//! This module aims at generating Verilog code automatically.

#![allow(non_snake_case)]  // allow non snake case for variables in Verilog code

use super::union_find_decoder;
use super::ftqec;
use super::types::ErrorType;
use super::types::QubitType;
use super::pyo3::prelude::*;
use super::pyo3::types::{IntoPyDict, PyDict};
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
            fault_tolerant_distributed_union_find(d, measurement_rounds, p, autotune);
        }
        _ => unreachable!()
    }
}

pub struct DistributedUnionFind<N: DufNode> {
    pub module_name: String,
    pub distance_solver_name: String,
    /// 2 for perfect measurement or 3 for imperfect measurement
    pub dimensions: usize,
    pub nodes: Vec<N>,
    pub neighbors: Vec<DufNeighbor>,
    pub fast_channels: Vec<DufFastChannel>,
}

pub trait DufNode {
    /// printable name, consist of only a-z0-9_
    fn name(&self) -> String;
    fn get_address(&self, per_dimension_width: usize) -> String;
    fn maximum_coordinate(&self) -> usize;
    fn boundary_cost(&self) -> Option<usize>;
    fn is_error_syndrome(&self) -> bool;
    fn get_neighbor_count(&self) -> usize;
    fn get_channel_count(&self) -> usize;
    fn index_2_channel(&self, index: usize) -> usize;
    fn channel_2_index(&self, neighbor: usize) -> usize;
}

impl<N: DufNode> DistributedUnionFind<N> {
    pub fn generate_code(&self) -> String {
        let mut code = format!("");
        let mut maximum_coordinate = 0;
        for node in self.nodes.iter() {
            maximum_coordinate = std::cmp::max(maximum_coordinate, node.maximum_coordinate());
        }
        let PU_COUNT = self.nodes.len();
        let PER_DIMENSION_WIDTH = ((maximum_coordinate + 1) as f64).log2().ceil() as usize;
        let ADDRESS_WIDTH = self.dimensions * PER_DIMENSION_WIDTH;
        let DISTANCE_WIDTH = (((maximum_coordinate + 1) * self.dimensions) as f64).log2().ceil() as usize;
        // head
        code.push_str(&format!(r###"
`timescale 1ns / 1ps

module {0} (
    clk,
    reset,
    stage,
    is_error_syndromes,
    is_odd_clusters,
    is_odd_cardinalities,
    roots,
    has_message_flying
);

`include "parameters.sv"

"###, self.module_name));
        // parameters and interface
        code.push_str(&format!(r###"
localparam PU_COUNT = {0};
localparam PER_DIMENSION_WIDTH = {1};
localparam ADDRESS_WIDTH = {2};
localparam DISTANCE_WIDTH = {3};
// localparam WEIGHT = 1;  // the weight in MWPM graph
// localparam BOUNDARY_COST = 2 * WEIGHT;
// localparam NEIGHBOR_COST = 2 * WEIGHT;
// localparam BOUNDARY_WIDTH = $clog2(BOUNDARY_COST + 1);
localparam UNION_MESSAGE_WIDTH = 2 * ADDRESS_WIDTH;  // [old_root, updated_root]
localparam DIRECT_MESSAGE_WIDTH = ADDRESS_WIDTH + 1 + 1;  // [receiver, is_odd_cardinality_root, is_touching_boundary]
`define SLICE_ADDRESS_VEC(vec, idx) (vec[(((idx)+1)*ADDRESS_WIDTH)-1:(idx)*ADDRESS_WIDTH])
`define SLICE_UNION_MESSAGE_VEC(vec, idx) (vec[(((idx)+1)*UNION_MESSAGE_WIDTH)-1:(idx)*UNION_MESSAGE_WIDTH])
`define SLICE_DIRECT_MESSAGE_VEC(vec, idx) (vec[(((idx)+1)*DIRECT_MESSAGE_WIDTH)-1:(idx)*DIRECT_MESSAGE_WIDTH])

input clk;
input reset;
input [STAGE_WIDTH-1:0] stage;
input [PU_COUNT-1:0] is_error_syndromes;
output [PU_COUNT-1:0] is_odd_clusters;
output [PU_COUNT-1:0] is_odd_cardinalities;
output [(ADDRESS_WIDTH * PU_COUNT)-1:0] roots;
output has_message_flying;
wire [PU_COUNT-1:0] has_message_flyings;
reg [PU_COUNT-1:0] has_message_flyings_reg;
wire initialize_neighbors;
reg [STAGE_WIDTH-1:0] stage_internal;

"###, PU_COUNT, PER_DIMENSION_WIDTH, ADDRESS_WIDTH, DISTANCE_WIDTH));
        // global logic
        code.push_str(&format!(r###"
assign has_message_flying = |has_message_flyings_reg;

always@(posedge clk) begin
    has_message_flyings_reg <= has_message_flyings;
end

// this is to emualte the delay in the PUs
always @(posedge clk) begin
    if (reset) begin
        stage_internal <= STAGE_IDLE;
    end else begin
        stage_internal <= stage;
    end
end

assign initialize_neighbors = (stage_internal == STAGE_MEASUREMENT_LOADING);

"###));
        // instantiate PUs
        for node in self.nodes.iter() {
            let prefix = format!("{}", node.name());
            let NEIGHBOR_COUNT = node.get_neighbor_count();
            let CHANNEL_COUNT = node.get_channel_count();
            let NEIGHBOR_COUNT = node.get_neighbor_count();
            code.push_str(&format!(r###"
// {0}  address: {2}
// instantiate compare solver
localparam {0}_CHANNEL_COUNT = {1};
localparam {0}_CHANNEL_WIDTH = $clog2({0}_CHANNEL_COUNT);
localparam {0}_NEIGHBOR_COUNT = {3};
wire [ADDRESS_WIDTH-1:0] {0}_compare_solver_default_addr;
wire [(ADDRESS_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_compare_solver_addrs;
wire [{0}_CHANNEL_COUNT-1:0] {0}_compare_solver_addrs_valid;
wire [ADDRESS_WIDTH-1:0] {0}_compare_solver_result;
tree_compare_solver #(
    .DATA_WIDTH(ADDRESS_WIDTH),
    .CHANNEL_COUNT({0}_CHANNEL_COUNT)
) u_tree_compare_solver (
    .default_value({0}_compare_solver_default_addr),
    .values({0}_compare_solver_addrs),
    .valids({0}_compare_solver_addrs_valid),
    .result({0}_compare_solver_result)
);
// instantiate distance solver
wire [ADDRESS_WIDTH-1:0] {0}_distance_solver_target;
wire [{0}_CHANNEL_WIDTH-1:0] {0}_distance_solver_result_idx;
wire [(ADDRESS_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_channel_addresses;
"###, prefix, CHANNEL_COUNT, node.get_address(PER_DIMENSION_WIDTH), NEIGHBOR_COUNT));
            // connect addresses of both neighbors and fast channels
            for i in 0..CHANNEL_COUNT {
                let neighbor = node.index_2_channel(i);
                code.push_str(&format!("assign `SLICE_ADDRESS_VEC({0}_channel_addresses, {1}) = {2};\n"
                    , prefix, i, self.nodes[neighbor].get_address(PER_DIMENSION_WIDTH)));
            }
            code.push_str(&format!(r###"
{1} #(
    .PER_DIMENSION_WIDTH(PER_DIMENSION_WIDTH),
    .CHANNEL_COUNT({0}_CHANNEL_COUNT)
) u_tree_distance_2d_solver (
    .points({0}_channel_addresses),
    .target({0}_distance_solver_target),
    .result_idx({0}_distance_solver_result_idx)
);
"###, prefix, self.distance_solver_name));
            let BOUNDARY_COST = match node.boundary_cost() {
                Some(cost) => cost,
                None => 0,
            };
            code.push_str(&format!(r###"
// instantiate processing unit
localparam {0}_BOUNDARY_COST = {2};
wire [ADDRESS_WIDTH-1:0] {0}_init_address;
assign {0}_init_address = {1};
wire [{0}_NEIGHBOR_COUNT-1:0] {0}_neighbor_is_fully_grown;
wire [(ADDRESS_WIDTH * {0}_NEIGHBOR_COUNT)-1:0] {0}_neighbor_old_roots;
wire {0}_neighbor_increase;
wire [(UNION_MESSAGE_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_union_out_channels_data;
wire {0}_union_out_channels_valid;
wire [(UNION_MESSAGE_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_union_in_channels_data;
wire [{0}_CHANNEL_COUNT-1:0] {0}_union_in_channels_valid;
wire [DIRECT_MESSAGE_WIDTH-1:0] {0}_direct_out_channels_data_single;
wire [{0}_CHANNEL_COUNT-1:0] {0}_direct_out_channels_valid;
wire [{0}_CHANNEL_COUNT-1:0] {0}_direct_out_channels_is_full;
wire [(DIRECT_MESSAGE_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_direct_in_channels_data;
wire [{0}_CHANNEL_COUNT-1:0] {0}_direct_in_channels_valid;
wire [{0}_CHANNEL_COUNT-1:0] {0}_direct_in_channels_is_taken;
wire [ADDRESS_WIDTH-1:0] {0}_old_root;
processing_unit #(
    .ADDRESS_WIDTH(ADDRESS_WIDTH),
    .DISTANCE_WIDTH(DISTANCE_WIDTH),
    .BOUNDARY_WIDTH($clog2({0}_BOUNDARY_COST + 1)),
    .NEIGHBOR_COUNT({0}_NEIGHBOR_COUNT),
    .FAST_CHANNEL_COUNT(FAST_CHANNEL_COUNT)
) u_processing_unit (
    .clk(clk),
    .reset(reset),
    .init_address({0}_init_address),
    .init_is_error_syndrome(`init_is_error_syndrome(i, j)),
    .init_has_boundary(`init_has_boundary(i, j)),
    .init_boundary_cost({0}_BOUNDARY_COST),
    .stage_in(stage),
    .compare_solver_default_addr({0}_compare_solver_default_addr),
    .compare_solver_addrs({0}_compare_solver_addrs),
    .compare_solver_addrs_valid({0}_compare_solver_addrs_valid),
    .compare_solver_result({0}_compare_solver_result),
    .distance_solver_target({0}_distance_solver_target),
    .distance_solver_result_idx({0}_distance_solver_result_idx),
    .neighbor_is_fully_grown({0}_neighbor_is_fully_grown),
    .neighbor_old_roots({0}_neighbor_old_roots),
    .neighbor_increase({0}_neighbor_increase),
    .union_out_channels_data({0}_union_out_channels_data),
    .union_out_channels_valid({0}_union_out_channels_valid),
    .union_in_channels_data({0}_union_in_channels_data),
    .union_in_channels_valid({0}_union_in_channels_valid),
    .direct_out_channels_data_single({0}_direct_out_channels_data_single),
    .direct_out_channels_valid({0}_direct_out_channels_valid),
    .direct_out_channels_is_full({0}_direct_out_channels_is_full),
    .direct_in_channels_data({0}_direct_in_channels_data),
    .direct_in_channels_valid({0}_direct_in_channels_valid),
    .direct_in_channels_is_taken({0}_direct_in_channels_is_taken),
    .old_root({0}_old_root),
    .updated_root(`roots(i, j)),
    .is_odd_cluster(`is_odd_cluster(i, j)),
    .is_odd_cardinality(`is_odd_cardinality(i, j))
);

"###, prefix, node.get_address(PER_DIMENSION_WIDTH), BOUNDARY_COST));
        }
        // create neighbor links
        for DufNeighbor { a, b, length } in self.neighbors.iter() {
            let a_node = &self.nodes[*a];
            let a_prefix = format!("{}", a_node.name());
            let a_index = a_node.channel_2_index(*b);
            let b_node = &self.nodes[*b];
            let b_prefix = format!("{}", b_node.name());
            let b_index = b_node.channel_2_index(*a);
            code.push_str(&format!(r###"
neighbor_link #(.LENGTH({4}), .ADDRESS_WIDTH(ADDRESS_WIDTH)) {0}_and_{1}_neighbor_link (\
    .clk(clk), .reset(reset), .initialize(initialize_neighbors), .is_fully_grown({0}_neighbor_is_fully_grown[{2}]),\
    .a_old_root_in({0}_old_root), .a_increase({0}_neighbor_increase),\
    .b_old_root_out(`SLICE_ADDRESS_VEC({0}_neighbor_old_roots, {2})),\
    .b_old_root_in({1}_old_root), .b_increase({1}_neighbor_increase),\
    .a_old_root_out(`SLICE_ADDRESS_VEC({1}_neighbor_old_roots, {3}))\
);\
assign `{1}_neighbor_is_fully_grown[{3}] = {0}_neighbor_is_fully_grown[{2}];
"###, a_prefix, b_prefix, a_index, b_index, length));
            for (source, target) in [(*a, *b), (*b, *a)].iter() {
                let source_node = &self.nodes[*source];
                let source_prefix = format!("{}", source_node.name());
                let source_index = source_node.channel_2_index(*target);
                let target_node = &self.nodes[*target];
                let target_prefix = format!("{}", target_node.name());
                let target_index = target_node.channel_2_index(*source);
                code.push_str(&format!(r###"
nonblocking_channel #(.WIDTH(UNION_MESSAGE_WIDTH)) {0}_to_{1}_nonblocking_channel (
    .clk(clk), .reset(reset), .initialize(initialize_neighbors),
    .in_data(`SLICE_UNION_MESSAGE_VEC({0}_union_out_channels_data, {2})),
    .in_valid({0}_union_out_channels_valid),
    .out_data(`SLICE_UNION_MESSAGE_VEC({1}_union_in_channels_data, {3})),
    .out_valid({1}_union_in_channels_valid[{3}])
);
blocking_channel #(.WIDTH(DIRECT_MESSAGE_WIDTH)) {0}_to_{1}_blocking_channel (
    .clk(clk), .reset(reset), .initialize(initialize_neighbors), 
    .in_data(`{0}_direct_out_channels_data_single),
    .in_valid(`{0}_direct_out_channels_valid[{2}]),
    .in_is_full(`{0}_direct_out_channels_is_full[{2}]),
    .out_data(`SLICE_DIRECT_MESSAGE_VEC(`{1}_direct_in_channels_data, {3})),
    .out_valid(`{1}_direct_in_channels_valid[{3}]),
    .out_is_taken(`{1}_direct_in_channels_is_taken[{3}])
);
"###, source_prefix, target_prefix, source_index, target_index));
            }
        }
        code
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

pub struct DufFastChannel {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
}

pub struct DufNode2d {
    pub address: (usize, usize),  // compressed address, can reduce 1 bit for each axis
    pub origin: (usize, usize),  // original address
    pub init_boundary_cost: Option<usize>,
    pub init_is_error_syndrome: bool,
    pub channel_index: HashMap<usize, usize>,
    pub index_channel: Vec<usize>,
    pub neighbor_count: usize,
}

impl DufNode for DufNode2d {
    fn name(&self) -> String {
        format!("duf2d_{}_{}", self.origin.0, self.origin.1)
    }
    fn get_address(&self, per_dimension_width: usize) -> String {
        format!("{{ {0}'{1}, {0}'{2} }}", per_dimension_width, self.address.0, self.address.1)
    }
    fn boundary_cost(&self) -> Option<usize> {
        self.init_boundary_cost
    }
    fn is_error_syndrome(&self) -> bool {
        self.init_is_error_syndrome
    }
    fn maximum_coordinate(&self) -> usize {
        std::cmp::max(self.address.0, self.address.1)
    }
    fn get_neighbor_count(&self) -> usize {
        self.neighbor_count
    }
    fn get_channel_count(&self) -> usize {
        self.channel_index.len()
    }
    fn index_2_channel(&self, index: usize) -> usize {
        self.index_channel[index]
    }
    fn channel_2_index(&self, neighbor: usize) -> usize {
        self.channel_index[&neighbor]
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
                init_boundary_cost: node.node.boundary_cost,
                init_is_error_syndrome: node.node.is_error_syndrome,
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
            dimensions: 2,
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
fn fault_tolerant_distributed_union_find(d: usize, measurement_rounds: usize, p: f64, autotune: bool) {
    // let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(measurement_rounds, d);
    // if autotune {
    //     model.set_depolarizing_error_with_perfect_initialization(p);
    // } else {
    //     model.set_phenomenological_error_with_perfect_initialization(p);
    // }
    // model.build_graph();
    // let (nodes, _position_to_index, neighbors) = union_find_decoder::make_decoder_given_ftqec_model(&model, QubitType::StabZ);
    // let decoder = union_find_decoder::UnionFindDecoder::new(nodes, neighbors);
    // let duf = DistributedUnionFind::<DufNode2d>::from_union_find_decoder(&decoder);
    // println!("{}", duf.generate_code());
}

#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test fpga_generator_test_basic_python_call -- --nocapture` to run specific test

    #[test]
    fn fpga_generator_test_basic_python_call() {
        Python::with_gil(|py| {
            (|py: Python| -> PyResult<()> {
                // find source file first
                let module_name = format!("test_call_python_from_rust");
                let lib_filename = format!("{}.py", module_name);
                let lib_source = getFileContentFromMultiplePlaces(&PY_SEARCH_DIRECTORIES.read().unwrap(), &lib_filename).unwrap();
                let module = PyModule::from_code(py, lib_source.as_str(), lib_filename.as_str(), module_name.as_str())?;
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
