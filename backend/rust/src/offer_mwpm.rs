//! # Offer Algorithm: Minimum Weight Perfect Matching for Topological QEC Codes
//!
//! ## Introduction
//!
//! Minimum weight perfect matching (MWPM) solves a perfect matching with minimum sum of weight given a weighted graph.
//! It's very useful in quantum error correction, where the degeneracy of quantum codes cannot be handled efficiently by traditional decoders like belief propagation.
//! Initially proposed by J. Edmonds. and further developed by others, blossom algorithm is an efficient sequential algorithm for solving MWPM.
//! Quantum error correction (QEC) needs a super low-latency MWPM solver in order to fit into the timing constraint, which can be possibly realized by parallel programming.
//! Offer algorithm is thus proposed to solve MWPM problem in parallel efficiently in the settings of quantum error correction codes.
//!
//! ## Interface
//!
//! The interface of offer algorithm is a little bit different from standard MWPM solver, but it's compatible with standard one.
//! It requires some auxiliary information to assist decoding, which exists in most topological quantum codes like surface code, namely sparseness and locality.
//! Sparseness means each checker (or stabilizer in QEC) only connect to small number of nodes (or data qubits in QEC).
//! Locality means the checker and data qubits can be arranged in space so that there is only connection between neighbors.
//! These two attributes together leads to a conclusion that, with single data qubit error, each stabilizer only match with a constant small number of neighbor stabilizers.
//! This is an auxiliary information to the offer algorithm, which is called "direct neighbors".
//! Standard MWPM problem may simply connect all other check nodes as direct neighbors, however this will impact the performance a lot.
//! Another difference is that, the nodes input to the offer algorithm are not necessarily all going to be matched.
//! That means, each node has a flag indicating whether it's going to be matched, and those who are not going to be matched only work as assisting nodes.
//! This design corresponds to the topological quantum codes, where the topology of checker are constant, but only a few of them has error syndrome and needs to be matched.
//! For standard MWPM problem one can just set all input nodes as going to be matched.
//! Also, offer algorithm allows those going-to-be-matched nodes to remain unmatched, with a specific cost called "boundary cost".
//! This attribute is extremely suitable for topological quantum codes that are not periodic where each stabilizer can match to "virtual boundary" with some cost.
//! Standard MWPM can simply set this boundary cost to the +inf to avoid matching to boundary.
//!
//! - Initialization
//!   - nodes: Vec\<Node\>, each node has the following field
//!     - user_data: \<U\>, could be anything for user-defined functions to be used
//!     - going_to_be_matched: bool
//!     - boundary_cost: f64, the cost of matching node to boundary
//!   - direct_neighbors: Vec\<(usize, usize)\>, connection of direct neighbors (order doesn't matter)
//!   - max_path_length: usize, the maximum length to search augmenting path (can be set to code distance for surface code)
//!   - cost_of_matching: fn(a: \<U\>, b: \<U\>) -> f64, the cost of matching two nodes
//!   - seed: u64, the seed to Xoroshiro128StarStar random number generator, to remain reproducible result
//!
//! After initialization, the algorithm will instantiate multiple processing unit (PU), each corresponds to a node.
//! Each PU can be execute individually, or all PUs can execute until stable, depending on the granularity of the simulation.
//! Ultimately, all those PUs can be instantiated in FPGA(s) in order to reach a low decoding latency.
//!

use std::collections::HashMap;
use super::rand_core::SeedableRng;
use super::reproducible_rand::{Xoroshiro128StarStar};
use super::serde::{Serialize, Deserialize};

#[derive(Derivative, Serialize, Deserialize)]
#[derivative(Debug)]
pub struct OfferAlgorithm<U> {
    /// maximum length of augmenting path searching
    pub max_path_length: usize,
    #[derivative(Debug="ignore")]
    #[serde(skip_serializing)]
    #[serde(default = "default_cost_func")]
    #[serde(skip_deserializing)]
    /// cost function given two nodes' user data
    pub cost_of_matching: Box<dyn Fn(&U, &U) -> f64>,
    /// statistics: message flying in a single round
    pub message_count_single_round: usize,
    /// statistics: message flying in total
    pub message_count: usize,
    /// statistics: ignoring probabilistic accept mechanism, is there possible acceptance of offer?
    pub has_potential_acceptance: bool,
    /// probabilistic accept helps to solve conflicts
    pub disable_probabilistic_accept: bool,
    /// seed of reproducible random generator
    pub reproducible_error_generator_seed: u64,
    pub reproducible_error_generator: Xoroshiro128StarStar,
    /// processing units, each one corresponding to a node of the input graph
    pub processing_units: Vec<ProcessingUnit<U>>,
}

fn default_cost_func<U>() -> Box<dyn Fn(&U, &U) -> f64> {
    Box::new(|_a: &U, _b: &U| -> f64 {
        panic!("cost function doesn't exists, this may happen after deserializing from JSON without set the cost function properly");
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingUnit<U> {
    /// the corresponding node in the input graph
    pub node: OfferNode<U>,
    /// directly connected neighbors
    pub direct_neighbors: Vec<usize>,
    /// only for nodes to be matched
    pub mailbox: Vec<Message>,
    /// only for nodes to be matched
    pub out_queue: Vec<OutMessage>,
    /// only for nodes to be matched
    pub active_timestamp: usize,
    /// cache to avoid redundant message passing (could be simply BRAM in FPGA)
    pub cache: HashMap::<usize, CachedOffer>,
    /// this is set when set flag `is_waiting_contract`, as the next hop of `Contract` or `RefuseAcceptance` message
    pub broker_next_hop: Option<usize>,
    /// the node is "locked" when waiting for contract, only for nodes to be matched
    pub is_waiting_contract: bool,
    /// `None` for matching with boundary 
    pub match_with: Option<usize>,
    /// the probability of taking one offer when it's an augmenting one, known as probabilistic accept
    pub accept_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OfferNode<U> {
    /// user defined data structure which is used in cost computation
    pub user_data: U,
    /// is the node going to be matched? in QEC, it corresponds to error syndrome
    pub going_to_be_matched: bool,
    /// the cost of matching to boundary
    pub boundary_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    MatchOffer {
        /// newer timestamp will overwrite older one in cache, also node will never accept offer with older timestamp than the active timestamp
        timestamp: usize,
        /// the source of offer, only those going-to-be-matched nodes will be the source
        source: usize,
        /// all the nodes in the path except for source
        brokers: Vec::<usize>,
        /// sending between matched pairs, `true` corresponds to `BrokeredOffer` in older code
        brokering: bool,
    },
    AcceptOffer {
        /// copy the timestamp of `MatchOffer`
        timestamp: usize,
        /// copy the source of offer, and `AcceptOffer` will end at the source
        source: usize,
        /// all the nodes in the path except for source and target
        brokers: Vec::<usize>,
        /// the one who take the offer
        target: usize,
        /// sending between matched pairs, `true` corresponds to `BrokeredAcceptOffer` in older code
        brokering: bool,
    },
    RefuseAcceptance {
        /// the one who take the offer
        target: usize,
    },
    Contract {
        /// the one who take the offer
        target: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedOffer {
    /// keep a copy of timestamp, so that messages with newer timestamp will overwrite older one in cache 
    pub timestamp: usize,
    /// the minimum cost among latest messages
    pub cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutMessage {
    /// the receiver of the message
    pub receiver: usize,
    /// the content of the message
    pub message: Message,
}

impl<U> OfferNode<U> {
    pub fn new(user_data: U, going_to_be_matched: bool, boundary_cost: f64) -> Self {
        Self {
            user_data: user_data,
            going_to_be_matched: going_to_be_matched,
            boundary_cost: boundary_cost,
        }
    }
}

impl<U> OfferAlgorithm<U> {
    pub fn new(mut nodes: Vec<OfferNode<U>>, direct_neighbors: Vec<(usize, usize)>, max_path_length: usize
            , cost_of_matching: impl Fn(&U, &U) -> f64 + 'static, seed: u64) -> Self {
        let mut processing_units: Vec<_> = nodes.drain(..).map(|node| {
            ProcessingUnit {
                node: node,
                direct_neighbors: Vec::new(),
                mailbox: Vec::new(),
                out_queue: Vec::new(),
                active_timestamp: 0,
                cache: HashMap::new(),
                broker_next_hop: None,
                is_waiting_contract: false,
                match_with: None,
                accept_probability: 1.,
            }
        }).collect();
        for (a, b) in direct_neighbors.iter() {
            processing_units[*a].direct_neighbors.push(*b);
            processing_units[*b].direct_neighbors.push(*a);
        }
        // remove duplicate direct neighbors for each PU
        for pu in processing_units.iter_mut() {
            pu.direct_neighbors.sort_unstable();  // may not preserve the order of equal elements
            pu.direct_neighbors.dedup();
        }
        Self {
            processing_units: processing_units,
            max_path_length: max_path_length,
            cost_of_matching: Box::new(cost_of_matching),
            message_count_single_round: 0,
            message_count: 0,
            has_potential_acceptance: false,
            disable_probabilistic_accept: false,
            reproducible_error_generator_seed: seed,
            reproducible_error_generator: Xoroshiro128StarStar::seed_from_u64(seed),
        }
    }

    /// execute a single processing unit, return the message processed in this round
    pub fn single_pu_execute(&mut self, pu_idx: usize, only_process_one_message: bool) -> usize {
        let pu = &mut self.processing_units[pu_idx];
        if pu.node.going_to_be_matched == false {
            return 0;
        }
        // read message
        let mut message_processed = 0;
        let reproducible_error_generator = &mut self.reproducible_error_generator;
        let mut random_generate = || { reproducible_error_generator.next_f64() };
        while self.processing_units[pu_idx].mailbox.len() > 0 {
            let pu = &mut self.processing_units[pu_idx];  // have to re-borrow it as mutable
            let message = pu.mailbox.remove(0);  // take the first message in mailbox
            message_processed += 1;
            // TODO: handle message
            // process only one message
            if only_process_one_message {
                break
            }
        }
        message_processed
    }

    /// let single processing unit resend offer and increase the timestamp value 
    pub fn single_pu_resend_offer(&mut self, pu_idx: usize) {
        let pu = &mut self.processing_units[pu_idx];
        if pu.node.going_to_be_matched == false {
            return
        }
        // TODO: send offer
    }

    /// clear single processing unit's out queue by pushing them into the receiver's mailbox
    pub fn single_pu_out_queue_send(&mut self, pu_idx: usize) {
        // send messages from out_queue
        let mut mut_messages = self.processing_units[pu_idx].out_queue.split_off(0);
        for out_message in mut_messages.drain(..) {
            self.message_count_single_round += 1;
            self.message_count += 1;
            let receiver = out_message.receiver;
            let message = out_message.message;
            self.processing_units[receiver].mailbox.push(message);
        }
    }

    /// force breaking the matched node (debugging the algorithm by manually control the initial state)
    pub fn force_break_matched(&mut self, pu_idx1: usize) {
        let pu = &mut self.processing_units[pu_idx1];
        if pu.is_waiting_contract {
            return  // do not break pairs when waiting for contract, otherwise may cause unrecoverable states
        }
        let match_with = match pu.match_with {
            Some(match_with) => match_with,
            None => {
                return
            }
        };
        pu.match_with = None;
        pu.cache = HashMap::new();
        let matched_pu = &mut self.processing_units[match_with];
        matched_pu.match_with = None;
        matched_pu.cache = HashMap::new();
    }

    /// force matching the nodes (debugging the algorithm by manually control the initial state)
    pub fn force_match_nodes(&mut self, pu_idx1: usize, pu_idx2: usize) {
        if pu_idx1 == pu_idx2 { return }  // why match the same node?
        if self.processing_units[pu_idx1].is_waiting_contract || self.processing_units[pu_idx2].is_waiting_contract {
            return  // do not break pairs when waiting for contract, otherwise may cause unrecoverable states
        }
        // break them first
        self.force_break_matched(pu_idx1);
        self.force_break_matched(pu_idx2);
        // connect them
        let pu1 = &mut self.processing_units[pu_idx1];
        pu1.match_with = Some(pu_idx2);
        pu1.cache = HashMap::new();
        let pu2 = &mut self.processing_units[pu_idx2];
        pu2.match_with = Some(pu_idx1);
        pu2.cache = HashMap::new();
    }

    /// get the matching results with the overall cost
    pub fn matching_result(&self) -> (f64, Vec<Option<usize>>) {
        let mut cost = 0.;
        let cost_of_matching = |obj: &Self, idx1: usize, idx2: usize| {
            (obj.cost_of_matching)(&obj.processing_units[idx1].node.user_data, &obj.processing_units[idx2].node.user_data)
        };
        let matchings = (0..self.processing_units.len()).map(|pu_idx| {
            let pu = &self.processing_units[pu_idx];
            if !pu.node.going_to_be_matched {
                return None
            }
            if pu.is_waiting_contract || pu.match_with == None {  // view as not matched
                cost += pu.node.boundary_cost;
                return None
            }
            let match_with = pu.match_with.unwrap();
            let matched_pu = &self.processing_units[match_with];
            if matched_pu.node.going_to_be_matched && !matched_pu.is_waiting_contract && matched_pu.match_with == Some(pu_idx) {
                if pu_idx < match_with {
                    cost += cost_of_matching(self, pu_idx, match_with);
                }
                Some(match_with)
            } else {  // view as not matched
                cost += pu.node.boundary_cost;
                None
            }
        }).collect();
        (cost, matchings)
    }
}
