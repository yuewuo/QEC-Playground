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
use super::offer_decoder;

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
    pub cache: HashMap<usize, CachedOffer>,
    /// this is set when set flag `is_waiting_contract`, as the next hop of `Contract` or `RefuseAcceptance` message
    pub broker_next_hop: Option<usize>,
    /// the node is "locked" when waiting for contract, only for nodes to be matched
    pub is_waiting_contract: bool,
    /// the node may initiating augmenting loop, this is set to `true` only when `is_waiting_contract` is `true`
    pub is_initiating_augmenting_loop: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Message {
    MatchOffer {
        /// newer timestamp will overwrite older one in cache, also node will never accept offer with older timestamp than the active timestamp
        timestamp: usize,
        /// all the nodes in the path, source is at index 0
        path: Vec::<usize>,
        /// sending between matched pairs, `true` corresponds to `BrokeredOffer` in older code
        brokering: bool,
        /// cost until last broker
        cost_to_last_broker: f64,
    },
    AcceptOffer {
        /// copy the timestamp of `MatchOffer`
        timestamp: usize,
        /// all the nodes in the path, source is at index 0
        path: Vec::<usize>,
        /// sending between matched pairs, `true` corresponds to `BrokeredAcceptOffer` in older code
        brokering: bool,
        /// target
        target: usize,
    },
    RefuseAcceptance {
        /// the one who take the offer
        target: usize,
    },
    Contract {
        /// sender of this message
        previous: usize,
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

impl<U: std::fmt::Debug> OfferAlgorithm<U> {
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
                is_initiating_augmenting_loop: false,
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

    /// compute cost of matching given indices
    pub fn cost_of_matching_idx(&self, idx1: usize, idx2: usize) -> f64 {
        (self.cost_of_matching)(&self.processing_units[idx1].node.user_data, &self.processing_units[idx2].node.user_data)
    }

    /// execute a single processing unit, return the message processed in this round
    pub fn single_pu_execute(&mut self, pu_idx: usize, only_process_one_message: bool) -> usize {
        // read message
        let mut message_processed = 0;
        while self.processing_units[pu_idx].mailbox.len() > 0 {
            let message = self.processing_units[pu_idx].mailbox.remove(0);  // take the first message in mailbox
            let pu = &self.processing_units[pu_idx];  // re-borrow it as immutable, so that `self.cost_of_matching_idx` can be called
            message_processed += 1;
            // for debugging
            // let debug_execute_detail = pu_idx == 7 && message == Message::MatchOffer { timestamp: 1, path: vec![9, 5, 6], brokering: false, cost_to_last_broker: -2.0 };
            // if debug_execute_detail { // debug print
            //     println!("debug_execute_detail enabled for message: {:?}", message);
            // }
            match message {
                Message::MatchOffer{ timestamp, ref path, brokering, cost_to_last_broker } => {
                    assert!(path.len() > 0, "path should at least contain the source");
                    let source = *path.first().unwrap();
                    let broker = *path.last().unwrap();
                    // check for loops in the path
                    let mut loop_index = None;
                    for i in 0..path.len() {
                        if pu_idx == path[i] {
                            loop_index = Some(i);
                            break
                        }
                    }
                    if brokering && pu.match_with != Some(broker) {
                        // why should a brokering message sent from node other than the current matching?
                        // this is inconsistent matching state, should just ignore
                    } else if let Some(loop_index) = loop_index {
                        // only matched node can be the starting point of an augmenting loop
                        // augmenting loop must originate from matched node and must have the current active timestamp
                        if loop_index == 0 && path.len() >= 4 && timestamp == pu.active_timestamp && !pu.is_waiting_contract && pu.match_with.is_some()
                                && path.len() % 2 == 0 {  // odd cardinality loop can occur because of inconsistent states in the middle nodes, just ignore it
                            assert_eq!(pu.is_initiating_augmenting_loop, false);
                            let mut is_the_smallest_node = true;
                            // augmenting loop must be established by the smallest node, to avoid conflicts
                            for i in 1..path.len() {
                                if path[i] < pu_idx {
                                    is_the_smallest_node = false;
                                    break
                                }
                            }
                            if is_the_smallest_node {
                                let previous = *path.last().unwrap();
                                let augmenting_cost = cost_to_last_broker - pu.node.boundary_cost + self.cost_of_matching_idx(pu_idx, previous);
                                if augmenting_cost < 0. {
                                    // println!("found augmenting loop path: {:?}, augmenting_cost = {}", path, augmenting_cost);
                                    self.has_potential_acceptance = true;
                                    let accept_this_offer = self.reproducible_error_generator.next_f64() < pu.accept_probability;
                                    // println!("has_potential_acceptance from {}, path: {:?}, with probability {}, take it: {}"
                                    //     , pu_idx, path, pu.accept_probability, accept_this_offer);
                                    if accept_this_offer {
                                        let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                                        pu.is_waiting_contract = true;
                                        pu.is_initiating_augmenting_loop = true;
                                        let mut path = path.clone();
                                        path.push(pu_idx);  // push myself at the end, as the final target
                                        pu.out_queue.push(OutMessage {
                                            receiver: previous,  // send back to the last broker
                                            message: Message::AcceptOffer {
                                                timestamp: timestamp,
                                                path: path,
                                                brokering: false,
                                                target: pu_idx,
                                            },
                                        });
                                    }
                                }
                            }
                        }
                    } else {
                        // the total cost should minus the matching cost if brokering, or should add the matching cost if not brokering
                        let cost = cost_to_last_broker + ( if brokering { -1.0 } else { 1.0 } ) * self.cost_of_matching_idx(pu_idx, broker);
                        let cached_offer = pu.cache.get(&source);
                        let not_cached_this_offer = match cached_offer {
                            Some(cached_offer) => {
                                cached_offer.timestamp < timestamp || cost < cached_offer.cost
                            },
                            None => true,
                        };
                        let should_cache_this_offer = !brokering && not_cached_this_offer;  // only cache offer if it's not brokering
                        let should_broadcast_this_offer = brokering || not_cached_this_offer;  // broadcast offer when not cached or it's brokering
                        // if debug_execute_detail { // debug print
                        //     println!("should_cache_this_offer: {}", should_cache_this_offer);
                        //     println!("cached_offer: {:?}", cached_offer);
                        // }
                        let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                        // cache this offer if not cached
                        if should_cache_this_offer {
                            // if pu_idx == 7 && cost == -1.0 && timestamp == 1 { // debug print
                            //     println!("cache inserted by {:?}", message);
                            // }
                            pu.cache.insert(source, CachedOffer {
                                timestamp: timestamp,
                                cost: cost,
                            });
                        }
                        let pu = &self.processing_units[pu_idx];  // re-borrow it as immutable, so that `self.cost_of_matching_idx` can be called
                        // may broker this offer
                        if let Some(match_with) = pu.match_with {
                            // when cost < cached_offer.cost, the farther node will not broker this offer backward
                            // and also, this makes an infinite ping-pong between the matched pairs impossible, which is harmful to the system
                            if !brokering && should_cache_this_offer && path.len() < self.max_path_length && pu.node.going_to_be_matched {
                                let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                                let mut path = path.clone();
                                path.push(pu_idx);  // add myself to the path
                                pu.out_queue.push(OutMessage {
                                    receiver: match_with,
                                    message: Message::MatchOffer {
                                        timestamp: timestamp,
                                        path: path,
                                        brokering: true,
                                        cost_to_last_broker: cost,
                                    },
                                });
                            }
                        }
                        // propagate this offer if cache is updated
                        let pu = &self.processing_units[pu_idx];  // re-borrow it as immutable, so that `self.cost_of_matching_idx` can be called
                        if should_broadcast_this_offer {
                            let mut new_path = path.clone();
                            let mut new_cost_to_last_broker = cost_to_last_broker;
                            let mut new_broker = broker;
                            if brokering {
                                new_path.push(pu_idx);
                                new_cost_to_last_broker -= self.cost_of_matching_idx(pu_idx, broker);
                                new_broker = pu_idx;
                            }
                            let direct_neighbors = pu.direct_neighbors.clone();
                            for receiver in direct_neighbors.iter() {
                                if self.cost_of_matching_idx(pu_idx, new_broker) < self.cost_of_matching_idx(*receiver, new_broker)
                                        && new_path.len() <= self.max_path_length {
                                    let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                                    pu.out_queue.push(OutMessage {
                                        receiver: *receiver,
                                        message: Message::MatchOffer {
                                            timestamp: timestamp,
                                            path: new_path.clone(),
                                            brokering: false,
                                            cost_to_last_broker: new_cost_to_last_broker,
                                        },
                                    });
                                }
                            }
                        }
                        // take this offer if it is an augmenting path
                        let pu = &self.processing_units[pu_idx];  // re-borrow it as immutable, so that `self.cost_of_matching_idx` can be called
                        if pu.node.going_to_be_matched && !pu.is_waiting_contract && source < pu_idx {
                            let augmenting_cost = match pu.match_with {
                                Some(_match_with) => {
                                    if brokering {  // only accept offer if this is the last node
                                        cost + pu.node.boundary_cost
                                    } else {
                                        f64::MAX  // never accept offer if matched peer is not in the path
                                    }
                                },
                                None => {
                                    cost - pu.node.boundary_cost
                                },
                            };
                            if augmenting_cost < 0. {  // this is an augmenting path
                                self.has_potential_acceptance = true;
                                let accept_this_offer = self.reproducible_error_generator.next_f64() < pu.accept_probability;
                                // println!("has_potential_acceptance from {}, path: {:?}, with probability {}, take it: {}"
                                //     , pu_idx, path, pu.accept_probability, accept_this_offer);
                                if accept_this_offer {
                                    let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                                    pu.is_waiting_contract = true;
                                    let mut path = path.clone();
                                    path.push(pu_idx);  // push myself at the end, as the final target
                                    pu.out_queue.push(OutMessage {
                                        receiver: broker,  // send back to the last broker
                                        message: Message::AcceptOffer {
                                            timestamp: timestamp,
                                            path: path,
                                            brokering: brokering,
                                            target: pu_idx,
                                        },
                                    });
                                }
                            }
                        }
                    }
                },
                Message::AcceptOffer{ timestamp, ref path, brokering, target } => {
                    assert!(path.len() >= 2, "path should at least contain the source and the next node along the path");
                    let source = *path.first().unwrap();
                    let next = *path.last().unwrap();
                    let should_be_myself = path[path.len() - 2];
                    assert!(should_be_myself == pu_idx, "why should I even receive this message?");
                    let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                    if source == pu_idx {  // I'm the source
                        if pu.is_waiting_contract && pu.is_initiating_augmenting_loop && target == pu_idx {  // always approve augmenting loop
                            pu.active_timestamp += 1;  // to prevent duplicate augmenting path in the same round
                            // do not change others attributes because the message will eventually reach this node again, as Contract
                            pu.out_queue.push(OutMessage {
                                receiver: next,  // send back to the last broker
                                message: Message::Contract {
                                    previous: pu_idx,
                                    target: target,
                                },
                            });
                        } else {  // possible augmenting path
                            let should_approve = !pu.is_waiting_contract && timestamp == pu.active_timestamp && match pu.match_with {
                                Some(match_with) => {
                                    brokering && next == match_with
                                },
                                None => {
                                    !brokering
                                },
                            };
                            if should_approve {
                                pu.active_timestamp += 1;  // to prevent duplicate augmenting path in the same round
                                pu.accept_probability = 1.;
                                match pu.match_with {
                                    Some(_match_with) => {
                                        pu.match_with = None;
                                    },
                                    None => {
                                        pu.match_with = Some(next);
                                    },
                                }
                                pu.out_queue.push(OutMessage {
                                    receiver: next,  // send back to the last broker
                                    message: Message::Contract {
                                        previous: pu_idx,
                                        target: target,
                                    },
                                });
                            } else {
                                pu.out_queue.push(OutMessage {
                                    receiver: next,  // send back to the last broker
                                    message: Message::RefuseAcceptance {
                                        target: target,
                                    },
                                });
                            }
                        }
                    } else {
                        assert!(path.len() >= 3, "path should at least contain the source, myself and next");
                        let previous = path[path.len() - 3];
                        let should_approve = !pu.is_waiting_contract && match pu.match_with {
                            Some(match_with) => {
                                if brokering {
                                    next == match_with
                                } else {
                                    previous == match_with
                                }
                            },
                            None => {
                                false
                            },
                        };
                        if should_approve {
                            pu.is_waiting_contract = true;
                            pu.broker_next_hop = Some(next);
                            let mut new_path = path.clone();
                            new_path.pop();
                            pu.out_queue.push(OutMessage {
                                receiver: previous,  // send back to the last broker
                                message: Message::AcceptOffer {
                                    timestamp: timestamp,
                                    path: new_path,
                                    brokering: !brokering,
                                    target: target,
                                },
                            });
                        } else {
                            pu.out_queue.push(OutMessage {
                                receiver: next,  // send back to the last broker
                                message: Message::RefuseAcceptance {
                                    target: target,
                                },
                            });
                        }
                    }
                },
                Message::Contract{ previous, target } => {
                    let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                    assert!(pu.is_waiting_contract, "why should one receive Contract when it's not waiting for contract?");
                    pu.is_waiting_contract = false;
                    pu.accept_probability = 1.;
                    if target == pu_idx {
                        assert!(pu.broker_next_hop == None, "why should target has next hop?");
                        match pu.match_with {
                            Some(_match_with) => {
                                if pu.is_initiating_augmenting_loop {
                                    pu.is_initiating_augmenting_loop = false;
                                    pu.match_with = Some(previous);
                                } else {
                                    pu.match_with = None;
                                }
                            },
                            None => {
                                pu.match_with = Some(previous);
                            },
                        }
                    } else {
                        assert!(pu.broker_next_hop.is_some(), "must have next hop because this is not the target");
                        let broker_next_hop = pu.broker_next_hop.unwrap();
                        pu.broker_next_hop = None;
                        pu.out_queue.push(OutMessage {
                            receiver: broker_next_hop,  // send to next hop
                            message: Message::Contract {
                                previous: pu_idx,
                                target: target,
                            },
                        });
                        let match_with = pu.match_with.expect("all nodes in the path should be matched pairs");
                        if previous == match_with {
                            pu.match_with = Some(broker_next_hop);
                        } else {
                            assert!(broker_next_hop == match_with, "when receiving contract, node should either match to `previous` or match to `broker_next_hop`");
                            pu.match_with = Some(previous);
                        }
                    }
                },
                Message::RefuseAcceptance{ target } => {
                    let pu = &mut self.processing_units[pu_idx];  // re-borrow it as mutable to change the internal state
                    assert!(pu.is_waiting_contract, "why should one receive RefuseAcceptance when it's not waiting for contract?");
                    pu.is_waiting_contract = false;
                    if target == pu_idx {
                        assert!(pu.broker_next_hop.is_none(), "why should target has next hop?");
                        pu.is_initiating_augmenting_loop = false;
                        if self.disable_probabilistic_accept {
                            pu.accept_probability = 1.;  // keep always accept
                        } else {
                            // suppose 2 path conflicts, maximize the probability of next success
                            // next time success = p(1-p) * 2, maximum is 0.5
                            pu.accept_probability *= 0.5;
                        }
                    } else {
                        assert!(pu.broker_next_hop.is_some(), "must have next hop because this is not the target");
                        let broker_next_hop = pu.broker_next_hop.unwrap();
                        pu.broker_next_hop = None;
                        pu.out_queue.push(OutMessage {
                            receiver: broker_next_hop,  // send to next hop
                            message: Message::RefuseAcceptance {
                                target: target,
                            },
                        });
                    }
                },
                // _ => {
                //     panic!("drop unknown message: {:?}", message);
                // },
            }
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
        if pu.is_waiting_contract {
            return
        }
        pu.active_timestamp += 1;  // invalidate previous offer
        match pu.match_with {
            Some(match_with) => {
                pu.out_queue.push(OutMessage {
                    receiver: match_with,
                    message: Message::MatchOffer {
                        timestamp: pu.active_timestamp,
                        path: vec![pu_idx],
                        brokering: true,
                        cost_to_last_broker: pu.node.boundary_cost,  // if break the match, then the cost of boundary is introduced
                    },
                });
            },
            None => {
                for receiver in pu.direct_neighbors.iter() {
                    pu.out_queue.push(OutMessage {
                        receiver: *receiver,
                        message: Message::MatchOffer {
                            timestamp: pu.active_timestamp,
                            path: vec![pu_idx],
                            brokering: false,
                            cost_to_last_broker: -pu.node.boundary_cost,  // if match, then the cost of boundary is reduced
                        },
                    });
                }
            },
        }
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
            // {  // debug printing
            //     if receiver == 7 || pu_idx == 7 {
            //         println!("{} -> {}: {:?}", pu_idx, receiver, message);
            //     }
            // }
            self.processing_units[receiver].mailbox.push(message);
        }
    }

    /// force breaking the matched node (debugging the algorithm by manually control the initial state)
    #[allow(dead_code)]  // not used in normal cases where error pattern never changes
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
    #[allow(dead_code)]  // not used in normal cases where error pattern never changes
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
                    cost += self.cost_of_matching_idx(pu_idx, match_with);
                }
                Some(match_with)
            } else {  // view as not matched
                cost += pu.node.boundary_cost;
                None
            }
        }).collect();
        (cost, matchings)
    }

    /// get readable matching results
    pub fn matching_result_edges(&self) -> (f64, Vec<((usize, &U), (usize, &U))>) {
        let mut cost = 0.;
        let mut matchings = Vec::new();
        for pu_idx in 0..self.processing_units.len() {
            let pu = &self.processing_units[pu_idx];
            if !pu.node.going_to_be_matched {
                continue
            }
            if pu.is_waiting_contract || pu.match_with == None {  // view as not matched
                cost += pu.node.boundary_cost;
                continue
            }
            let match_with = pu.match_with.unwrap();
            let matched_pu = &self.processing_units[match_with];
            if matched_pu.node.going_to_be_matched && !matched_pu.is_waiting_contract && matched_pu.match_with == Some(pu_idx) {
                if pu_idx < match_with {
                    cost += self.cost_of_matching_idx(pu_idx, match_with);
                    matchings.push(((pu_idx, &pu.node.user_data), (match_with, &matched_pu.node.user_data)))
                }
            } else {  // view as not matched
                cost += pu.node.boundary_cost;
            }
        }
        (cost, matchings)
    }

    /// resend offer once and then run to stable, with maximum cycles as bound
    pub fn pseudo_parallel_resend_offer_run_to_stable(&mut self, max_cycles: usize) -> usize {
        let length = self.processing_units.len();
        let mut cycles = 0;
        // resend offer
        for i in 0..length {
            self.single_pu_resend_offer(i);
            self.single_pu_out_queue_send(i);
        }
        let mut message_processed = 1;
        // loop until no message flying
        while message_processed > 0 && cycles < max_cycles {
            message_processed = 0;
            for i in 0..length {
                message_processed += self.single_pu_execute(i, true);
            }
            for i in 0..length {
                self.single_pu_out_queue_send(i);
            }
            cycles += 1;
            // println!("message_processed: {}", message_processed);
        }
        // println!("resend round end with cycles: {}", cycles);
        cycles
    }

    /// resend offer multiple times and then run to stable, with maximum cycles as bound
    pub fn pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(&mut self, max_resend: usize, max_cycles: usize) -> Result<usize, usize> {
        let mut match_result_changed = true;
        let mut cycles = 0;
        // loop until match pattern doesn't change
        let mut resend_rounds = 0;
        while match_result_changed && resend_rounds < max_resend && cycles < max_cycles {
            let last_match_result = self.matching_result();
            match_result_changed = false;
            self.has_potential_acceptance = false;
            cycles += self.pseudo_parallel_resend_offer_run_to_stable(max_cycles - cycles);
            if self.disable_probabilistic_accept {  // use match pattern changed to judge stop point
                if self.matching_result() != last_match_result {
                    match_result_changed = true;
                }
            } else {  // use `has_potential_acceptance` to judge stop point, because match pattern may not change in a single round
                if self.has_potential_acceptance {
                    match_result_changed = true;
                }
            }
            resend_rounds += 1;
            // println!("matching_result_edges after one round: {:?}", self.matching_result_edges());
            // println!("self: {:?}", self);
        }
        if resend_rounds < max_resend {
            Ok(cycles)
        } else {
            Err(cycles)
        }
    }

    /// run with infinite time and cycles
    pub fn pseudo_parallel_execute_to_stable(&mut self) -> usize {
        self.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(usize::MAX, usize::MAX).unwrap()
    }

}

/// the same as `pseudo_parallel_execute_to_stable_with_max_resend_max_cycles`, return cost of X, cycles of X, cost of Z, cycles of Z
pub fn run_given_offer_decoder_instance(decoder: &mut offer_decoder::OfferDecoder, max_resend: usize, max_cycles: usize) ->
        ((f64, Result<usize, usize>), (f64, Result<usize, usize>)) {
    let d = decoder.d;
    decoder.error_changed();
    // decode X errors
    let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes(d, true);
    for i in (0..=2*d-2).step_by(2) {
        for j in (1..=2*d-3).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].going_to_be_matched = true;
            }
        }
    }
    let mut offer_algorithm_x = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
    let cycles_x = offer_algorithm_x.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(max_resend, max_cycles);
    let (cost_x, edges_x) = offer_algorithm_x.matching_result_edges();
    for ((_, &(i1, j1)), (_, &(i2, j2))) in edges_x.iter() {
        decoder.qubits[i1][j1].match_with = Some((i2, j2));
        decoder.qubits[i2][j2].match_with = Some((i1, j1));
    }
    // decode Z errors
    let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes(d, false);
    for i in (1..=2*d-3).step_by(2) {
        for j in (0..=2*d-2).step_by(2) {
            if decoder.qubits[i][j].measurement {
                nodes[position_to_index[&(i, j)]].going_to_be_matched = true;
            }
        }
    }
    let mut offer_algorithm_z = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
    let cycles_z = offer_algorithm_z.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(max_resend, max_cycles);
    let (cost_z, edges_z) = offer_algorithm_z.matching_result_edges();
    for ((_, &(i1, j1)), (_, &(i2, j2))) in edges_z.iter() {
        decoder.qubits[i1][j1].match_with = Some((i2, j2));
        decoder.qubits[i2][j2].match_with = Some((i1, j1));
    }
    ((cost_x, cycles_x), (cost_z, cycles_z))
}

/// create nodes for standard planar code (2d, perfect measurement condition). return only X stabilizers or only Z stabilizers
pub fn make_standard_planar_code_2d_nodes(d: usize, is_x_stabilizers: bool) -> (Vec<OfferNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<(usize, usize)>) {
    let mut nodes = Vec::new();
    let mut position_to_index = HashMap::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            position_to_index.insert((i, j), nodes.len());
            nodes.push(OfferNode::new((i, j), false, std::cmp::min((j + 1) / 2, d - (j + 1) / 2) as f64));
        }
    }
    let mut direct_neighbors = Vec::new();
    for i in (if is_x_stabilizers { 0..=2*d-2 } else { 1..=2*d-3 }).step_by(2) {
        for j in (if is_x_stabilizers { 1..=2*d-3 } else { 0..=2*d-2 }).step_by(2) {
            for (di, dj) in [(2, 0), (0, 2)].iter() {
                let ni = i + di;
                let nj = j + dj;
                if ni <= 2*d-2 && nj <= 2*d-3 {
                    direct_neighbors.push((position_to_index[&(i, j)], position_to_index[&(ni, nj)]));
                }
            }
        }
    }
    (nodes, position_to_index, direct_neighbors)
}

pub fn simple_cost_standard_planar_code_2d_nodes(a: &(usize, usize), b: &(usize, usize)) -> f64 {
    let (i1, j1) = *a;
    let (i2, j2) = *b;
    let di = (i1 as isize - i2 as isize).abs();
    let dj = (j1 as isize - j2 as isize).abs();
    assert!(di % 2 == 0 && dj % 2 == 0, "cannot compute cost between different types of stabilizers");
    (di + dj) as f64 / 2.
}

#[cfg(test)]
mod tests {
    use super::*;

    // use `cargo test offer_algorithm_test_case_1 -- --nocapture` to run specific test

    fn make_standard_planar_code_2d_nodes_only_x_stabilizers(d: usize) -> (Vec<OfferNode<(usize, usize)>>, HashMap<(usize, usize), usize>, Vec<(usize, usize)>) {
        make_standard_planar_code_2d_nodes(d, true)
    }

    #[test]
    fn offer_algorithm_test_case_1() {
        let d = 3;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        assert_eq!(nodes.len(), 6, "d=3 should have 6 nodes");
        assert_eq!(direct_neighbors.len(), 7, "d=3 should have 7 direct neighbor connections");
        nodes[position_to_index[&(2, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        println!("nodes: {:?}", nodes);  // run `cargo test -- --nocapture` to view these outputs
        println!("direct_neighbors: {:?}", direct_neighbors);
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (1.0, vec![((2, &(2, 1)), (3, &(2, 3)))]));
    }
    
    #[test]
    fn offer_algorithm_test_case_2() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 7)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        offer_algorithm.force_match_nodes(position_to_index[&(4, 3)], position_to_index[&(4, 7)]);
        println!("error matching_result_edges: {:?}", offer_algorithm.matching_result_edges());
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (2.0, vec![((5, &(2, 3)), (9, &(4, 3)))]));
    }

    #[test]
    fn offer_algorithm_test_case_3() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 7)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 3)], position_to_index[&(2, 5)]);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (2.0, vec![((4, &(2, 1)), (5, &(2, 3))), ((6, &(2, 5)), (7, &(2, 7)))]));
    }

    #[test]
    fn offer_algorithm_test_case_4() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(6, 7)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 1)], position_to_index[&(2, 5)]);
        offer_algorithm.force_match_nodes(position_to_index[&(4, 5)], position_to_index[&(6, 7)]);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (3.0, vec![((6, &(2, 5)), (10, &(4, 5)))]));
    }

    #[test]
    fn offer_algorithm_test_case_5() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(6, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(6, 5)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 3)], position_to_index[&(6, 3)]);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 5)], position_to_index[&(6, 5)]);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (2.0, vec![((5, &(2, 3)), (6, &(2, 5))), ((13, &(6, 3)), (14, &(6, 5)))]));
    }

    #[test]
    fn offer_algorithm_test_case_6() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(0, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 7)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (2.0, vec![((1, &(0, 3)), (5, &(2, 3))), ((2, &(0, 5)), (3, &(0, 7)))]));
    }

    #[test]
    fn offer_algorithm_test_case_7() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 7)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 7)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 3)], position_to_index[&(2, 5)]);
        offer_algorithm.force_match_nodes(position_to_index[&(2, 7)], position_to_index[&(4, 7)]);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (3.0, vec![((5, &(2, 3)), (9, &(4, 3))), ((6, &(2, 5)), (7, &(2, 7)))]));
    }

    #[test]
    fn offer_algorithm_test_case_8() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(0, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (3.0, vec![((1, &(0, 3)), (5, &(2, 3))), ((2, &(0, 5)), (6, &(2, 5)))]));
    }

    #[test]
    fn offer_algorithm_test_case_9() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(0, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(0, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 7)]].going_to_be_matched = true;
        nodes[position_to_index[&(6, 5)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (4.0, vec![((0, &(0, 1)), (4, &(2, 1))), ((1, &(0, 3)), (2, &(0, 5))), ((10, &(4, 5)), (14, &(6, 5)))]));
    }

    #[test]
    fn offer_algorithm_test_case_10() {
        let d = 5;
        let (mut nodes, position_to_index, direct_neighbors) = make_standard_planar_code_2d_nodes_only_x_stabilizers(d);
        nodes[position_to_index[&(2, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 3)]].going_to_be_matched = true;
        nodes[position_to_index[&(2, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 1)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 5)]].going_to_be_matched = true;
        nodes[position_to_index[&(4, 7)]].going_to_be_matched = true;
        nodes[position_to_index[&(6, 5)]].going_to_be_matched = true;
        let mut offer_algorithm = OfferAlgorithm::new(nodes, direct_neighbors, 2 * 2 * d, simple_cost_standard_planar_code_2d_nodes, 0);
        let cycles = offer_algorithm.pseudo_parallel_execute_to_stable();
        println!("cycles: {:?}", cycles);
        let matching_result_edges = offer_algorithm.matching_result_edges();
        println!("matching_result_edges: {:?}", matching_result_edges);
        assert_eq!(matching_result_edges, (4.0, vec![((4, &(2, 1)), (8, &(4, 1))), ((5, &(2, 3)), (6, &(2, 5))), ((10, &(4, 5)), (14, &(6, 5)))]));
    }

}
