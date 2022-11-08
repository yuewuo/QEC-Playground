/*
 * Example of Generated Top Module
 *
 * This should compile using [verilator](https://github.com/verilator/verilator)
 * Documentation is at: https://verilator.org/guide/latest/verilating.html
 *
 */

`timescale 1ns / 10ps

module generated_top_example(
    input reg clk,
    input reg reset,
    output reg [31:0] count_c
);

`include "parameters.sv"

always@(posedge clk) begin
    if (reset) begin
        count_c <= 0;
    end else begin
        count_c <= count_c + 1;
    end
end

endmodule
