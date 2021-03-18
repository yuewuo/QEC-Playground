//! # Distributed UnionFind Decoder
//!
//! ## Introduction
//!
//! UnionFind decoder has good accuracy and computational complexity when running on CPU, which is in worst case $O(n α(n))$.
//! In a running quantum computer, the number of errors $n$ that are actively concerned in every round is $O(d^3)$, given the code distance $d$.
//! Suppose every fault tolerant operation requires $O(d)$ rounds, that means we need to solve $n = O(d^3)$ errors in $O(d)$ time.
//! This latency requirement is much stricter than the currently sequential implementation of UnionFind decoder, which is about $O(d^3)$ over the requirement of $O(d)$.
//!
//! We need to design a distributed UnionFind decoder to fit into the timing constraint.
//! This means we need to solve $O(d^3)$ errors in as much close to $O(d)$ time as possible.
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
//! In our case, however, since the root must be spread to all nodes, which takes $O(d)$ worst case bound, a fixed rule of root selection
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
//! We call this a "fast channel architecture".
//!
//! <div style="width: 100%; display: flex; justify-content: center;"><svg id="distributed_uf_decoder_connections_2D_demo" style="width: 300px;" viewBox="0 0 100 100"></svg></div>
//! <script>function draw_distributed_uf_decoder_connections_2D_demo(){let t=document.getElementById("distributed_uf_decoder_connections_2D_demo");if(!t)return;const e=parseInt(10.5);function r(t){for(;1!=t;){if(t%2!=0)return!1;t/=2}return!0}for(let i=0;i<21;++i)for(let n=0;n<20;++n){const o=(n+1.5)*(100/22),c=(i+1)*(100/22);let u=document.createElementNS("http://www.w3.org/2000/svg","circle");u.setAttribute("cx",o),u.setAttribute("cy",c),u.setAttribute("r",100/22*.3),u.setAttribute("fill","rgb(0, 0, 0)"),i==e&&n==e?u.setAttribute("fill","rgb(255, 0, 0)"):(i==e&&r(Math.abs(n-e))||n==e&&r(Math.abs(i-e)))&&u.setAttribute("fill","rgb(0, 0, 255)"),t.appendChild(u)}}document.addEventListener("DOMContentLoaded", draw_distributed_uf_decoder_connections_2D_demo)</script>
//! 
//! The worst case bound of the optimized design seems to be $O(d \log{d})$ at the first glance, but this isn't true when coming to a practical distributed implementation.
//! Considering the format of the messages passing through those connections, it's different from the naive design in that the node cannot easily know
//!     whether the receiver is in the same disjoint set as the sender.
//! It's better to let the receiver to decide whether it should respond to the message, to avoid some inconsistent state sharing.
//! In our design, the message has two field:
//! - the old root of the current node (this old root keeps constant at the beginning of the iteration, updated only at the end of the iteration)
//! - the updated root of the current node
//!
//! When the remote node receives a message, it will drop the message if his old root doesn't match with the old root in the message.
//! This means that they must be in the same disjoint set at the beginning of the iteration, otherwise they won't have a same old root.
//! The worst case comes when there are $O(d)$ equally spaced disjoint sets on a single line (e.g. all the stabilizers on a single line has error syndrome), like below
//!
//! <div style="width: 100%; display: flex; justify-content: center;"><svg id="distributed_uf_decoder_connections_worst_case" style="width: 400px;" viewBox="0 0 400 110"></svg></div>
//! <script>function draw_distributed_uf_decoder_connections_worst_case(){let t=document.getElementById("distributed_uf_decoder_connections_worst_case");if(!t)return;parseInt(10.5);const e="http://www.w3.org/2000/svg",r=["rgb(255, 0, 0)","rgb(0, 0, 255)","rgb(0, 255, 0)","rgb(255, 0, 255)","rgb(0, 255, 255)","rgb(255, 255, 0)"];for(let i=0;i<5;++i)for(let s=0;s<20;++s){const n=(s+1.5)*(400/22),d=(i+1)*(400/22);let o=document.createElementNS(e,"circle");o.setAttribute("cx",n),o.setAttribute("cy",d),o.setAttribute("r",400/22*.3),o.setAttribute("fill","rgb(0, 0, 0)"),2==i&&0!=s&&19!=s?o.setAttribute("fill",r[parseInt((s-1)/3)]):1!=i&&3!=i||0==s||19==s||(s+1)%3!=0||o.setAttribute("fill",r[parseInt((s-1)/3)]),t.appendChild(o)}for(let r=0;r<5;++r){let i=document.createElementNS(e,"rect");i.setAttribute("width",400/22*2),i.setAttribute("height",400/22),i.setAttribute("style","fill:none; stroke:black; stroke-width:3;"),i.setAttribute("x",400/22*(3*r+4)),i.setAttribute("y",400/22*2.5),t.appendChild(i)}}document.addEventListener("DOMContentLoaded",draw_distributed_uf_decoder_connections_worst_case);</script>
//!
//! The root (red block) will have to spread linearly from the left side to the right side.
//! The $O(\log{d})$ direct connections doesn't work in this case because the old root is not the same between disjoint sets (in different colors).
//! After spreading for $O(d)$ time, the system will be below
//!
//! <div style="width: 100%; display: flex; justify-content: center;"><svg id="distributed_uf_decoder_connections_worst_case_after" style="width: 400px;" viewBox="0 0 400 110"></svg></div>
//! <script>function draw_distributed_uf_decoder_connections_worst_case_after(){let t=document.getElementById("distributed_uf_decoder_connections_worst_case_after");if(!t)return;parseInt(10.5);const e="rgb(255, 0, 0)";for(let r=0;r<5;++r)for(let n=0;n<20;++n){const d=(n+1.5)*(400/22),c=(r+1)*(400/22);let i=document.createElementNS("http://www.w3.org/2000/svg","circle");i.setAttribute("cx",d),i.setAttribute("cy",c),i.setAttribute("r",400/22*.3),i.setAttribute("fill","rgb(0, 0, 0)"),2==r&&0!=n&&19!=n?i.setAttribute("fill",e):1!=r&&3!=r||0==n||19==n||(n+1)%3!=0||i.setAttribute("fill",e),t.appendChild(i)}}document.addEventListener("DOMContentLoaded",draw_distributed_uf_decoder_connections_worst_case_after);</script>
//!
//! Thus, the optimized design has only a smaller average time complexity of $O(\log{d})$ per round, but the worst case is still $O(d)$ per round.
//! If the algorithm does not finish in $O(\log{d})$ time bound, we cannot just stop it at the middle because that will cause unrecoverable inconsistent state.
//!
//! The Union-Find operation is solved, but there is still a hard problem, that how to compute the cardinality of a cluster with low complexity?
//! In the sequential union find algorithm, this is done by simply store the state in the root and update on every union operation.
//! Since the union operation happens locally and distributively in our algorithm, it's not easy to decide how to update the cardinality.
//! We solve this problem by maintaining the old cardinality in the root node, and also stores a counter that counts the increment of cardinality in another register.
//! When several disjoint sets merge into a new one, the merged root node will send a direct message to the new root node telling him the old cardinality.
//! The new root will add this cardinality into the counter without changing its old cardinality register.
//! This procedure takes a strict timing bound of $O(\log{d})$ because of the fast channel architecture.
//! In this way, the ultimate root node will receive the old cardinality message from all root nodes that are merged into him for exactly once, which is expected.
//! Note that those intermediate root will also have non-zero counter value, but this doesn't matter, since the old cardinality register keeps constant during iteration.
//!
//! ## Implementation
//!
//! There is a nice attribute of the fast channels in distributed UF decoder, that messages from all the channels can be processed simultaneously.
//! This happens because among those message there is only one that "outstands".
//! The messages can be first filtered simultaneously by the old root field (ignore those whose old root doesn't match to the local register).
//! Then, those that are valid can be compared in a two-two manner, to elect a best one, which takes $O(\log{m})$ circuit level latency in a tree-structured compare.
//! Since the amount of channels for each node is $m = O(\log{d})$, the circuit level latency is just $O(\log{\log{d}})$ which is super scalable.
//! The outstanding message will finally reach the core logic and been executed.
//! Since all the messages are guaranteed to be handled in a single clock cycle, the channel could simply be a FIFO with length 1.
//! This saves a lot of hardware resources.
//!
//! For the cardinality updating message, though, it must be processed sequentially.
//! Thus, it will use another set of fast channels, different from the Union messages discussed above.
//! It could happen that one node receives multiple messages (brokering them to different roots) at the same time, but could not handle them.
//! The channel will be a special one that has a feedback bit, representing whether the last message is already handled.
//! If the sender wants to send a message but the receiver is still busy at handling other messages, it will just delay the message and waiting for next clock cycle.
//! The cardinality update doesn't affect the correctness of union operations, so it has least priority.
//! The iteration stops once there are no messages pending, and till then the cardinality at the new root will increase by the increment counter.
//! This ensures a consistent state at the end of the iteration.
//!
//! ## Interface
//!
//! - Initialization
//!   - nodes: Vec\<InputNode\>, each node has the following field
//!     - user_data: \<U\>, could be anything for user-defined functions to be used (dynamic update prohibited)
//!     - is_error_syndrome: bool, if the stabilizer detects an error, set this to `true` (dynamic update prohibited)
//!     - boundary_cost: Option<usize>, if connected to boundary, then `Some(cost)` (dynamic update allowed)
//!   - neighbors: Vec\<InputNeighbor\>, connection of direct neighbors (order doesn't matter)
//!   - fast_channels: Vec\<InputFastChannel\>, fast channels to reduce the average time complexity, see "fast channel architecture" in Design section
//!   - distance: fn(a: \<U\>, b: \<U\>) -> usize, the Manhattan distance between two nodes, which is used to send direct messages in a fastest path
//!
//! After initialization, the algorithm will instantiate multiple processing unit (PU), each corresponds to a node.
//!

use std::collections::{HashMap, VecDeque};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use super::serde::{Serialize, Deserialize};
use super::derive_more::{Constructor};

#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct InputNode<U: std::fmt::Debug> {
    /// user defined data corresponds to each node
    pub user_data: U,
    /// whether this stabilizer has detected a error
    pub is_error_syndrome: bool,
    /// if this node has a direct path to boundary, then set to `Some(cost)` given the integer cost of matching to boundary, otherwise `None`.
    /// This attribute can be modified later, as what's happening in a continuously running quantum computer
    pub boundary_cost: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct InputNeighbor {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
    /// current increased value, should be 0 in general, and can be `length` if there exists an erasure error (Pauli error with known position)
    pub increased: usize,
    /// the total length of this edge
    pub length: usize,
    /// latency of the message passing, note that this latency also affect reading the `increased` value
    /// , and thus will increase the clock cycle in spreading stage
    pub latency: usize,
}

#[derive(Debug, Serialize, Deserialize, Constructor)]
pub struct InputFastChannel {
    /// address of node `a`
    pub a: usize,
    /// address of node `b`
    pub b: usize,
    /// latency of the message passing
    pub latency: usize,
}

#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct DistributedUnionFind<U: std::fmt::Debug> {
    /// the immutable input nodes, which maps to processing units in `processing_units`
    pub nodes: Vec<InputNode<U>>,
    /// processing units, each one corresponding to a node of the input graph
    pub processing_units: Vec<ProcessingUnit>,
    #[derivative(Debug="ignore")]
    #[serde(skip_serializing)]
    /// distance function given two nodes' user data
    pub distance: Box<dyn Fn(&U, &U) -> usize>,
}

#[derive(Debug, Serialize)]
pub struct ProcessingUnit {
    /// directly connected neighbors, (address, neighbor_link)
    #[serde(skip_serializing)]
    pub neighbors: Vec<(usize, Rc<RefCell<NeighborLink>>)>,
    /// union message channels, including both neighbor channels and fast channels, where each neighbor channel has the same indices as in `neighbors`
    #[serde(skip_serializing)]
    pub union_out_channels: Vec<(usize, Rc<RefCell<Channel<UnionMessage>>>)>,
    #[serde(skip_serializing)]
    pub union_in_channels: Vec<(usize, Rc<RefCell<Channel<UnionMessage>>>)>,
    /// direct message channels, including both neighbor channels and fast channels
    #[serde(skip_serializing)]
    pub direct_out_channels: Vec<(usize, Rc<RefCell<Channel<DirectMessage>>>)>,
    #[serde(skip_serializing)]
    pub direct_in_channels: Vec<(usize, Rc<RefCell<Channel<DirectMessage>>>)>,
    /// increased value towards boundary, only valid when `node.boundary_cost` is `Some(_)`
    pub boundary_increased: usize,
    /// old root register
    pub old_root: usize,
    /// updated root register
    pub updated_root: usize,
    /// is odd cluster
    pub is_odd_cluster: bool,
    /// is touching boundary
    pub is_touching_boundary: bool,
    /// is odd cardinality, counts the number of error syndromes in a region
    pub is_odd_cardinality: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NeighborLink {
    /// current increased value, should be 0 in general, and can be `length` if there exists an erasure error (Pauli error with known position)
    pub increased: usize,
    /// the total length of this edge
    pub length: usize,
    /// latency of the `increased` value to be updated at the peer
    pub latency: usize,
}

#[derive(Derivative, Serialize)]
#[derivative(Debug)]
pub struct Channel<Message> {
    /// latency of the message passing
    pub latency: usize,
    /// a ring buffer having exactly `latency` objects in stable state
    pub deque: VecDeque<Option<Message>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnionMessage {
    /// the old root of the sender
    pub old_root: usize,
    /// the updated root of the sender
    pub updated_root: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DirectMessage {
    /// the receiver address
    pub receiver: usize,
    /// if this is the root of odd cardinality that will be merged into `receiver`'s cluster
    pub is_odd_cardinality_root: bool,
    /// if this is a boundary node, so that the receiver as it's new root will know it is odd cluster
    pub is_touching_boundary: bool,
}

pub fn unordered_edge_compare(a1: usize, b1: usize, a2: usize, b2: usize) -> Ordering {
    let x_s = std::cmp::min(a1, b1);
    let x_l = std::cmp::max(a1, b1);
    let y_s = std::cmp::min(a2, b2);
    let y_l = std::cmp::max(a2, b2);
    match x_s.cmp(&y_s) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => x_l.cmp(&y_l),
    }
}

impl Ord for InputNeighbor {
    fn cmp(&self, other: &Self) -> Ordering {
        unordered_edge_compare(self.a, self.b, other.a, other.b)
    }
}

impl PartialOrd for InputNeighbor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InputNeighbor {
    fn eq(&self, other: &Self) -> bool {
        unordered_edge_compare(self.a, self.b, other.a, other.b) == Ordering::Equal
    }
}

impl Eq for InputNeighbor {}

impl Ord for InputFastChannel {
    fn cmp(&self, other: &Self) -> Ordering {
        unordered_edge_compare(self.a, self.b, other.a, other.b)
    }
}

impl PartialOrd for InputFastChannel {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for InputFastChannel {
    fn eq(&self, other: &Self) -> bool {
        unordered_edge_compare(self.a, self.b, other.a, other.b) == Ordering::Equal
    }
}

impl Eq for InputFastChannel {}

impl<Message> Channel<Message> {
    pub fn has_message_flying(&self) -> bool {
        let mut found = false;
        for message in self.deque.iter() {
            if message.is_some() {
                found = true;
                break
            }
        }
        found
    }
} 

impl<U: std::fmt::Debug> DistributedUnionFind<U> {
    pub fn new(nodes: Vec<InputNode<U>>, mut neighbors: Vec<InputNeighbor>, mut fast_channels: Vec<InputFastChannel>, 
            distance: impl Fn(&U, &U) -> usize + 'static) -> Self {
        // filter invalid and duplicated neighbor edges
        let nodes_len = nodes.len();
        let neighbors_length = neighbors.len();
        neighbors.retain(|edge| {
            edge.a != edge.b && edge.a < nodes_len && edge.b < nodes_len
        });  // remove invalid neighbor edges
        assert_eq!(neighbors_length, neighbors.len(), "`neighbors` contains invalid edges (either invalid address or edge connecting the same node)");
        neighbors.sort_unstable();
        neighbors.dedup();
        assert_eq!(neighbors_length, neighbors.len(), "`neighbors` contains duplicate elements (including the same ends)");
        // filter invalid and duplicated fast_channels edges
        let fast_channels_length = fast_channels.len();
        fast_channels.retain(|edge| {
            edge.a != edge.b && edge.a < nodes_len && edge.b < nodes_len
        });  // remove invalid neighbor edges
        assert_eq!(fast_channels_length, fast_channels.len(), "`fast_channels` contains invalid edges (either invalid address or edge connecting the same node)");
        fast_channels.sort_unstable();
        fast_channels.dedup();
        assert_eq!(fast_channels_length, fast_channels.len(), "`fast_channels` contains duplicate elements (including the same ends)");
        // build processing units
        let mut processing_units: Vec<_> = nodes.iter().enumerate().map(|(i, node)| {
            ProcessingUnit {
                neighbors: Vec::new(),
                union_out_channels: Vec::new(),
                union_in_channels: Vec::new(),
                direct_out_channels: Vec::new(),
                direct_in_channels: Vec::new(),
                boundary_increased: 0,
                old_root: i,
                updated_root: i,
                is_odd_cluster: node.is_error_syndrome,
                is_touching_boundary: false,
                is_odd_cardinality: node.is_error_syndrome,
            }
        }).collect();
        // build neighbors and their channels
        for InputNeighbor { a, b, increased, length, latency } in neighbors.iter() {
            let neighbor_link = Rc::new(RefCell::new(NeighborLink {
                increased: *increased,
                length: *length,
                latency: *latency,
            }));
            processing_units[*a].neighbors.push((*b, neighbor_link.clone()));
            processing_units[*b].neighbors.push((*a, neighbor_link.clone()));
            // build channels
            for (x, y) in [(*a, *b), (*b, *a)].iter() {
                // union channels
                let channel = Rc::new(RefCell::new(Channel {
                    latency: *latency,
                    deque: (0..*latency).map(|_| None).collect(),
                }));
                processing_units[*x].union_out_channels.push((*y, channel.clone()));
                processing_units[*y].union_in_channels.push((*x, channel.clone()));
                // direct message channels
                let channel = Rc::new(RefCell::new(Channel {
                    latency: *latency,
                    deque: (0..*latency).map(|_| None).collect(),
                }));
                processing_units[*x].direct_out_channels.push((*y, channel.clone()));
                processing_units[*y].direct_in_channels.push((*x, channel.clone()));
            }
        }
        for pu in processing_units.iter() {
            assert_eq!(pu.neighbors.len(), pu.union_out_channels.len(), "each neighbor should have exactly one channel");
            assert_eq!(pu.neighbors.len(), pu.union_in_channels.len(), "each neighbor should have exactly one channel");
            assert_eq!(pu.neighbors.len(), pu.direct_out_channels.len(), "each neighbor should have exactly one channel");
            assert_eq!(pu.neighbors.len(), pu.direct_in_channels.len(), "each neighbor should have exactly one channel");
        }
        // build fast channels
        for InputFastChannel { a, b, latency } in fast_channels.iter() {
            // build channels
            for (x, y) in [(*a, *b), (*b, *a)].iter() {
                // union channels
                let channel = Rc::new(RefCell::new(Channel {
                    latency: *latency,
                    deque: (0..*latency).map(|_| None).collect(),
                }));
                processing_units[*x].union_out_channels.push((*y, channel.clone()));
                processing_units[*y].union_in_channels.push((*x, channel.clone()));
                // direct message channels
                let channel = Rc::new(RefCell::new(Channel {
                    latency: *latency,
                    deque: (0..*latency).map(|_| None).collect(),
                }));
                processing_units[*x].direct_out_channels.push((*y, channel.clone()));
                processing_units[*y].direct_in_channels.push((*x, channel.clone()));
            }
        }
        for pu in processing_units.iter() {
            assert_eq!(pu.union_out_channels.len(), pu.union_in_channels.len(), "amount of channels should be the same");
            assert_eq!(pu.union_out_channels.len(), pu.direct_out_channels.len(), "amount of channels should be the same");
            assert_eq!(pu.union_out_channels.len(), pu.direct_in_channels.len(), "amount of channels should be the same");
        }
        Self {
            nodes: nodes,
            processing_units: processing_units,
            distance: Box::new(distance),
        }
    }

    /// sanity check only for simulation, to check that the latency simulation is actually working
    pub fn channels_sanity_check(&self) {
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let pu = &self.processing_units[i];
            for (_peer, out_channel) in pu.union_out_channels.iter() {
                let out_channel = out_channel.borrow();
                assert_eq!(out_channel.deque.len(), out_channel.latency, "there should be `latency` elements in deque in stable state");
            }
            for (_peer, out_channel) in pu.direct_out_channels.iter() {
                let out_channel = out_channel.borrow();
                assert_eq!(out_channel.deque.len(), out_channel.latency, "there should be `latency` elements in deque in stable state");
            }
        }
    }

    /// test if there is still message spreading, this can be done in O(1) on FPGA, however this is O(n) in CPU, very expensive
    pub fn has_message_flying(&self) -> bool {
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let pu = &self.processing_units[i];
            for (_peer, out_channel) in pu.union_out_channels.iter() {
                if out_channel.borrow().has_message_flying() {
                    return true
                }
            }
            for (_peer, out_channel) in pu.direct_out_channels.iter() {
                if out_channel.borrow().has_message_flying() {
                    return true
                }
            }
        }
        false
    }

    /// suppose the root node has the correct cardinality, it should spread to all of his nodes
    pub fn spread_is_odd_cluster(&mut self) -> usize {
        let mut clock_cycles = 0;
        let nodes_len = self.nodes.len();
        // in FPGA, this is done by giving a trigger signal to all PUs, in O(1) time
        for i in 0..nodes_len {
            let pu = &mut self.processing_units[i];
            assert_eq!(pu.updated_root, pu.old_root, "when spreading cardinality, old root must already been updated, otherwise it's inconsistent state");
            pu.is_odd_cluster = false;  // first set them all to even cluster
        }
        // in FPGA, with fast channel architecture, this has worst case bound of O(log(d)) time
        let mut spreading = true;
        while spreading {
            clock_cycles += 1;  // each clock cycle can process one message from every in channels and then push one message to every out channels
            for i in 0..nodes_len {
                let pu = &mut self.processing_units[i];
                // first retrieve messages from all union in channel
                let old_is_odd_cluster = pu.is_odd_cluster;
                let in_messages: Vec<Option<UnionMessage>> = pu.union_in_channels.iter().map(|(_peer, in_channel)| {
                    let mut in_channel = in_channel.borrow_mut();
                    in_channel.deque.pop_front().unwrap()
                }).collect();
                // handle those messages to compute pu.is_odd_cluster, this can be done in O(1) on FPGA, 
                //    with O(log(log(d))) higher gate level latency, which may reduce the clock cycle a little bit, but still pretty scalable
                for message in in_messages.iter() {
                    match message {
                        Some(UnionMessage{ old_root, updated_root: _ }) => {
                            if *old_root == pu.old_root {
                                pu.is_odd_cluster = true;
                            }
                        },
                        None => { },
                    }
                }
                if i == pu.updated_root {
                    // if touching boundary, then it's even cluster
                    // if even cardinality, then it's even cluster
                    pu.is_odd_cluster = !pu.is_touching_boundary && pu.is_odd_cardinality;
                }
                // then broadcast messages
                let should_broadcast = pu.is_odd_cluster != old_is_odd_cluster;
                if should_broadcast {
                    assert_eq!(old_is_odd_cluster, false, "pu.is_odd_cluster never changes from `true` to `false` in this stage");
                    assert_eq!(pu.is_odd_cluster, true, "pu.is_odd_cluster never changes from `true` to `false` in this stage");
                }
                for (_peer, out_channel) in pu.union_out_channels.iter() {
                    let mut out_channel = out_channel.borrow_mut();
                    out_channel.deque.push_back(if should_broadcast { 
                        Some(UnionMessage {
                            old_root: pu.old_root,
                            updated_root: pu.updated_root,
                        })
                    } else {
                        None
                    });
                }
            }
            spreading = self.has_message_flying();
        }
        clock_cycles
    }

    /// return (clock cycle used in this iteration, need to run another iteration)
    pub fn run_single_iteration(&mut self) -> (usize, bool) {
        let mut clock_cycles = 0;
        clock_cycles += self.spread_is_odd_cluster();
        (clock_cycles, false)
    }
}

/// create nodes for standard planar code (2d, perfect measurement condition). return only X stabilizers or only Z stabilizers.
/// return (nodes, position_to_index, neighbors), the fast channel should be empty, which is Vec::new()
pub fn make_standard_planar_code_2d_nodes_no_fast_channel(d: usize, is_x_stabilizers: bool) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>,
        Vec<InputNeighbor>) {
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            position_to_index.insert((i, j), nodes.len());
            let is_boundary = if is_x_stabilizers { j == 1 || j == 2*d-3 } else { i == 1 || i == 2*d-3 };
            nodes.push(InputNode {
                user_data: (i, j),
                is_error_syndrome: false,
                boundary_cost: if is_boundary { Some(2) } else { None },
            });
        }
    }
    let mut neighbors = Vec::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            for (di, dj) in [(2, 0), (0, 2)].iter() {
                let ni = i + di;
                let nj = j + dj;
                if ni <= 2*d-2 && nj <= 2*d-2 {
                    neighbors.push(InputNeighbor{
                        a: position_to_index[&(i, j)],
                        b: position_to_index[&(ni, nj)],
                        increased: 0,
                        length: 2,
                        latency: 1,
                    });
                }
            }
        }
    }
    (nodes, position_to_index, neighbors)
}

pub fn manhattan_distance_standard_planar_code_2d_nodes(a: &(usize, usize), b: &(usize, usize)) -> usize {
    let (i1, j1) = *a;
    let (i2, j2) = *b;
    let di = (i1 as isize - i2 as isize).abs() as usize;
    let dj = (j1 as isize - j2 as isize).abs() as usize;
    assert!(di % 2 == 0 && dj % 2 == 0, "cannot compute cost between different types of stabilizers");
    (di + dj) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test distributed_union_find_decoder_test_case_1 -- --nocapture` to run specific test
    
    fn make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d: usize) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<InputNeighbor>) {
        make_standard_planar_code_2d_nodes_no_fast_channel(d, true)
    }

    fn pretty_print_standard_planar_code(decoder: &DistributedUnionFind<(usize, usize)>) {
        let nodes_len = decoder.nodes.len();
        for i in 0..nodes_len {
            let node = &decoder.nodes[i];
            let pu = &decoder.processing_units[i];
            let updated_root_user_data = &decoder.nodes[pu.updated_root].user_data;
            let old_root_user_data = &decoder.nodes[pu.old_root].user_data;
            let error_symbol = if node.is_error_syndrome { "x" } else { " " };
            let odd_cluster_symbol = if pu.is_odd_cluster { "o" } else { " " };
            let touching_boundary_symbol = if pu.updated_root == i && pu.is_touching_boundary { "t" } else { " " };
            let odd_cardinality_symbol = if pu.updated_root == i && pu.is_odd_cardinality { "c" } else { " " };
            let boundary_string = match node.boundary_cost {
                Some(boundary_cost) => {
                    format!("b({}/{})", pu.boundary_increased, boundary_cost)
                },
                None => format!("      "),
            };
            let neighbors_len = pu.neighbors.len();
            let mut neighbor_string = String::new();
            for j in 0..neighbors_len {
                let (neighbor_addr, edge) = &pu.neighbors[j];
                let edge = edge.borrow();
                let neighbor_user_data = &decoder.nodes[*neighbor_addr].user_data;
                let string = format!("{:?}[{}/{}] ", neighbor_user_data, edge.increased, edge.length);
                neighbor_string.push_str(string.as_str());
            }
            println!("{:?} ∈ updated {:?} old {:?} {} {} {} {} {} n: {}", node.user_data, updated_root_user_data, old_root_user_data, error_symbol, odd_cluster_symbol,
                touching_boundary_symbol, odd_cardinality_symbol, boundary_string, neighbor_string);
        }
    }

    #[test]
    #[should_panic]
    fn distributed_union_find_decoder_sanity_check_1() {
        let (nodes, position_to_index, mut neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        println!("nodes: {:#?}", nodes);
        println!("position_to_index: {:?}", position_to_index);
        println!("neighbors: {:?}", neighbors);
        // add duplicate neighbor edge
        neighbors.push(InputNeighbor::new(position_to_index[&(0, 3)], position_to_index[&(0, 1)], 1000, 1000, 1));
        // should then panic
        DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes);
    }

    #[test]
    fn distributed_union_find_decoder_sanity_check_2() {
        let (nodes, _position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        let distributed_union_find = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes);
        pretty_print_standard_planar_code(&distributed_union_find);
    }

    #[test]
    fn distributed_union_find_decoder_sanity_check_3() {
        // test `spread_is_odd_cluster` function
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = true;  // test touching boundary
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;  // test single error syndrome
        nodes[position_to_index[&(4, 1)]].is_error_syndrome = true;  // test 2 matching together
        nodes[position_to_index[&(4, 3)]].is_error_syndrome = true;
        let mut distributed_union_find = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes);
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(4, 1)]].is_odd_cluster, true);
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(4, 3)]].is_odd_cluster, true);
        distributed_union_find.processing_units[position_to_index[&(0, 1)]].is_touching_boundary = true;
        distributed_union_find.processing_units[position_to_index[&(4, 3)]].old_root = position_to_index[&(4, 1)];
        distributed_union_find.processing_units[position_to_index[&(4, 3)]].updated_root = position_to_index[&(4, 1)];
        distributed_union_find.processing_units[position_to_index[&(4, 1)]].is_odd_cardinality = false;  // because the set has (4, 1) and (4, 3)
        distributed_union_find.channels_sanity_check();
        distributed_union_find.spread_is_odd_cluster();
        distributed_union_find.channels_sanity_check();
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(4, 1)]].is_odd_cluster, false);
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(4, 3)]].is_odd_cluster, false);
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(0, 1)]].is_odd_cluster, false);
        assert_eq!(distributed_union_find.processing_units[position_to_index[&(2, 3)]].is_odd_cluster, true);
        pretty_print_standard_planar_code(&distributed_union_find);
    }

}
