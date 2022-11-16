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

use std::collections::{HashMap, VecDeque, HashSet};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::rc::Rc;
use std::fs::OpenOptions;
use std::io::prelude::*;
use super::serde::{Serialize, Deserialize};
use super::derive_more::{Constructor};
use super::offer_decoder;
use super::ftqec;
use super::types::QubitType;
use super::union_find_decoder;

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
    #[derivative(Debug="ignore")]
    #[serde(skip_serializing)]
    /// compare function given two nodes' user data
    pub compare: Box<dyn Fn(&U, &U) -> Ordering>,
    /// original inputs
    pub input_neighbors: Vec<InputNeighbor>,
    pub input_fast_channels: Vec<InputFastChannel>,
}

#[derive(Debug, Serialize)]
pub struct ProcessingUnit {
    /// directly connected neighbors, (address, is_old_root_different, neighbor_link)
    pub neighbors: Vec<Neighbor>,
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
    /// is touching boundary, both used in root and the node that is really touching the boundary
    pub is_touching_boundary: bool,
    /// is odd cardinality, counts the number of error syndromes in a region
    pub is_odd_cardinality: bool,
    /// this is only used for debugging, to count the real cardinality of cluster. `is_odd_cardinality` should be enough for implementing on FPGA
    pub debug_cardinality: usize,
    /// whether need to tell the new root about the cardinality and the boundary
    pub pending_tell_new_root_cardinality: bool,
    pub pending_tell_new_root_touching_boundary: bool,
}

#[derive(Debug, Serialize)]
pub struct Neighbor {
    /// the index of this neighbor
    pub address: usize,
    /// the supposed updated root of the neighbor, which may not be the real updated_root. only used to stop broadcast when unnecessary
    pub supposed_updated_root: usize,
    /// this will sync with the peer at the start of the iteration, and keeps constant within the iteration
    pub old_root: usize,
    /// this will need `latency` time to sync with peer
    pub is_fully_grown: bool,
    /// the shared link, which is used to pass information
    #[serde(skip_serializing)]
    pub link: Rc<RefCell<NeighborLink>>,
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
pub struct Channel<Message: std::fmt::Debug> {
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

impl<Message: std::fmt::Debug> Channel<Message> {
    pub fn has_message_flying(&self) -> bool {
        let mut found = false;
        for message in self.deque.iter() {
            if message.is_some() {
                found = true;
                // println!("[flying] {:?}", message);
                break
            }
        }
        found
    }
}

impl<U: std::fmt::Debug> DistributedUnionFind<U> {
    pub fn new(nodes: Vec<InputNode<U>>, mut neighbors: Vec<InputNeighbor>, mut fast_channels: Vec<InputFastChannel>, 
            distance: impl Fn(&U, &U) -> usize + 'static, compare: impl Fn(&U, &U) -> Ordering + 'static) -> Self {
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
                debug_cardinality: if node.is_error_syndrome { 1 } else { 0 },
                pending_tell_new_root_cardinality: false,
                pending_tell_new_root_touching_boundary: false,
            }
        }).collect();
        // build neighbors and their channels
        for InputNeighbor { a, b, increased, length, latency } in neighbors.iter() {
            assert!(*latency >= 1, "latency must be at least 1, with 1 meaning the peer receives the information in the next clock cycle");
            let neighbor_link = Rc::new(RefCell::new(NeighborLink {
                increased: *increased,
                length: *length,
                latency: *latency,
            }));
            processing_units[*a].neighbors.push(Neighbor {
                address: *b,
                supposed_updated_root: *b,
                old_root: *b,
                is_fully_grown: false,  // will update in `spread_cluster`
                link: neighbor_link.clone(),
            });
            processing_units[*b].neighbors.push(Neighbor {
                address: *a,
                supposed_updated_root: *a,
                old_root: *a,
                is_fully_grown: false,  // will update in `spread_cluster`
                link: neighbor_link.clone(),
            });
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
            assert!(*latency >= 1, "latency must be at least 1, with 1 meaning the peer receives the information in the next clock cycle");
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
            compare: Box::new(compare),
            input_neighbors: neighbors,
            input_fast_channels: fast_channels,
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
                    // println!("[flying] {:?} -> {:?}", self.nodes[i].user_data, self.nodes[*_peer].user_data);
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

    pub fn grow_boundary(&mut self) -> usize {
        let nodes_len = self.nodes.len();
        // in FPGA, this is done by giving a trigger signal to all PUs, in O(1) time
        for i in 0..nodes_len {
            let pu = &self.processing_units[i];
            assert_eq!(pu.updated_root, pu.old_root, "when growing boundary, old root must already been updated, otherwise it's inconsistent state");
            if pu.is_odd_cluster {
                let neighbors_len = pu.neighbors.len();
                for j in 0..neighbors_len {
                    let pu = &self.processing_units[i];
                    let neighbor = &pu.neighbors[j];
                    let mut neighbor_link = neighbor.link.borrow_mut();
                    if neighbor_link.increased < neighbor_link.length {
                        neighbor_link.increased += 1;  // grow the edge if it's not fully grown
                    }
                }
                match self.nodes[i].boundary_cost {
                    Some(boundary_cost) => {
                        let pu = &mut self.processing_units[i];
                        if pu.boundary_increased < boundary_cost {
                            pu.boundary_increased += 1;
                        }
                    },
                    None => { },
                }
            }
        }
        1  // always done in 1 clock cycle, this doesn't need to be synchronized
    }

    /// compare nodes given their addresses
    pub fn get_node_smaller(&self, a: usize, b: usize) -> usize {
        if (self.compare)(&self.nodes[a].user_data, &self.nodes[b].user_data) == Ordering::Less { a } else { b }
    }

    /// compute distance given address
    pub fn get_node_distance(&self, a: usize, b: usize) -> usize {
        (self.distance)(&self.nodes[a].user_data, &self.nodes[b].user_data)
    }

    pub fn spread_clusters(&mut self) -> usize {
        let mut clock_cycles = 0;
        let nodes_len = self.nodes.len();
        // compute `is_old_root_different` for each neighbor, which is finished in maximum latency between all neighbors
        let mut maximum_latency = 1;
        for i in 0..nodes_len {
            let pu = &self.processing_units[i];
            assert_eq!(pu.updated_root, pu.old_root, "when growing boundary, old root must already been updated, otherwise it's inconsistent state");
            let neighbors_len = pu.neighbors.len();
            for j in 0..neighbors_len {
                let pu = &mut self.processing_units[i];
                let neighbor = &mut pu.neighbors[j];
                let neighbor_link = neighbor.link.borrow();
                if neighbor_link.latency > maximum_latency {
                    maximum_latency = neighbor_link.latency;
                }
                // then sync the information
                let neighbor_addr = neighbor.address;
                neighbor.is_fully_grown = neighbor_link.increased >= neighbor_link.length;
                drop(neighbor_link);
                let neighbor_root = self.processing_units[neighbor_addr].updated_root;
                let neighbor = &mut self.processing_units[i].neighbors[j];  // re-borrow as mutable
                neighbor.supposed_updated_root = neighbor_root;
                neighbor.old_root = neighbor_root;
            }
        }
        clock_cycles += maximum_latency;
        // then broadcast messages if need to, both to union channel and direct channel
        let mut spreading = true;
        // run until there is no messages flying
        while spreading {
            clock_cycles += 1;
            let mut intermediate_states = Vec::with_capacity(nodes_len);
            for i in 0..nodes_len {
                let pu = &mut self.processing_units[i];
                // check if this is the first time to touch the boundary, finished in O(1) time on FPGA
                let old_is_touching_boundary = pu.is_touching_boundary;
                match self.nodes[i].boundary_cost {
                    Some(boundary_cost) => {
                        if pu.boundary_increased >= boundary_cost {
                            pu.is_touching_boundary = true;
                        }
                    },
                    None => { },
                }
                // check if there is any neighbor with fully grown edge, this can be done in O(1) time on FPGA
                // if old_updated_root != pu.updated_root && is_error_syndrome, then should send direct message to new root to add cardinality
                let old_updated_root = pu.updated_root;
                let pu_old_root = pu.old_root;
                // `new_updated_root` is set to the minimum one among all messages as well as all neighbor old root's if that's fully grown
                //     which is done in O(log(log(d))) gate level latency on FPGA, so it's still 1 clock cycle with slightly lower clock rate in large code distances
                let mut new_updated_root = pu.updated_root;
                let neighbors_len = pu.neighbors.len();
                for j in 0..neighbors_len {
                    let pu = &self.processing_units[i];
                    let neighbor = &pu.neighbors[j];
                    if neighbor.is_fully_grown {
                        new_updated_root = self.get_node_smaller(new_updated_root, neighbor.address);
                    }
                }
                // processing one message from all union channels
                let pu = &mut self.processing_units[i];
                let in_messages: Vec<Option<UnionMessage>> = pu.union_in_channels.iter().map(|(_peer, in_channel)| {
                    let mut in_channel = in_channel.borrow_mut();
                    in_channel.deque.pop_front().unwrap()
                }).collect();
                // handle those messages to compute `new_updated_root`, this can be done in O(1) on FPGA, 
                //    with O(log(log(d))) higher gate level latency, which may reduce the clock cycle a little bit, but still pretty scalable
                for message in in_messages.iter() {
                    match message {
                        Some(UnionMessage{ old_root, updated_root }) => {
                            if *old_root == pu_old_root {  // otherwise don't consider it at all!
                                new_updated_root = self.get_node_smaller(new_updated_root, *updated_root);
                            }
                        },
                        None => { },
                    }
                }
                // send messages to all union channels if the updated root changes in this cycle, this can be done in O(1) on FPGA
                let pu = &mut self.processing_units[i];
                for (j, (_peer, out_channel)) in pu.union_out_channels.iter().enumerate() {
                    let mut out_channel = out_channel.borrow_mut();
                    let mut old_root = pu_old_root;
                    if j < pu.neighbors.len() {
                        let neighbor = &pu.neighbors[j];
                        if neighbor.is_fully_grown {
                            old_root = neighbor.old_root;  // must fit into the peer's old_root, otherwise he won't take it!
                        }
                    }
                    out_channel.deque.push_back(if new_updated_root != old_updated_root { 
                        Some(UnionMessage {
                            old_root: old_root,
                            updated_root: new_updated_root,
                        })
                    } else {
                        None
                    });
                }
                // finally update the state
                pu.updated_root = new_updated_root;
                // at the same time, try to find a direct message to route
                // since the direct message should be very rare in the system, just a simple logic would suffice
                if new_updated_root != old_updated_root {
                    if self.nodes[i].is_error_syndrome {  // only nodes with error syndrome should tell the root about updated cardinality
                        pu.pending_tell_new_root_cardinality = true;
                    }
                    if pu.is_touching_boundary {
                        pu.pending_tell_new_root_touching_boundary = true;
                    }
                }
                if pu.is_touching_boundary != old_is_touching_boundary {
                    pu.pending_tell_new_root_touching_boundary = true;
                }
                if new_updated_root == i {  // don't need to send message to myself
                    pu.pending_tell_new_root_cardinality = false;
                    pu.pending_tell_new_root_touching_boundary = false;
                }
                let mut pending_direct_message = None;
                if pu.pending_tell_new_root_cardinality || pu.pending_tell_new_root_touching_boundary {
                    pending_direct_message = Some(DirectMessage {
                        receiver: new_updated_root,
                        is_odd_cardinality_root: pu.pending_tell_new_root_cardinality,
                        is_touching_boundary: pu.pending_tell_new_root_touching_boundary,
                    });
                }
                // read from all direct channels, finish in O(1) on FPGA
                let mut need_to_pop_direct_in_channel_from_idx = None;
                for (j, (_peer, in_channel)) in pu.direct_in_channels.iter().enumerate() {
                    let mut in_channel = in_channel.borrow_mut();
                    let in_message = in_channel.deque.get(0).unwrap();
                    if in_message.is_none() {
                        in_channel.deque.pop_front().unwrap();  // always get None message from the queue
                    } else {
                        let in_message = in_message.as_ref().unwrap();
                        if in_message.receiver == i {  // I'm the receiver, so I'll process this information
                            // if children has error syndrome and is its first time to join this cluster, then the cluster's cardinality +1
                            pu.debug_cardinality += if in_message.is_odd_cardinality_root { 1 } else { 0 };
                            pu.is_odd_cardinality ^= in_message.is_odd_cardinality_root;
                            // if some of its children touch the boundary, then the cluster touches the boundary
                            pu.is_touching_boundary |= in_message.is_touching_boundary;
                            // always pop it because it's already been handled
                            in_channel.deque.pop_front().unwrap();
                        } else {
                            // never pop valid in_message here, do it after making sure that the message can be handled or brokered
                            if pending_direct_message.is_none() {  // retrieve Some message only if pending_direct_message is none
                                // do not take it from deque, because it may not be able to send out, and need to try again next clock cycle
                                pending_direct_message = Some(in_message.clone());
                                need_to_pop_direct_in_channel_from_idx = Some(j);
                            }
                        }
                    }
                }
                // find the most attractive channel for `pending_direct_message`, finish in O(1) on FPGA
                let mut best_channel_for_pending_message_idx = None;
                let pu = &self.processing_units[i];
                match pending_direct_message {
                    Some(DirectMessage{ receiver, is_odd_cardinality_root: _, is_touching_boundary: _ }) => {
                        // find the best channel (peer with smallest distance)
                        for (j, (peer, _out_channel)) in pu.direct_out_channels.iter().enumerate() {
                            best_channel_for_pending_message_idx = Some(match best_channel_for_pending_message_idx {
                                Some(idx) => {
                                    let (last_address, _out_channel) = &pu.direct_out_channels[idx];
                                    if self.get_node_distance(*peer, receiver) < self.get_node_distance(*last_address, receiver) {
                                        j
                                    } else {
                                        idx
                                    }
                                },
                                None => {
                                    j
                                },
                            });
                        }
                    },
                    None => { },
                }
                // save intermediate states, this is not necessary in FPGA, it's only used to mimic direct channels with latency and busy flag
                intermediate_states.push((pending_direct_message, best_channel_for_pending_message_idx, need_to_pop_direct_in_channel_from_idx));
            }
            // first let all nodes retrieve message from direct channels, and then send messages to direct channels
            // have to do this in two iterations, otherwise it's difficult and problematic to check if a deque is full (with length `latency`) or not
            // this is not necessarily two clock cycle in FPGA, because it can always check if the queue is full and then respond to that
            // all the logic here can map to a combinational logic in FPGA (whose longest path must fit into 1 clock cycle)
            // in this way, the parallelism of FPGA is utilized
            for i in 0..nodes_len {
                // send to all direct channels
                let pu = &mut self.processing_units[i];
                let mut pending_message_sent_successfully = false;
                let (mut pending_direct_message, best_channel_for_pending_message_idx, need_to_pop_direct_in_channel_from_idx) = intermediate_states.remove(0);
                for (j, (_peer, out_channel)) in pu.direct_out_channels.iter().enumerate() {
                    let mut out_channel = out_channel.borrow_mut();
                    // push a message only if it's last message is taken by the peer
                    if out_channel.deque.len() < out_channel.latency {
                        out_channel.deque.push_back(if best_channel_for_pending_message_idx == Some(j) {
                            pending_message_sent_successfully = true;  // mark as sent successfully
                            pending_direct_message.take()  // leaving a None in pending_direct_message
                        } else {
                            None
                        });
                    }
                }
                // update internal state
                if pending_message_sent_successfully {  // don't send again next time
                    pu.pending_tell_new_root_cardinality = false;
                    pu.pending_tell_new_root_touching_boundary = false;
                    match need_to_pop_direct_in_channel_from_idx {
                        Some(direct_in_channel_idx) => {
                            let (_peer, in_channel) = &pu.direct_in_channels[direct_in_channel_idx];
                            let mut in_channel = in_channel.borrow_mut();
                            // mark the original message as taken, but not change the amount of elements in the channel
                            in_channel.deque.pop_front().unwrap();
                            in_channel.deque.push_front(None);
                        },
                        None => { },
                    }
                }
            }
            if cfg!(test) {
                self.channels_sanity_check();
            }
            spreading = self.has_message_flying();
        }
        // finally set old root to updated root in O(1) time on FPGA
        clock_cycles += 1;
        for i in 0..nodes_len {
            let pu = &mut self.processing_units[i];
            pu.old_root = pu.updated_root;
        }
        clock_cycles
    }

    /// necessary to run before the first iteration when there exists any erasure error, return the clock cycle and also whether needs to run further iterations
    pub fn reach_consistent_state(&mut self) -> (usize, bool) {
        let mut clock_cycles = 0;
        // first grow the clusters so that erasure errors are considered, O(1) time needed if no erasure error appears
        // during iteration, this corresponds to Union operations in sequential UF decoder and takes average O(log(d)) time but worst O(d) time
        clock_cycles += self.spread_clusters();
        // update the odd cluster state, requires O(log(d)) time in 
        clock_cycles += self.spread_is_odd_cluster();
        // check if there are still odd clusters, if so, then it needs to run further
        let mut has_odd_cluster = false;
        for pu in self.processing_units.iter() {
            if pu.is_odd_cluster {
                has_odd_cluster = true;
                break
            }
        }
        (clock_cycles, has_odd_cluster)
    }

    /// return (clock cycle used in this iteration, need to run another iteration)
    pub fn run_single_iteration(&mut self) -> (usize, bool) {
        let mut clock_cycles = 0;
        // grow the boundary, without sending any messages, takes O(k) time where k is the largest latency between neighbors
        clock_cycles += self.grow_boundary();
        // then reach a consistent state
        let (sub_clock_cycles, has_odd_cluster) = self.reach_consistent_state();
        (clock_cycles + sub_clock_cycles, has_odd_cluster)
    }

    #[allow(dead_code)]
    pub fn detailed_print_run_to_stable(&mut self, detailed_print: bool) -> usize {
        let (mut clock_cycles, mut has_odd_cluster) = self.reach_consistent_state();
        if detailed_print {
            self.debug_print();
        }
        while has_odd_cluster {
            let (sub_clock_cycles, sub_has_odd_cluster) = self.run_single_iteration();
            if detailed_print {
                println!("clock cycle: {}, has_odd_cluster: {}", sub_clock_cycles, sub_has_odd_cluster);
                self.debug_print();
            }
            has_odd_cluster = sub_has_odd_cluster;
            clock_cycles += sub_clock_cycles;
        }
        if detailed_print {
            println!("clock cycle: {}", clock_cycles);
        }
        clock_cycles
    }

    #[allow(dead_code)]
    pub fn run_to_stable(&mut self) -> usize {
        self.detailed_print_run_to_stable(false)
    }

    /// only for debugging
    fn debug_print(&self) {
        println!("[debug print start]");
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let node = &self.nodes[i];
            let pu = &self.processing_units[i];
            let updated_root_user_data = &self.nodes[pu.updated_root].user_data;
            let old_root_user_data = &self.nodes[pu.old_root].user_data;
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
                let neighbor = &pu.neighbors[j];
                let edge = neighbor.link.borrow();
                let neighbor_user_data = &self.nodes[neighbor.address].user_data;
                let string = format!("{:?}[{}/{}] ", neighbor_user_data, edge.increased, edge.length);
                neighbor_string.push_str(string.as_str());
            }
            let debug_cardinality_string = if pu.updated_root == i { format!("[{}]", pu.debug_cardinality) } else { format!("   ") };
            println!("{:?} ∈ updated {:?} {} old {:?} {} {} {} {} {} n: {}", node.user_data, updated_root_user_data, debug_cardinality_string, 
                old_root_user_data, error_symbol, odd_cluster_symbol,
                touching_boundary_symbol, odd_cardinality_symbol, boundary_string, neighbor_string);
        }
        println!("[debug print end]");
    }

    pub fn dump_print_input(&self, id:usize) {
        println!("[dump print start] {}", id);
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("input.txt")
            .unwrap();
        if let Err(e) = writeln!(file, "{:08X}", id) {
            eprintln!("Couldn't write to file: {}", e);
        }
        let nodes_len = self.nodes.len();
        for i in 0..nodes_len {
            let node = &self.nodes[i];
            let error_val = if node.is_error_syndrome { 1 } else { 0 };
            // println!("{}", error_val);
            if let Err(e) = writeln!(file, "{:08X}", error_val) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }

    pub fn dump_print_output(&self, id : usize) {
        println!("[dump print output] {}", id);
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("output.txt")
            .unwrap();
        if let Err(e) = writeln!(file, "{:08X}", id) {
            eprintln!("Couldn't write to file: {}", e);
        }
        let nodes_len = self.processing_units.len();
        for i in 0..nodes_len {
            let node = &self.processing_units[i];
            // println!("{} {}", (node.updated_root)/4, (node.updated_root)%4);
            if let Err(e) = writeln!(file, "{:04X}{:04X}", (node.updated_root)/4, (node.updated_root)%4) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
}

/// create nodes for standard planar code (2d, perfect measurement condition). return only X stabilizers or only Z stabilizers.
/// return (nodes, position_to_index, neighbors), the fast channel should be empty, which is Vec::new()
pub fn make_standard_planar_code_2d_nodes_no_fast_channel(d: usize, is_x_stabilizers: bool) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>,
        Vec<InputNeighbor>) {
    let (nodes, position_to_index, neighbors, _fast_channels) = make_standard_planar_code_2d_nodes(d, is_x_stabilizers, 0);
    (nodes, position_to_index, neighbors)
}

/// create nodes for standard planar code (2d, perfect measurement condition). return only X stabilizers or only Z stabilizers.
/// return (nodes, position_to_index, neighbors, fast_channels), the fast channel is build every (fast_channel_interval) ^ k distance
/// fast_channel_interval = 0 will generate no fast channels
pub fn make_standard_planar_code_2d_nodes(d: usize, is_x_stabilizers: bool, fast_channel_interval: usize) -> (Vec<InputNode<(usize, usize)>>,
HashMap<(usize, usize), usize>, Vec<InputNeighbor>, Vec<InputFastChannel>) {
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
    let mut fast_channels = Vec::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            for (di, dj) in [(2, 0), (0, 2)].iter() {
                let ni = i + di;
                let nj = j + dj;
                if ni <= 2*d-2 && nj <= 2*d-2 {
                    neighbors.push(InputNeighbor {
                        a: position_to_index[&(i, j)],
                        b: position_to_index[&(ni, nj)],
                        increased: 0,
                        length: 2,
                        latency: 1,
                    });
                }
            }
            if fast_channel_interval > 1 {
                // build fast channels to bottom direction
                let mut interval = fast_channel_interval;
                loop {
                    let fi = i + interval;
                    if fi <= 2*d-2 {
                        fast_channels.push(InputFastChannel {
                            a: position_to_index[&(i, j)],
                            b: position_to_index[&(fi, j)],
                            latency: 1,
                        })
                    } else {
                        break
                    }
                    interval *= fast_channel_interval;
                }
                // build fast channels to right direction
                let mut interval = fast_channel_interval;
                loop {
                    let fj = j + interval;
                    if fj <= 2*d-2 {
                        fast_channels.push(InputFastChannel {
                            a: position_to_index[&(i, j)],
                            b: position_to_index[&(i, fj)],
                            latency: 1,
                        })
                    } else {
                        break
                    }
                    interval *= fast_channel_interval;
                }
            }
        }
    }
    (nodes, position_to_index, neighbors, fast_channels)
}

pub fn manhattan_distance_standard_planar_code_2d_nodes(a: &(usize, usize), b: &(usize, usize)) -> usize {
    let (i1, j1) = *a;
    let (i2, j2) = *b;
    let di = (i1 as isize - i2 as isize).abs() as usize;
    let dj = (j1 as isize - j2 as isize).abs() as usize;
    assert!(di % 2 == 0 && dj % 2 == 0, "cannot compute cost between different types of stabilizers");
    (di + dj) / 2
}

pub fn compare_standard_planar_code_2d_nodes(a: &(usize, usize), b: &(usize, usize)) -> Ordering {
    let (i1, j1) = *a;
    let (i2, j2) = *b;
    if i1 < i2 {
        Ordering::Less
    } else if i1 > i2 {
        Ordering::Greater
    } else {
        j1.cmp(&j2)
    }
}

pub fn get_standard_planar_code_2d_left_boundary_cardinality(d: usize, position_to_index: &HashMap<(usize, usize), usize>
        , decoder: &DistributedUnionFind<(usize, usize)>, get_top_boundary_instead: bool) -> usize {
    let mut boundary_cardinality = 0;
    let mut counted_sets = HashSet::new();
    for index in (0..=2*d-2).step_by(2) {
        let i = if get_top_boundary_instead { 1 } else { index };
        let j = if get_top_boundary_instead { index } else { 1 };
        let index = position_to_index[&(i, j)];
        let pu = &decoder.processing_units[index];
        let root = pu.updated_root;
        if counted_sets.get(&root).is_none() {  // every set should only be counted once
            let node = &decoder.nodes[index];
            if pu.boundary_increased >= node.boundary_cost.unwrap() {  // only when this node is bleeding into the boundary
                let root_pu = &decoder.processing_units[root];
                if root_pu.is_odd_cardinality {  // connect to boundary only if the cardinality is odd
                    counted_sets.insert(root);
                    boundary_cardinality += 1;
                }
            }
        }
    }
    boundary_cardinality
}

/// return `(has_x_logical_error, has_z_logical_error, cycle)`
pub fn run_given_offer_decoder_instance_with_cycle(decoder: &mut offer_decoder::OfferDecoder, fast_channel_interval: usize) -> (bool, bool, usize) {
    let d = decoder.d;
    decoder.error_changed();
    // decode X errors
    let (mut nodes, position_to_index, neighbors, fast_channels) = make_standard_planar_code_2d_nodes(d, true, fast_channel_interval);
    for i in (0..=2*d-2).step_by(2) {
        for j in (1..=2*d-3).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].is_error_syndrome = true;
            }
        }
    }
    let mut uf_decoder = DistributedUnionFind::new(nodes, neighbors, fast_channels, manhattan_distance_standard_planar_code_2d_nodes,
        compare_standard_planar_code_2d_nodes);
    let cycle_x = uf_decoder.run_to_stable();
    let left_boundary_cardinality = get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &uf_decoder, false)
        + decoder.origin_error_left_boundary_cardinality();
    let has_x_logical_error = left_boundary_cardinality % 2 == 1;
    // decode Z errors
    let (mut nodes, position_to_index, neighbors, fast_channels) = make_standard_planar_code_2d_nodes(d, false, fast_channel_interval);
    for i in (1..=2*d-3).step_by(2) {
        for j in (0..=2*d-2).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].is_error_syndrome = true;
            }
        }
    }
    let mut uf_decoder = DistributedUnionFind::new(nodes, neighbors, fast_channels, manhattan_distance_standard_planar_code_2d_nodes,
        compare_standard_planar_code_2d_nodes);
    let cycle_z = uf_decoder.run_to_stable();
    let top_boundary_cardinality = get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &uf_decoder, true)
        + decoder.origin_error_top_boundary_cardinality();
    let has_z_logical_error = top_boundary_cardinality % 2 == 1;
    (has_x_logical_error, has_z_logical_error, std::cmp::max(cycle_x, cycle_z))
}

/// return `(has_x_logical_error, has_z_logical_error, cycle)`
pub fn run_given_offer_decoder_instance_no_fast_channel_with_cycle(decoder: &mut offer_decoder::OfferDecoder) -> (bool, bool, usize) {
    run_given_offer_decoder_instance_with_cycle(decoder, 0)
}

/// return `(has_x_logical_error, has_z_logical_error)`
pub fn run_given_offer_decoder_instance_no_fast_channel(decoder: &mut offer_decoder::OfferDecoder) -> (bool, bool) {
    let (has_x_logical_error, has_z_logical_error, _cycle) = run_given_offer_decoder_instance_no_fast_channel_with_cycle(decoder);
    (has_x_logical_error, has_z_logical_error)
}

/// return (nodes, position_to_index, neighbors)
pub fn make_decoder_given_ftqec_model(model: &ftqec::PlanarCodeModel, stabilizer: QubitType, fast_channel_interval: usize) ->
        (Vec<InputNode<(usize, usize, usize)>>, HashMap<(usize, usize, usize), usize>, Vec<InputNeighbor>, Vec<InputFastChannel>) {
    assert!(stabilizer == QubitType::StabZ || stabilizer == QubitType::StabX, "stabilizer must be either StabZ or StabX");
    assert!(fast_channel_interval <= 1, "fast channel not supported yet");
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            position_to_index.insert((t, i, j), nodes.len());
            nodes.push(InputNode {
                user_data: (t, i, j),
                is_error_syndrome: false,
                boundary_cost: if node.boundary.is_some() { Some(2) } else { None },
            });
        }
    });
    model.iterate_measurement_errors(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            nodes[position_to_index[&(t, i, j)]].is_error_syndrome = true;
        }
    });
    let mut neighbors = Vec::new();
    let fast_channels = Vec::new();
    model.iterate_measurement_stabilizers(|t, i, j, node| {
        if t > 12 && node.qubit_type == stabilizer {  // ignore the bottom layer
            let idx = position_to_index[&(t, i, j)];
            for edge in node.edges.iter() {
                if edge.t > 12 {
                    let peer_idx = position_to_index[&(edge.t, edge.i, edge.j)];
                    if idx < peer_idx {  // remove duplicated neighbors
                        neighbors.push(InputNeighbor {
                            a: idx,
                            b: peer_idx,
                            increased: 0,
                            length: 2,
                            latency: 1,
                        });
                    }

                } else {
                    nodes[idx].boundary_cost = Some(2);  // viewing the bottom layer as boundary
                }
            }
        }
    });
    assert!(neighbors.len() > 0, "ftqec model may not have `build_graph` called, so the neighbor connections have not built yet");
    (nodes, position_to_index, neighbors, fast_channels)
}

#[allow(dead_code)]
pub fn get_standard_planar_code_3d_boundary_cardinality(stabilizer: QubitType, position_to_index: &HashMap<(usize, usize, usize), usize>
        , decoder: &DistributedUnionFind<(usize, usize, usize)>) -> usize {
    let mut boundary_cardinality = 0;
    let mut counted_sets = HashSet::new();
    let mut counting_indices = Vec::new();
    for ((_t, i, j), index) in position_to_index.iter() {
        match stabilizer {
            QubitType::StabZ => {
                if *j == 1 {
                    counting_indices.push(*index);
                }
            },
            QubitType::StabX => {
                if *i == 1 {
                    counting_indices.push(*index);
                }
            },
            _ => panic!("unsupported stabilizer type")
        }
    }
    for index in counting_indices.into_iter() {
        let pu = &decoder.processing_units[index];
        let root = pu.updated_root;
        if counted_sets.get(&root).is_none() {  // every set should only be counted once
            let node = &decoder.nodes[index];
            if pu.boundary_increased >= node.boundary_cost.unwrap() {  // only when this node is bleeding into the boundary
                let root_pu = &decoder.processing_units[root];
                if root_pu.is_odd_cardinality {  // connect to boundary only if the cardinality is odd
                    counted_sets.insert(root);
                    boundary_cardinality += 1;
                }
            }
        }
    }
    boundary_cardinality
}

pub fn manhattan_distance_standard_planar_code_3d_nodes(a: &(usize, usize, usize), b: &(usize, usize, usize)) -> usize {
    let (t1, i1, j1) = *a;
    let (t2, i2, j2) = *b;
    let dt_origin = (t1 as isize - t2 as isize).abs() as usize;
    assert!(dt_origin % 6 == 0, "dt should only be 6x");
    let dt = dt_origin / 6;
    let di = (i1 as isize - i2 as isize).abs() as usize;
    let dj = (j1 as isize - j2 as isize).abs() as usize;
    assert!(di % 2 == 0 && dj % 2 == 0, "cannot compute cost between different types of stabilizers");
    (di + dj) / 2 + dt
}

pub fn compare_standard_planar_code_3d_nodes(a: &(usize, usize, usize), b: &(usize, usize, usize)) -> Ordering {
    let (t1, i1, j1) = *a;
    let (t2, i2, j2) = *b;
    let sum1 = t1 / 6 + i1 / 2 + j1 / 2;
    let sum2 = t2 / 6 + i2 / 2 + j2 / 2;
    if sum1 < sum2 {
        Ordering::Less
    } else if sum1 > sum2 {
        Ordering::Greater
    } else {
        if i1 < i2 {
            Ordering::Less
        } else if i1 > i2 {
            Ordering::Greater
        } else {
            j1.cmp(&j2)
        }
    }
}

pub fn build_distributed_union_find_given_uf_decoder_3d<>(decoder: &union_find_decoder::UnionFindDecoder<(usize, usize, usize)>) ->
        (DistributedUnionFind<(usize, usize, usize)>, HashMap<(usize, usize, usize), usize>) {
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    for node in decoder.nodes.iter() {
        let position = node.node.user_data.clone();
        position_to_index.insert(position.clone(), nodes.len());
        nodes.push(InputNode {
            user_data: position,
            is_error_syndrome: node.node.is_error_syndrome,
            boundary_cost: node.node.boundary_cost.clone(),
        });
    }
    let mut neighbors = Vec::new();
    for neighbor in decoder.input_neighbors.iter() {
        neighbors.push(InputNeighbor {
            a: neighbor.a,
            b: neighbor.b,
            increased: 0,
            length: neighbor.length,
            latency: 1,
        });
    }
    let duf_decoder = DistributedUnionFind::new(nodes, neighbors, vec![], manhattan_distance_standard_planar_code_3d_nodes,
        compare_standard_planar_code_3d_nodes);
    (duf_decoder, position_to_index)
}

pub fn copy_state_back_to_union_find_decoder(decoder: &mut union_find_decoder::UnionFindDecoder<(usize, usize, usize)>, 
        duf_decoder: &DistributedUnionFind<(usize, usize, usize)>) {
    // copy root, but the root might be changed internally by UnionFind library
    assert_eq!(decoder.nodes.len(), duf_decoder.nodes.len(), "they should have the same number of nodes");
    for a in 0..decoder.nodes.len() {
        let duf_processing_unit = &duf_decoder.processing_units[a];
        let b = duf_processing_unit.updated_root;
        decoder.union_find.union(a, b);
    }
    // copy cardinality and is_touching_boundary
    for a in 0..decoder.nodes.len() {
        let duf_processing_unit = &duf_decoder.processing_units[a];
        let duf_updated_root = duf_processing_unit.updated_root;
        let duf_root_processing_unit = &duf_decoder.processing_units[duf_updated_root];
        let uf_union_node = decoder.union_find.get_mut(a);
        uf_union_node.cardinality = duf_root_processing_unit.debug_cardinality;  // accurate cardinality
        // uf_union_node.cardinality = if duf_root_processing_unit.is_odd_cardinality { 1 } else { 0 };  // or just use parity of cardinality
        uf_union_node.is_touching_boundary = duf_root_processing_unit.is_touching_boundary;
    }
    // copy boundary increase and boundary state
    for a in 0..decoder.nodes.len() {
        let duf_processing_unit = &duf_decoder.processing_units[a];
        let uf_decoder_node = &mut decoder.nodes[a];
        uf_decoder_node.boundary_increased = duf_processing_unit.boundary_increased;
    }
    // copy neighbor increase and neighbor state
    for a in 0..decoder.nodes.len() {
        let duf_processing_unit = &duf_decoder.processing_units[a];
        let uf_decoder_node = &mut decoder.nodes[a];
        for duf_neighbor in duf_processing_unit.neighbors.iter() {
            let uf_neighbor_index = uf_decoder_node.neighbor_index[&duf_neighbor.address];
            let uf_neighbor_partial_edge = &mut uf_decoder_node.neighbors[uf_neighbor_index];
            assert_eq!(uf_neighbor_partial_edge.address, duf_neighbor.address);
            assert_eq!(uf_neighbor_partial_edge.length, duf_neighbor.link.borrow().length);
            uf_neighbor_partial_edge.increased = duf_neighbor.link.borrow().increased;
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test distributed_union_find_decoder_test_case_1 -- --nocapture` to run specific test
    
    fn make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d: usize) -> (Vec<InputNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<InputNeighbor>) {
        make_standard_planar_code_2d_nodes_no_fast_channel(d, true)
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
        DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes, compare_standard_planar_code_2d_nodes);
    }

    #[test]
    fn distributed_union_find_decoder_sanity_check_2() {
        let (nodes, _position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        let distributed_union_find = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        distributed_union_find.debug_print();
    }

    #[test]
    fn distributed_union_find_decoder_sanity_check_3() {
        // test `spread_is_odd_cluster` function
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = true;  // test touching boundary
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;  // test single error syndrome
        nodes[position_to_index[&(4, 1)]].is_error_syndrome = true;  // test 2 matching together
        nodes[position_to_index[&(4, 3)]].is_error_syndrome = true;
        let mut distributed_union_find = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,          
            compare_standard_planar_code_2d_nodes);
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
        distributed_union_find.debug_print();
    }

    #[test]
    fn distributed_union_find_decoder_sanity_check_4() {
        // test `spread_clusters` function
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(3);
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;  // test single error syndrome
        let mut distributed_union_find = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,          
            compare_standard_planar_code_2d_nodes);
        distributed_union_find.debug_print();
        distributed_union_find.reach_consistent_state();
        distributed_union_find.debug_print();
        let (_, need_to_run_another) = distributed_union_find.run_single_iteration();
        assert_eq!(need_to_run_another, true, "1 iteration is not enough");
        distributed_union_find.debug_print();
        let (_, need_to_run_another) = distributed_union_find.run_single_iteration();
        assert_eq!(need_to_run_another, false, "2 iterations should be enough");
        distributed_union_find.debug_print();
    }

    #[test]
    fn distributed_union_find_decoder_bug_find_1() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        nodes[position_to_index[&(4, 5)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,          
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(0, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_1() {
        let d = 3;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        assert_eq!(nodes.len(), 6, "d=3 should have 6 nodes");
        assert_eq!(neighbors.len(), 7, "d=3 should have 7 direct neighbor connections");
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(0, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_2() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 7)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(0, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_3() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(1, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_4() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        nodes[position_to_index[&(4, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(6, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(6, 7)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        decoder.debug_print();
        assert_eq!(1, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_5() {
        let d = 5;
        let (mut nodes, position_to_index, neighbors) = make_standard_planar_code_2d_nodes_no_fast_channel_only_x(d);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = false;
        nodes[position_to_index[&(0, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 5)]].is_error_syndrome = false;
        nodes[position_to_index[&(0, 7)]].is_error_syndrome = false;
        nodes[position_to_index[&(2, 1)]].is_error_syndrome = false;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = false;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 7)]].is_error_syndrome = false;
        nodes[position_to_index[&(4, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(4, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(4, 5)]].is_error_syndrome = false;
        nodes[position_to_index[&(4, 7)]].is_error_syndrome = false;
        nodes[position_to_index[&(6, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(6, 3)]].is_error_syndrome = false;
        nodes[position_to_index[&(6, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(6, 7)]].is_error_syndrome = true;
        nodes[position_to_index[&(8, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(8, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(8, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(8, 7)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, Vec::new(), manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        decoder.debug_print();
        assert_eq!(1, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_case_6() {
        // test fast channels, same as test case 3
        let d = 5;
        let (mut nodes, position_to_index, neighbors, fast_channels) = make_standard_planar_code_2d_nodes(d, true, 2);
        nodes[position_to_index[&(0, 1)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(0, 5)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 3)]].is_error_syndrome = true;
        nodes[position_to_index[&(2, 5)]].is_error_syndrome = true;
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, fast_channels, manhattan_distance_standard_planar_code_2d_nodes,
            compare_standard_planar_code_2d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(1, get_standard_planar_code_2d_left_boundary_cardinality(d, &position_to_index, &decoder, false)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_build_given_ftqec_model_1() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(measurement_rounds, d);
        model.set_phenomenological_error_with_perfect_initialization(p);
        model.build_graph(ftqec::weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        model.add_error_at(el2t(0), 0, 2, &ErrorType::X).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.add_error_at(el2t(1), 2, 3, &ErrorType::X).expect("error rate = 0 here");  // measurement error (detected by this and next layer)
        model.propagate_error();
        let (nodes, position_to_index, neighbors, fast_channels) = make_decoder_given_ftqec_model(&model, QubitType::StabZ, 1);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, fast_channels, manhattan_distance_standard_planar_code_3d_nodes,
            compare_standard_planar_code_3d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(0, get_standard_planar_code_3d_boundary_cardinality(QubitType::StabZ, &position_to_index, &decoder)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_build_given_ftqec_model_2() {
        let measurement_rounds = 3;
        let d = 3;
        let p = 0.01;  // physical error rate
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(measurement_rounds, d);
        model.set_phenomenological_error_with_perfect_initialization(p);
        model.build_graph(ftqec::weight_autotune);
        let el2t = |layer| layer * 6usize + 18 - 1;  // error from layer 0 is at t = 18-1 = 17
        model.add_error_at(el2t(0), 0, 0, &ErrorType::X).expect("error rate = 0 here");  // data qubit error (detected by next layer)
        model.propagate_error();
        let (nodes, position_to_index, neighbors, fast_channels) = make_decoder_given_ftqec_model(&model, QubitType::StabZ, 1);
        assert_eq!(d * (d - 1) * measurement_rounds, nodes.len());
        assert_eq!((measurement_rounds * (d * (d - 1) * 2 - d - (d - 1)) + (measurement_rounds - 1) * d * (d - 1)), neighbors.len());
        let mut decoder = DistributedUnionFind::new(nodes, neighbors, fast_channels, manhattan_distance_standard_planar_code_3d_nodes,
            compare_standard_planar_code_3d_nodes);
        decoder.detailed_print_run_to_stable(true);
        assert_eq!(1, get_standard_planar_code_3d_boundary_cardinality(QubitType::StabZ, &position_to_index, &decoder)
            , "cardinality of one side of boundary determines if there is logical error");
    }

    #[test]
    fn distributed_union_find_decoder_test_xzzx_code_1() {
        let di = 3;
        let dj = 3;
        let measurement_rounds = 3;
        let bias_zeta = 100.;  // definition of zeta see arXiv:2104.09539v1
        let p = 0.006;
        let max_half_weight = 4;
        let mut model = ftqec::PlanarCodeModel::new_standard_XZZX_code_rectangle(measurement_rounds, di, dj);
        model.apply_error_model(&ErrorModelName::GenericBiasedWithBiasedCX, None, p, bias_zeta, 0.);
        model.build_graph(ftqec::weight_autotune);
        // model.optimize_correction_pattern();  // no need if not building corrections
        // model.build_exhausted_path();
        let use_random_error = true;
        if use_random_error {
            let mut rng = thread_rng();
            let error_count = model.generate_random_errors(|| rng.gen::<f64>());
            println!("error_count: {}", error_count);
        }
        model.propagate_error();
        let measurement = model.generate_measurement();
        let decoders = union_find_decoder::suboptimal_matching_by_union_find_given_measurement_build_decoders(&model, &measurement
            , &ftqec::DetectedErasures::new(di, dj), max_half_weight);
        for mut decoder in decoders.into_iter() {
            // build distributed union-find decoder from union-find decoder
            let (mut duf_decoder, _position_to_index) = build_distributed_union_find_given_uf_decoder_3d(&decoder);
            duf_decoder.detailed_print_run_to_stable(true);
            copy_state_back_to_union_find_decoder(&mut decoder, &duf_decoder);
        }
    }

}
