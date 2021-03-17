//! # Distributed UnionFind Decoder
//!
//! ## Introduction
//!
//! UnionFind decoder has good accuracy and computational complexity when running on CPU, which is in worst case $O(n α(n))$.
//! In a running quantum computer, the number of errors $n$ that are actively concerned in every round is $O(d^3)$, given the code distance $d$.
//! Suppose every fault tolerant operation requires O(d) rounds, that means we need to solve $n = O(d^3)$ errors in O(d) time.
//! This latency requirement is much stricter than the currently sequential implementation of UnionFind decoder, which is about $O(d^3)$ over the requirement of $O(d)$.
//!
//! We need to design a distributed UnionFind decoder to fit into the timing constraint.
//! This means we need to solve $O(d^3)$ errors in as much close to O(d) time as possible.
//! In this work, we propose a $O(d \log{d})$ average time distributed UnionFind decoder implemented on FPGA(s).
//!
//! ## Background
//!
//! ### Union Find Decoder
//! UnionFind decoder for topological quantum error correction (TQEC) codes is one of the currently most practical decoders both in accuracy and time complexity.
//! It requires at most $O(d)$ iterations, in each iteration the exploratory region of each odd cluster grows.
//! This growing cluster requires a tracking of the disjoint sets, which is extremely efficient using Union Find algorithm.
//! After analyzing each steps in the sequential UnionFind decoder, we found that the Union Find solver is the main challenge that blocks a low-latency distributed version.
//!
//! ### Parallel Union Find Solver
//! There exists some works for parallel Union-Find algorithm, e.g. [arXiv:2003.02351](https://arxiv.org/pdf/2003.02351.pdf),
//!     [arXiv:1710.02260](https://arxiv.org/pdf/1710.02260.pdf).
//! But none of them applies to our requirement direction, which is nano-second level latency with at least $O(d^2)$ concurrent requests needed.
//!
//! ## Design
//!
//! Instead of seeking for a general distributed Union Find algorithm, we try to improve the Union Find performance by exploiting the attributes of TQEC codes.
//! The main property is that, the interactions of the stabilizers are local, meaning that two stabilizers have direct connection only if they're neighbors in the space.
//! Thus, the disjoint set during the execution of UF decoder has an attribute that it's spreading in the space, which has a longest spreading path of length $d$.
//!
//! A naive design would be spreading the root of the disjoint set in the graph.
//! When a union operation should apply to two disjoint sets, the root is updated to the smallest root.
//! This is not considered optimal in sequential union-find algorithms, actually they use rank-based or weight-based merging to improve performance.
//! In our case, however, since the root must be spread to all nodes, which takes O(d) worst case bound, a fixed rule of root selection
//!      (so that node can choose the updated root without querying the root's internal state) is more important than reducing the number of updated nodes.
//! This operation is totally distributed, as merging union will ultimately be updated to the smallest root, although some intermediate state has invalid root.
//!
//! The naive design has a strict $O(d)$ worst case bound for each iteration, and the number of iteration is strictly $d$.
//! Thus, the total complexity is $O(d^2)$, which is growing faster than the time budget of $O(d)$.
//! To solve this gap, we propose a optimized version of distributed UF decoder that still has $O(d^2)$ worst case bound but the average complexity reduces to $O(d\log{d})$.
//!
//! The optimization originates from the key observation that the time is spending on spreading the updated root from one side to the very far end.
//! If we can send the updated root directly from one side to all other nodes, then the problem solves in $O(1)$ strict time bound.
//! But this is problematic in that it requires a complete connection between every two nodes, introducing $O(d^6)$ connections which is not scalable in hardware.
//! To balance between hardware complexity and time complexity, we try to add connections more cleverly.
//! We add connections to a pair of nodes if they're at exact distance of 2, 4, 8, 16, ··· in one dimension and also must be identical in the other dimensions.
//! For example, in a 2D arranged nodes (figure below), the <span style="color: red;">red</span> node connects to the <span style="color: blue;">blue</span> nodes.
//! Every node connects to $O(\log{d})$ other nodes in the optimized design, instead of $O(1)$ in the naive design.
//! This overhead is scalable with all practical code distances, and this will reduce the longest path from $O(d)$ to $O(\log{d})$.
//!
//! <div style="width: 100%; display: flex; justify-content: center;"><svg id="distributed_uf_decoder_connections_2D_demo" style="width: 300px; height: 300px;" viewBox="0 0 100 100"></svg></div>
//! <script>function draw_distributed_uf_decoder_connections_2D_demo(){let t=document.getElementById("distributed_uf_decoder_connections_2D_demo");if(!t)return;const e=parseInt(10.5);function r(t){for(;1!=t;){if(t%2!=0)return!1;t/=2}return!0}for(let i=0;i<21;++i)for(let n=0;n<20;++n){const o=(n+1.5)*(100/22),c=(i+1)*(100/22);let u=document.createElementNS("http://www.w3.org/2000/svg","circle");u.setAttribute("cx",o),u.setAttribute("cy",c),u.setAttribute("r",100/22*.3),u.setAttribute("fill","rgb(0, 0, 0)"),i==e&&n==e?u.setAttribute("fill","rgb(255, 0, 0)"):(i==e&&r(Math.abs(n-e))||n==e&&r(Math.abs(i-e)))&&u.setAttribute("fill","rgb(0, 0, 255)"),t.appendChild(u)}}document.addEventListener("DOMContentLoaded", draw_distributed_uf_decoder_connections_2D_demo)</script>
//! 
//! The worst case bound of the optimized design seems to be $O(d \log{d})$ at the first glance, but this isn't true when coming to a practical distributed implementation.
//! Considering the format of the messages passing through those connections, it's different from the naive design in that the node cannot easily know
//!     whether the receiver is in the same disjoint set as the sender.
//! It's better to let the receiver to decide whether it should respond to the message, to avoid some inconsistent state sharing.
//!