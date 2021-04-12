//! # FPGA Generator
//!
//! This module aims at generating Verilog code automatically.

#![allow(non_snake_case)]  // allow non snake case for variables in Verilog code

use super::union_find_decoder;
use super::ftqec;
use super::types::ErrorType;
use super::types::QubitType;
use super::indoc::formatdoc;
use std::collections::{HashMap};

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
    /// 2 for perfect measurement or 3 for imperfect measurement
    pub dimensions: usize,
    pub nodes: Vec<N>,
    pub neighbors: Vec<DufNeighbor>,
    pub fast_channels: Vec<DufFastChannel>,
}

pub trait DufNode {
    /// printable name, consist of only a-z0-9_
    fn name(&self) -> String;
    fn maximum_coordinate(&self) -> usize;
    fn boundary_cost(&self) -> Option<usize>;
    fn is_error_syndrome(&self) -> bool;
    fn get_neighbor_index(&self, neighbor: usize) -> usize;
    fn channel_count(&self) -> usize;
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
        code.push_str(&formatdoc!(r###"
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
        code.push_str(&formatdoc!(r###"

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
        code.push_str(&formatdoc!(r###"

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
            // instant compare solver
            let CHANNEL_COUNT = node.channel_count();
            code.push_str(&formatdoc!(r###"

// {0}
localparam {0}_CHANNEL_COUNT = {1};
wire [ADDRESS_WIDTH-1:0] {0}_compare_solver_default_addr;
wire [(ADDRESS_WIDTH * {0}_CHANNEL_COUNT)-1:0] {0}_compare_solver_addrs;
wire [{0}_CHANNEL_COUNT-1:0] {0}_compare_solver_addrs_valid;
wire [ADDRESS_WIDTH-1:0] {0}_compare_solver_result;
            "###, prefix, CHANNEL_COUNT));
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
    pub neighbor_index: HashMap<usize, usize>,
}

impl DufNode for DufNode2d {
    fn name(&self) -> String {
        format!("duf_2d_{}_{}", self.origin.0, self.origin.1)
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
    fn get_neighbor_index(&self, neighbor: usize) -> usize {
        self.neighbor_index[&neighbor]
    }
    fn channel_count(&self) -> usize {
        self.neighbor_index.len()
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
                neighbor_index: HashMap::new(),
            }
        }).collect();
        let mut neighbors = Vec::new();
        for union_find_decoder::NeighborEdge {a, b, increased: _, length} in decoder.input_neighbors.iter() {
            neighbors.push(DufNeighbor {
                a: *a,
                b: *b,
                length: *length,
            });
            let a_idx = nodes[*a].neighbor_index.len();
            nodes[*a].neighbor_index.insert(*b, a_idx);
            let b_idx = nodes[*b].neighbor_index.len();
            nodes[*b].neighbor_index.insert(*a, b_idx);
        }
        Self {
            module_name: format!("module_name"),
            dimensions: 2,
            nodes: nodes,
            neighbors: neighbors,
            fast_channels: Vec::new(),  // no fast channels if built from union find decoder
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
