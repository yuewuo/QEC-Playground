import math

class DistributedUnionFind:
    def __init__(self, module_name, distance_solver_name, dimension, nodes, neighbors, fast_channels):
        self.module_name = module_name
        self.distance_solver_name = distance_solver_name
        self.dimension = dimension
        self.nodes = nodes
        self.neighbors = neighbors
        self.fast_channels = fast_channels
    def generate_code(self, verbose = False):
        code = ""
        maximum_coordinate = 0
        for node in self.nodes:
            maximum_coordinate = max(maximum_coordinate, node.maximum_coordinate())
        PU_COUNT = len(self.nodes)
        PER_DIMENSION_WIDTH = int(math.ceil(math.log2(maximum_coordinate + 1)))
        ADDRESS_WIDTH = self.dimension * PER_DIMENSION_WIDTH
        DISTANCE_WIDTH = int(math.ceil(math.log2((maximum_coordinate + 1) * self.dimension)))
        if verbose:
            print(f"PU_COUNT: {PU_COUNT}")
            print(f"PER_DIMENSION_WIDTH: {PER_DIMENSION_WIDTH}")
            print(f"ADDRESS_WIDTH: {ADDRESS_WIDTH}")
            print(f"DISTANCE_WIDTH: {DISTANCE_WIDTH}")
        code += f"""\
`timescale 1ns / 1ps

module {self.module_name} (
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

"""
        # parameters and interface
        code += f"""\
localparam PU_COUNT = {PU_COUNT};
localparam PER_DIMENSION_WIDTH = {PER_DIMENSION_WIDTH};
localparam ADDRESS_WIDTH = {ADDRESS_WIDTH};
localparam DISTANCE_WIDTH = {DISTANCE_WIDTH};
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

"""
        # global logic
        code += f"""\
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

"""
        # instantiate PUs
        for node in self.nodes:
            prefix = node.name()
            NEIGHBOR_COUNT = node.get_neighbor_count()
            CHANNEL_COUNT = node.get_channel_count()
            code += f"""\
// {prefix}  address: {node.get_address(PER_DIMENSION_WIDTH)}
// instantiate compare solver
localparam {prefix}_CHANNEL_COUNT = {CHANNEL_COUNT};
localparam {prefix}_CHANNEL_WIDTH = $clog2({prefix}_CHANNEL_COUNT);
localparam {prefix}_NEIGHBOR_COUNT = {NEIGHBOR_COUNT};
wire [ADDRESS_WIDTH-1:0] {prefix}_compare_solver_default_addr;
wire [(ADDRESS_WIDTH * {prefix}_CHANNEL_COUNT)-1:0] {prefix}_compare_solver_addrs;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_compare_solver_addrs_valid;
wire [ADDRESS_WIDTH-1:0] {prefix}_compare_solver_result;
tree_compare_solver #(
    .DATA_WIDTH(ADDRESS_WIDTH),
    .CHANNEL_COUNT({prefix}_CHANNEL_COUNT)
) u_tree_compare_solver (
    .default_value({prefix}_compare_solver_default_addr),
    .values({prefix}_compare_solver_addrs),
    .valids({prefix}_compare_solver_addrs_valid),
    .result({prefix}_compare_solver_result)
);
// instantiate distance solver
wire [ADDRESS_WIDTH-1:0] {prefix}_distance_solver_target;
wire [{prefix}_CHANNEL_WIDTH-1:0] {prefix}_distance_solver_result_idx;
wire [(ADDRESS_WIDTH * {prefix}_CHANNEL_COUNT)-1:0] {prefix}_channel_addresses;
"""
            # connect addresses of both neighbors and fast channels
            for i in range(CHANNEL_COUNT):
                neighbor = node.index_2_channel(i)
                code += f"assign `SLICE_ADDRESS_VEC({prefix}_channel_addresses, {i}) = {self.nodes[neighbor].get_address(PER_DIMENSION_WIDTH)};\n"
            code += f"""\
{self.distance_solver_name} #(
    .PER_DIMENSION_WIDTH(PER_DIMENSION_WIDTH),
    .CHANNEL_COUNT({prefix}_CHANNEL_COUNT)
) u_tree_distance_2d_solver (
    .points({prefix}_channel_addresses),
    .target({prefix}_distance_solver_target),
    .result_idx({prefix}_distance_solver_result_idx)
);
"""
            BOUNDARY_COST = node.get_boundary_cost() or 0
            code += f"""\
// instantiate processing unit
localparam {prefix}_BOUNDARY_COST = {BOUNDARY_COST};
wire [ADDRESS_WIDTH-1:0] {prefix}_init_address;
assign {prefix}_init_address = {node.get_address(PER_DIMENSION_WIDTH)};
wire [{prefix}_NEIGHBOR_COUNT-1:0] {prefix}_neighbor_is_fully_grown;
wire [(ADDRESS_WIDTH * {prefix}_NEIGHBOR_COUNT)-1:0] {prefix}_neighbor_old_roots;
wire {prefix}_neighbor_increase;
wire [(UNION_MESSAGE_WIDTH * {prefix}_CHANNEL_COUNT)-1:0] {prefix}_union_out_channels_data;
wire {prefix}_union_out_channels_valid;
wire [(UNION_MESSAGE_WIDTH * {prefix}_CHANNEL_COUNT)-1:0] {prefix}_union_in_channels_data;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_union_in_channels_valid;
wire [DIRECT_MESSAGE_WIDTH-1:0] {prefix}_direct_out_channels_data_single;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_direct_out_channels_valid;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_direct_out_channels_is_full;
wire [(DIRECT_MESSAGE_WIDTH * {prefix}_CHANNEL_COUNT)-1:0] {prefix}_direct_in_channels_data;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_direct_in_channels_valid;
wire [{prefix}_CHANNEL_COUNT-1:0] {prefix}_direct_in_channels_is_taken;
wire [ADDRESS_WIDTH-1:0] {prefix}_old_root;
processing_unit #(
    .ADDRESS_WIDTH(ADDRESS_WIDTH),
    .DISTANCE_WIDTH(DISTANCE_WIDTH),
    .BOUNDARY_WIDTH($clog2({prefix}_BOUNDARY_COST + 1)),
    .NEIGHBOR_COUNT({prefix}_NEIGHBOR_COUNT),
    .FAST_CHANNEL_COUNT(FAST_CHANNEL_COUNT)
) u_processing_unit (
    .clk(clk),
    .reset(reset),
    .init_address({prefix}_init_address),
    .init_is_error_syndrome(`init_is_error_syndrome(i, j)),
    .init_has_boundary(`init_has_boundary(i, j)),
    .init_boundary_cost({prefix}_BOUNDARY_COST),
    .stage_in(stage),
    .compare_solver_default_addr({prefix}_compare_solver_default_addr),
    .compare_solver_addrs({prefix}_compare_solver_addrs),
    .compare_solver_addrs_valid({prefix}_compare_solver_addrs_valid),
    .compare_solver_result({prefix}_compare_solver_result),
    .distance_solver_target({prefix}_distance_solver_target),
    .distance_solver_result_idx({prefix}_distance_solver_result_idx),
    .neighbor_is_fully_grown({prefix}_neighbor_is_fully_grown),
    .neighbor_old_roots({prefix}_neighbor_old_roots),
    .neighbor_increase({prefix}_neighbor_increase),
    .union_out_channels_data({prefix}_union_out_channels_data),
    .union_out_channels_valid({prefix}_union_out_channels_valid),
    .union_in_channels_data({prefix}_union_in_channels_data),
    .union_in_channels_valid({prefix}_union_in_channels_valid),
    .direct_out_channels_data_single({prefix}_direct_out_channels_data_single),
    .direct_out_channels_valid({prefix}_direct_out_channels_valid),
    .direct_out_channels_is_full({prefix}_direct_out_channels_is_full),
    .direct_in_channels_data({prefix}_direct_in_channels_data),
    .direct_in_channels_valid({prefix}_direct_in_channels_valid),
    .direct_in_channels_is_taken({prefix}_direct_in_channels_is_taken),
    .old_root({prefix}_old_root),
    .updated_root(`roots(i, j)),
    .is_odd_cluster(`is_odd_cluster(i, j)),
    .is_odd_cardinality(`is_odd_cardinality(i, j))
);

"""
        # create neighbor links
        for neighbor in self.neighbors:
            a_node = self.nodes[neighbor.a]
            a_prefix = a_node.name()
            a_index = a_node.channel_2_index(neighbor.b)
            b_node = self.nodes[neighbor.b]
            b_prefix = b_node.name()
            b_index = b_node.channel_2_index(neighbor.a)
            code += f"""\
neighbor_link #(.LENGTH({neighbor.length}), .ADDRESS_WIDTH(ADDRESS_WIDTH)) {a_prefix}_and_{b_prefix}_neighbor_link (
    .clk(clk), .reset(reset), .initialize(initialize_neighbors), .is_fully_grown({a_prefix}_neighbor_is_fully_grown[{a_index}]),
    .a_old_root_in({a_prefix}_old_root), .a_increase({a_prefix}_neighbor_increase),
    .b_old_root_out(`SLICE_ADDRESS_VEC({a_prefix}_neighbor_old_roots, {a_index})),
    .b_old_root_in({b_prefix}_old_root), .b_increase({b_prefix}_neighbor_increase),
    .a_old_root_out(`SLICE_ADDRESS_VEC({b_prefix}_neighbor_old_roots, {b_index}))
);
assign {b_prefix}_neighbor_is_fully_grown[{b_index}] = {a_prefix}_neighbor_is_fully_grown[{a_index}];
"""
            for (source, target) in [(neighbor.a, neighbor.b), (neighbor.b, neighbor.a)]:
                source_node = self.nodes[source]
                source_prefix = source_node.name()
                source_index = source_node.channel_2_index(target)
                target_node = self.nodes[target]
                target_prefix = target_node.name()
                target_index = target_node.channel_2_index(source)
                code += f"""\
nonblocking_channel #(.WIDTH(UNION_MESSAGE_WIDTH)) {source_prefix}_to_{target_prefix}_nonblocking_channel (
    .clk(clk), .reset(reset), .initialize(initialize_neighbors),
    .in_data(`SLICE_UNION_MESSAGE_VEC({source_prefix}_union_out_channels_data, {source_index})),
    .in_valid({source_prefix}_union_out_channels_valid),
    .out_data(`SLICE_UNION_MESSAGE_VEC({target_prefix}_union_in_channels_data, {target_index})),
    .out_valid({target_prefix}_union_in_channels_valid[{target_index}])
);
blocking_channel #(.WIDTH(DIRECT_MESSAGE_WIDTH)) {source_prefix}_to_{target_prefix}_blocking_channel (
    .clk(clk), .reset(reset), .initialize(initialize_neighbors), 
    .in_data(`{source_prefix}_direct_out_channels_data_single),
    .in_valid(`{source_prefix}_direct_out_channels_valid[{source_index}]),
    .in_is_full(`{source_prefix}_direct_out_channels_is_full[{source_index}]),
    .out_data(`SLICE_DIRECT_MESSAGE_VEC(`{target_prefix}_direct_in_channels_data, {target_index})),
    .out_valid(`{target_prefix}_direct_in_channels_valid[{target_index}]),
    .out_is_taken(`{target_prefix}_direct_in_channels_is_taken[{target_index}])
);
"""
            code += "\n"
        return code

class DufNode:
    def __init__(self, address, origin, boundary_cost, is_error_syndrome, channel_index, index_channel, neighbor_count):
        # address: (usize, usize)
        assert isinstance(address, (list, tuple))
        for e in address:
            assert isinstance(e, int)
        self.address = address
        self.dimension = len(address)
        # origin: (usize, usize)
        assert isinstance(origin, (list, tuple))
        for e in origin:
            assert isinstance(e, int)
        assert len(origin) == self.dimension, "dimension should be same"
        self.origin = origin
        # init_boundary_cost: Option<usize>
        assert boundary_cost is None or isinstance(boundary_cost, int)
        self.boundary_cost = boundary_cost
        # init_is_error_syndrome: bool
        assert isinstance(is_error_syndrome, bool)
        self.is_error_syndrome = is_error_syndrome
        # channel_index: HashMap<usize, usize>
        assert isinstance(channel_index, dict)
        for e in channel_index.keys():
            assert isinstance(e, int)
        for e in channel_index.values():
            assert isinstance(e, int)
        self.channel_index = channel_index
        # index_channel: Vec<usize>
        assert isinstance(index_channel, list)
        for e in index_channel:
            assert isinstance(e, int)
        self.index_channel = index_channel
        # neighbor_count: usize
        assert isinstance(neighbor_count, int)
        self.neighbor_count = neighbor_count
    def name(self):  # str
        return f"duf{self.dimension}d_{'_'.join([str(e) for e in self.origin])}"
    def get_address(self, per_dimension_width):  # str
        width_decorate = "'" + str(per_dimension_width)
        return f"{{ {', '.join([str(e) + width_decorate for e in self.address])} }}"
    def maximum_coordinate(self):  # int
        return max(self.address)
    def get_boundary_cost(self):  # int or None
        return self.boundary_cost
    def get_is_error_syndrome(self):  # bool
        return self.is_error_syndrome
    def get_neighbor_count(self):  # int
        return self.neighbor_count
    def get_channel_count(self):  # int
        return len(self.channel_index)
    def index_2_channel(self, index):  # int
        return self.index_channel[index]
    def channel_2_index(self, neighbor):  # int
        return self.channel_index[neighbor]

class DufNeighbor:
    def __init__(self, a, b, length):
        self.a = a
        self.b = b
        self.length = length

class DufFastChannel:
    def __init__(self, a, b):
        self.a = a
        self.b = b

if __name__ == "__main__":
    module_name = "duf2d_tester_d3"
    distance_solver_name = "tree_distance_2d_solver"
    dimension = 2
    nodes = [DufNode(
        address = (0, 0),
        origin = (12, 15),
        boundary_cost = None,
        is_error_syndrome = False,
        channel_index = { 1: 0 },
        index_channel = [1],
        neighbor_count = 1,
    ), DufNode(
        address = (0, 1),
        origin = (33, 35),
        boundary_cost = 2,
        is_error_syndrome = True,
        channel_index = { 0: 0 },
        index_channel = [0],
        neighbor_count = 1,
    )]
    neighbors = [
        DufNeighbor(a = 0, b = 1, length = 2),
    ]
    fast_channels = []
    duf = DistributedUnionFind(
        module_name = module_name,
        distance_solver_name = distance_solver_name,
        dimension = dimension,
        nodes = nodes,
        neighbors = neighbors,
        fast_channels = fast_channels,
    )
    print(duf.generate_code(True))
