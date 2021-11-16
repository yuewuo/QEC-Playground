use super::types::QubitType;
use super::types::ErrorType;
use std::collections::HashMap;
use super::rand::prelude::*;
use super::rand_core::SeedableRng;
use super::reproducible_rand::{Xoroshiro128StarStar};

/// only for standard planar code
#[derive(Debug, Clone)]
pub struct OfferDecoder {
    pub d: usize,
    pub qubits: Vec< Vec<Qubit> >,
    pub default_max_hop: usize,
    // statistics
    pub message_count_single_round: usize,
    pub message_count: usize,
    pub has_potential_acceptance: bool,
    // random generator
    pub disable_probabilistic_accept: bool,
    pub rng: ThreadRng,
    pub use_reproducible_error_generator: bool,
    pub reproducible_error_generator: Xoroshiro128StarStar,
}

#[derive(Debug, Clone)]
pub struct Qubit {
    pub i: usize,
    pub j: usize,
    pub qubit_type: QubitType,
    pub error: ErrorType,
    pub measurement: bool,
    // for stabilizers only
    pub mailbox: Vec<Message>,
    pub out_queue: Vec<OutMessage>,
    pub active_timestamp: usize,
    pub offer_cache: HashMap::<(usize, usize), CachedOffer>,
    pub loop_cache: HashMap::<(usize, usize), CachedOffer>,
    pub state: NodeState,
    pub boundary_cost: f64,
    pub cost: f64,
    pub broker_next_hop: Option<(usize, usize)>,
    pub match_with: Option<(usize, usize)>,
    pub accept_probability: f64,
    // helper variables
    pub neighbors: Vec::<(usize, usize, f64)>,  // i, j, cost
    pub xor_data_qubits: Vec::<(usize, usize)>,  // i, j
}

#[derive(Debug, Clone)]
pub enum Message {
    MatchOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
        max_hop: usize,
    },
    // AugmentOffer,  // TODO: implement later
    AcceptOffer {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
        is_loop_offer: bool,
    },
    RefuseAcceptance{
        target: (usize, usize),
        source: (usize, usize),
    },
    Contract {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
    },
    BrokeredOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
        max_hop: usize,
    },
    AcceptBrokeredOffer {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
        is_loop_offer: bool,
    },
    BrokeredContract {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
    },
    BrokeredBreakOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
        max_hop: usize,
    },
    BreakOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
        max_hop: usize,
    },
    LoopOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
    },
    BrokeredLoopOffer {
        timestamp: usize,
        source: (usize, usize),
        broker: (usize, usize),
        cost: f64,
    },
}

#[derive(Debug, Clone)]
pub struct CachedOffer {
    pub timestamp: usize,
    pub cost: f64,
    pub broker: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct OutMessage {
    pub receiver: (usize, usize),
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeState {
    NoError,
    SentOffer,
    WaitingContract,
    Matched,
}

impl OfferDecoder {
    // return the message processed in this round
    pub fn qubit_node_execute(&mut self, i: usize, j: usize, process_one_message: bool) -> usize {
        let qubit = &mut self.qubits[i][j];
        // do not process data qubit
        if qubit.qubit_type == QubitType::Data { return 0 }
        // if some measurement error disappears, break the matched pairs (not happening in real)
        if qubit.measurement == false && qubit.state != NodeState::NoError {
            if qubit.state == NodeState::Matched {
                self.force_break_matched(i, j);
            } else {
                panic!("qubit ({},{}) measurement changed from `true` to `false`, which is not supported yet. this may cause deadlock.", i, j);
            }
        }
        // read message
        let mut message_processed = 0;
        let use_reproducible_error_generator = &self.use_reproducible_error_generator;
        let reproducible_error_generator = &mut self.reproducible_error_generator;
        let rng = &mut self.rng;
        let mut random_generate = || {
            if *use_reproducible_error_generator {
                reproducible_error_generator.next_f64()
            } else {
                rng.gen::<f64>()
            }
        };
        while self.qubits[i][j].mailbox.len() > 0 {
            let qubit = &mut self.qubits[i][j];  // have to re-borrow it as mutable
            let message = qubit.mailbox.remove(0);  // take the first message in mailbox
            message_processed += 1;
            // println!("qubit[{}][{}]: {:?}", i, j, message);
            match message {
                Message::MatchOffer{ timestamp, source: (si, sj), broker: (bi, bj), cost, max_hop } => {
                    let cached_offer = qubit.offer_cache.get(&(si, sj));
                    let not_caching_this_offer = match cached_offer {
                        Some(cached_offer) => {
                            cached_offer.timestamp < timestamp || cost < cached_offer.cost
                        },
                        None => true,
                    };
                    // cache this offer if not cached
                    if not_caching_this_offer {
                        qubit.offer_cache.insert((si, sj), CachedOffer {
                            timestamp: timestamp,
                            cost: cost,
                            broker: (bi, bj),
                        });
                    }
                    // may broker this offer
                    if qubit.state == NodeState::Matched {
                        // when cost < cached_offer.cost, the farther node will not broker this offer backward
                        // and also, this makes an infinite ping-pong between the matched pairs impossible, which is harmful to the system
                        if not_caching_this_offer {
                            qubit.out_queue.push(OutMessage {
                                receiver: qubit.match_with.expect("exist"),
                                message: Message::BrokeredOffer {
                                    timestamp: timestamp,
                                    source: (si, sj),
                                    broker: (i, j),
                                    cost: cost - qubit.cost,  // minus the cost of matching pair
                                    max_hop: max_hop - 1,  // when brokering the max_hop - 1
                                },
                            });
                        }
                    }
                    // propagate this offer if cache is updated
                    if not_caching_this_offer {
                        for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                            qubit.out_queue.push(OutMessage {
                                receiver: (*ni, *nj),
                                message: Message::MatchOffer {
                                    timestamp: timestamp,
                                    source: (si, sj),
                                    broker: (bi, bj),
                                    cost: cost + *neighbor_cost,
                                    max_hop: max_hop,
                                },
                            });
                        }
                    }
                    // take this offer if it has error
                    if qubit.measurement == true {
                        if qubit.state == NodeState::NoError || qubit.state == NodeState::SentOffer {
                            // take this offer only if target is smaller than source and cost is better than current
                            // the overall cost < 0 is an augmenting path
                            if Self::compare_i_j(i, j, si, sj) < 0 && cost - qubit.cost < 0. {
                                self.has_potential_acceptance = true;  // mark potential take
                                let accept_this_offer = random_generate() < qubit.accept_probability;
                                // println!("has_potential_acceptance from [{}][{}], with probability {}, take it: {}", i, j, qubit.accept_probability, accept_this_offer);
                                if accept_this_offer {
                                    qubit.state = NodeState::WaitingContract;
                                    qubit.out_queue.push(OutMessage {
                                        receiver: (bi, bj),  // send back to the last broker
                                        message: Message::AcceptOffer {
                                            target: (i, j),  // take this offer as target
                                            source: (si, sj),
                                            broker: (i, j),  // target is also the broker of this message
                                            is_loop_offer: false,
                                        },
                                    });
                                }
                            }
                        }
                    }
                },
                Message::AcceptOffer{ target: (ti, tj), source: (si, sj), broker: (bi, bj), is_loop_offer } => {
                    if si == i && sj == j {
                        if qubit.state == NodeState::SentOffer {
                            qubit.state = NodeState::Matched;
                            qubit.match_with = Some((bi, bj));  // always match with the first-hop broker
                            qubit.cost = Self::cost_of_matching(i, j, bi, bj);
                            qubit.accept_probability = 1.;
                            qubit.out_queue.push(OutMessage {
                                receiver: (bi, bj),  // send back to the last broker
                                message: Message::Contract {
                                    target: (ti, tj),
                                    source: (i, j),
                                    broker: (i, j),
                                },
                            });
                        } else {  // refuse acceptance
                            qubit.out_queue.push(OutMessage {
                                receiver: (bi, bj),  // send back to the last broker
                                message: Message::RefuseAcceptance {
                                    target: (ti, tj),
                                    source: (i, j),
                                },
                            });
                        }
                    } else {  // this is broker
                        if qubit.state == NodeState::Matched {
                            qubit.state = NodeState::WaitingContract;
                            qubit.broker_next_hop = Some((bi, bj));
                            qubit.out_queue.push(OutMessage {
                                receiver: qubit.match_with.expect("exist"),
                                message: Message::AcceptBrokeredOffer {
                                    target: (ti, tj),
                                    source: (si, sj),
                                    broker: (i, j),
                                    is_loop_offer: is_loop_offer,
                                },
                            });
                        } else {
                            qubit.out_queue.push(OutMessage {
                                receiver: (bi, bj),  // send back to the last broker
                                message: Message::RefuseAcceptance {
                                    target: (ti, tj),
                                    source: (i, j),  // mark who refuse the acceptance
                                },
                            });
                        }
                    }
                },
                Message::Contract{ target: (ti, tj), source: (si, sj), broker: (bi, bj) } => {
                    if qubit.state == NodeState::WaitingContract {
                        qubit.state = NodeState::Matched;
                        if ti != i || tj != j {  // this is broker
                            qubit.out_queue.push(OutMessage {
                                receiver: qubit.match_with.expect("exist"),
                                message: Message::BrokeredContract {
                                    target: (ti, tj),
                                    source: (si, sj),
                                    broker: (i, j),  // I'm the broker
                                },
                            });
                            qubit.accept_probability = 1.;
                        } else {  // this is the target
                            qubit.accept_probability = 1.;
                        }
                        qubit.match_with = Some((bi, bj));
                        qubit.cost = Self::cost_of_matching(i, j, bi, bj);
                    } else {
                        println!("This shoudn't happen! Contract is never sent to a node in state other than WaitingContract");
                        println!("message: {:?}", message);
                    }
                },
                Message::RefuseAcceptance{ target: (ti, tj), source: (si, sj) } => {
                    if ti == i && tj == j {
                        // reduce the probability to take this offer
                        qubit.accept_probability = Self::accept_probability_next(self.disable_probabilistic_accept, qubit.accept_probability);
                        qubit.state = match qubit.match_with {
                            None => NodeState::SentOffer,
                            Some(_) => NodeState::Matched,
                        };
                    } else {
                        qubit.state = NodeState::Matched;
                        match qubit.broker_next_hop {
                            None => {  // send refuse to peer
                                qubit.out_queue.push(OutMessage {
                                    receiver: qubit.match_with.expect("exist"),
                                    message: Message::RefuseAcceptance {
                                        target: (ti, tj),
                                        source: (si, sj),
                                    },
                                });
                            },
                            Some((ni, nj)) => {  // send to next hop
                                qubit.out_queue.push(OutMessage {
                                    receiver: (ni, nj),  // send back to the last broker
                                    message: Message::RefuseAcceptance {
                                        target: (ti, tj),
                                        source: (si, sj),
                                    },
                                });
                                qubit.broker_next_hop = None;
                            },
                        }
                    }
                },
                Message::BrokeredOffer{ timestamp, broker: (bi, bj), source: (si, sj), cost, max_hop } => {
                    if qubit.state == NodeState::Matched && qubit.match_with.unwrap() == (bi, bj) {
                        if cost + qubit.boundary_cost < 0. {  // break this matched pair is an augmenting path
                            // TODO: this may also find augmenting loop!!! may cause deadlock, handle this later
                            // take this offer
                            self.has_potential_acceptance = true;  // mark potential take
                            let accept_this_offer = random_generate() < qubit.accept_probability;
                            // println!("has_potential_acceptance from [{}][{}], with probability {}, take it: {}", i, j, qubit.accept_probability, accept_this_offer);
                            if accept_this_offer {
                                qubit.state = NodeState::WaitingContract;
                                qubit.out_queue.push(OutMessage {
                                    receiver: qubit.match_with.expect("exist"),
                                    message: Message::AcceptBrokeredOffer {
                                        target: (i, j),
                                        source: (si, sj),
                                        broker: (i, j),
                                        is_loop_offer: false,
                                    },
                                });
                            }
                        } else {  // propagate this offer to neighbors
                            for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                                qubit.out_queue.push(OutMessage {
                                    receiver: (*ni, *nj),
                                    message: Message::MatchOffer {
                                        timestamp: timestamp,
                                        source: (si, sj),
                                        broker: (i, j),  // I'm the broker (sink) of this offer
                                        cost: cost + *neighbor_cost,
                                        max_hop: max_hop,
                                    },
                                });
                            }
                        }
                    } else {
                        // simply ignore this
                        // assert_eq!(qubit.state, NodeState::Matched, "why should an unmatched qubit receive a BrokeredOffer message?");
                    }
                },
                Message::AcceptBrokeredOffer{ source: (si, sj), target: (ti, tj), broker: (bi, bj), is_loop_offer } => {
                    if qubit.state == NodeState::Matched && qubit.match_with.unwrap() == (bi, bj) {
                        if i == si && j == sj {  // this should be an acceptance of break offer
                            qubit.out_queue.push(OutMessage {
                                receiver: qubit.match_with.expect("exist"),
                                message: Message::BrokeredContract {
                                    target: (ti, tj),
                                    source: (i, j),
                                    broker: (i, j),
                                }
                            });
                            qubit.state = NodeState::SentOffer;  // connect to boundary
                            qubit.match_with = None;  // connect to boundary
                            qubit.cost = qubit.boundary_cost;
                        } else {
                            let cached_offer = if is_loop_offer {
                                qubit.loop_cache.get(&(si, sj))
                            } else {
                                qubit.offer_cache.get(&(si, sj))
                            };
                            match cached_offer {
                                Some(cached_offer) => {
                                    qubit.state = NodeState::WaitingContract;
                                    qubit.out_queue.push(OutMessage {
                                        receiver: cached_offer.broker,  // send back to the last broker
                                        message: Message::AcceptOffer {
                                            target: (ti, tj),
                                            source: (si, sj),
                                            broker: (i, j),  // I'm the broker of this offer
                                            is_loop_offer: is_loop_offer,
                                        },
                                    });
                                },
                                None => {
                                    qubit.out_queue.push(OutMessage {
                                        receiver: qubit.match_with.expect("exist"),  // send back to the last broker
                                        message: Message::RefuseAcceptance {
                                            target: (ti, tj),
                                            source: (si, sj),
                                        },
                                    });
                                },
                            }
                        }
                    } else {
                        qubit.out_queue.push(OutMessage {
                            receiver: qubit.match_with.expect("exist"),  // send back to the last broker
                            message: Message::RefuseAcceptance {
                                target: (ti, tj),
                                source: (si, sj),
                            },
                        });
                    }
                },
                Message::BrokeredContract{ target: (ti, tj), source: (si, sj), broker: (bi, bj) } => {
                    let (mi, mj) = qubit.match_with.expect("exist");
                    assert!(bi == mi && bj == mj, "matching information inconsistent. may caused by message disorder.");
                    match qubit.broker_next_hop {
                        Some((ni, nj)) => {
                            qubit.out_queue.push(OutMessage {
                                receiver: (ni, nj),  // send contract to the next hop
                                message: Message::Contract {
                                    target: (ti, tj),
                                    source: (si, sj),
                                    broker: (i, j),
                                },
                            });
                            qubit.state = NodeState::Matched;
                            qubit.match_with = Some((ni, nj));
                            qubit.cost = Self::cost_of_matching(i, j, ni, nj);
                            qubit.broker_next_hop = None;
                        },
                        None => {  // if no broker_next_hop, then it is the last node which should connect to boundary
                            qubit.accept_probability = 1.;
                            qubit.state = NodeState::SentOffer;  // unlock and connect to boundary
                            qubit.cost = qubit.boundary_cost;
                            qubit.match_with = None;
                        },
                    }
                },
                Message::BrokeredBreakOffer{ timestamp, broker: (bi, bj), source: (si, sj), cost, max_hop } => {
                    if qubit.state == NodeState::Matched && qubit.match_with.expect("exist") == (bi, bj) {
                        // whether take this offer
                        if Self::compare_i_j(i, j, si, sj) < 0 && cost + qubit.boundary_cost < 0. {  // the overall cost < 0 is an augmenting path
                            self.has_potential_acceptance = true;  // mark potential take
                            let accept_this_offer = random_generate() < qubit.accept_probability;
                            // println!("has_potential_acceptance from [{}][{}], with probability {}, take it: {}", i, j, qubit.accept_probability, accept_this_offer);
                            if accept_this_offer {
                                qubit.state = NodeState::WaitingContract;
                                qubit.out_queue.push(OutMessage {
                                    receiver: (bi, bj),  // send back to the last broker
                                    message: Message::AcceptBrokeredOffer {
                                        target: (i, j),  // take this offer as target
                                        source: (si, sj),
                                        broker: (i, j),  // target is also the broker of this message
                                        is_loop_offer: false,
                                    },
                                });
                                qubit.broker_next_hop = None;
                            }
                        } else {
                            // broadcast this offer to others
                            for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                                qubit.out_queue.push(OutMessage {
                                    receiver: (*ni, *nj),
                                    message: Message::BreakOffer {
                                        timestamp: timestamp,
                                        source: (si, sj),
                                        broker: (i, j),  // I'm the broker of this offer
                                        cost: cost + *neighbor_cost,
                                        max_hop: max_hop,
                                    },
                                });
                            }
                        }
                    }  // else just ignore this
                },
                Message::BreakOffer{ timestamp, broker: (bi, bj), source: (si, sj), cost, max_hop } => {
                    let cached_offer = qubit.offer_cache.get(&(si, sj));
                    let not_caching_this_offer = match cached_offer {
                        Some(cached_offer) => {
                            cached_offer.timestamp < timestamp || cost < cached_offer.cost
                        },
                        None => true,
                    };
                    // cache this offer if not cached
                    if not_caching_this_offer {
                        qubit.offer_cache.insert((si, sj), CachedOffer {
                            timestamp: timestamp,
                            cost: cost,
                            broker: (bi, bj),
                        });
                    }
                    // propagate this offer if cache is updated
                    if not_caching_this_offer {
                        for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                            qubit.out_queue.push(OutMessage {
                                receiver: (*ni, *nj),
                                message: Message::BreakOffer {
                                    timestamp: timestamp,
                                    source: (si, sj),
                                    broker: (bi, bj),
                                    cost: cost + *neighbor_cost,
                                    max_hop: max_hop,
                                },
                            });
                        }
                    }
                    if qubit.state == NodeState::Matched {
                        let (mi, mj) = qubit.match_with.expect("exist");
                        if not_caching_this_offer && !(si == mi && sj == mj) && !(si == i && sj == j) {
                            qubit.out_queue.push(OutMessage {
                                receiver: qubit.match_with.expect("exist"),
                                message: Message::BrokeredBreakOffer {
                                    timestamp: timestamp,
                                    source: (si, sj),
                                    broker: (i, j),
                                    cost: cost - qubit.cost,  // minus the cost of matching pair
                                    max_hop: max_hop - 1,
                                },
                            });
                        }
                    }
                },
                Message::LoopOffer{ timestamp, broker: (bi, bj), source: (si, sj), cost } => {
                    // cache and propagate this offer only if the cost is better than cached or timestamp is newer
                    let cached_offer = qubit.loop_cache.get(&(si, sj));
                    let not_caching_this_offer = match cached_offer {
                        Some(cached_offer) => {
                            cached_offer.timestamp < timestamp || cost < cached_offer.cost
                        },
                        None => true,
                    };
                    // cache this offer if not cached
                    if not_caching_this_offer {
                        qubit.loop_cache.insert((si, sj), CachedOffer {
                            timestamp: timestamp,
                            cost: cost,
                            broker: (bi, bj),
                        });
                    }
                    // propagate this offer if cache is updated
                    if not_caching_this_offer && (si != i || sj != j) {
                        for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                            qubit.out_queue.push(OutMessage {
                                receiver: (*ni, *nj),
                                message: Message::LoopOffer {
                                    timestamp: timestamp,
                                    source: (si, sj),
                                    broker: (bi, bj),
                                    cost: cost + *neighbor_cost,
                                },
                            });
                        }
                    }
                    if not_caching_this_offer && qubit.state == NodeState::Matched {
                        let (mi, mj) = qubit.match_with.expect("exist");
                        if mi == si && mj == sj {
                            if cost < 0. {  // the overall cost < 0 is an augmenting loop
                                self.has_potential_acceptance = true;  // mark potential take
                                let accept_this_offer = random_generate() < qubit.accept_probability;
                                // println!("has_potential_acceptance from [{}][{}], with probability {}, take it: {}", i, j, qubit.accept_probability, accept_this_offer);
                                if accept_this_offer {
                                    // always accept loop offer
                                    qubit.state = NodeState::WaitingContract;
                                    qubit.out_queue.push(OutMessage {
                                        receiver: (bi, bj),  // send back to the last broker
                                        message: Message::AcceptOffer {
                                            target: (i, j),  // take this offer as target
                                            source: (si, sj),
                                            broker: (i, j),  // target is also the broker of this message
                                            is_loop_offer: true,
                                        },
                                    });
                                }
                            }
                        } else if Self::compare_i_j(si, sj, i, j) < 0 && Self::compare_i_j(si, sj, mi, mj) < 0 {
                            // broker it only if source is smaller than any peer
                            if not_caching_this_offer {
                                qubit.out_queue.push(OutMessage {
                                    receiver: qubit.match_with.expect("exist"),
                                    message: Message::BrokeredLoopOffer {
                                        timestamp: timestamp,
                                        source: (si, sj),
                                        broker: (i, j),
                                        cost: cost - qubit.cost,  // minus the cost of matching pair
                                    },
                                });
                            }
                        }
                    }
                },
                Message::BrokeredLoopOffer{ timestamp, broker: (bi, bj), source: (si, sj), cost } => {
                    // broadcast this offer to others
                    if qubit.state == NodeState::Matched {
                        let (mi, mj) = qubit.match_with.expect("exist");
                        if mi == bi && mj == bj {
                            for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                                qubit.out_queue.push(OutMessage {
                                    receiver: (*ni, *nj),
                                    message: Message::LoopOffer {
                                        timestamp: timestamp,
                                        source: (si, sj),
                                        broker: (i, j),  // I'm the broker of this offer
                                        cost: cost + *neighbor_cost,
                                    },
                                });
                            }
                        }
                    }
                }
                // _ => {
                //     panic!("drop unknown message: {:?}", message);
                // },
            }
            // process only one message
            if process_one_message {
                break
            }
        }
        message_processed
    }
    pub fn qubit_resend_offer(&mut self, i: usize, j: usize) {
        let qubit = &mut self.qubits[i][j];
        // normal node never sends offer
        if qubit.measurement == false { return }
        // if the state of this qubit is matched, then the offer targets only self
        match qubit.state {
            NodeState::NoError | NodeState::SentOffer => {
                qubit.active_timestamp += 1;  // any timestamp smaller than this is an outdated offer and will be updated in cache (but will not be rejected)
                for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                    qubit.out_queue.push(OutMessage {
                        receiver: (*ni, *nj),
                        message: Message::MatchOffer {
                            timestamp: qubit.active_timestamp,
                            source: (i, j),
                            broker: (i, j),  // if broker == source then there is no broker
                            cost: *neighbor_cost - qubit.boundary_cost,  // if match, then the cost of boundary is reduced
                            max_hop: self.default_max_hop,
                        },
                    });
                }
                qubit.state = NodeState::SentOffer;  // offer sent and waiting for replies
            },
            NodeState::Matched => {
                let (mi, mj) = qubit.match_with.expect("exist");
                qubit.active_timestamp += 1;
                // send break offer which is never accepted by unmatched node
                qubit.out_queue.push(OutMessage {
                    receiver: (mi, mj),
                    message: Message::BrokeredBreakOffer {
                        timestamp: qubit.active_timestamp,
                        source: (i, j),
                        broker: (i, j),  // I'm the broker
                        cost: qubit.boundary_cost - qubit.cost,  // if break, then the cost of boundary is introduced but the cost of matching is reduced
                        max_hop: self.default_max_hop,
                    },
                });
                // send augmenting loop offer, which is only broadcasted if the broker is bigger than source (to avoid conflicts)
                if Self::compare_i_j(i, j, mi, mj) < 0 {  // smaller one is responsible for initiating the loop finding protocol
                    for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                        qubit.out_queue.push(OutMessage {
                            receiver: (*ni, *nj),
                            message: Message::LoopOffer {
                                timestamp: qubit.active_timestamp,
                                source: (i, j),
                                broker: (i, j),  // if broker == source then there is no broker
                                cost: *neighbor_cost - qubit.cost,  // if match, then the cost of matching is reduced
                            },
                        });
                    }
                }
            },
            _ => { },  // do nothing if in other states
        }
    }
    pub fn qubit_out_queue_send(&mut self, i: usize, j: usize) {
        // send messages from out_queue
        let mut mut_messages = self.qubits[i][j].out_queue.split_off(0);
        for out_message in mut_messages.drain(..) {
            self.message_count_single_round += 1;
            self.message_count += 1;
            let (ri, rj) = out_message.receiver;
            assert!(self.is_valid_i_j(ri, rj), "receiver must have valid address");
            let message = out_message.message;
            match message {
                Message::MatchOffer { timestamp: _, source: _, broker: _, cost: _, max_hop }
                | Message::BrokeredOffer { timestamp: _, source: _, broker: _, cost: _, max_hop }
                | Message::BreakOffer { timestamp: _, source: _, broker: _, cost: _, max_hop }
                | Message::BrokeredBreakOffer { timestamp: _, source: _, broker: _, cost: _, max_hop } => {
                    if max_hop == 0 {
                        // println!("max_hop reach 0");
                        continue  // do not send it if max_hop == 0
                    }
                }
                _ => { }
            }
            self.qubits[ri][rj].mailbox.push(message);
        }
    }
    pub fn force_match_qubits(&mut self, i1: usize, j1: usize, i2: usize, j2: usize) {
        if i1 == i2 && j1 == j2 { return }  // why match the same qubit?
        // break them first
        self.force_break_matched(i1, j1);
        self.force_break_matched(i2, j2);
        // connect them
        let qubit1 = &mut self.qubits[i1][j1];
        qubit1.state = NodeState::Matched;
        qubit1.match_with = Some((i2, j2));
        qubit1.cost = Self::cost_of_matching(i1, j1, i2, j2);
        let qubit2 = &mut self.qubits[i2][j2];
        qubit2.state = NodeState::Matched;
        qubit2.match_with = Some((i1, j1));
        qubit2.cost = Self::cost_of_matching(i1, j1, i2, j2);
    }
    pub fn force_break_matched(&mut self, i: usize, j: usize) {
        let qubit = &mut self.qubits[i][j];
        if qubit.state != NodeState::Matched { return }   // no need to break
        qubit.state = NodeState::NoError;
        qubit.offer_cache = HashMap::new();
        let (mi, mj) = qubit.match_with.expect("matched qubit must have `match_with`");
        qubit.match_with = None;
        qubit.cost = qubit.boundary_cost;
        let matched_qubit = &mut self.qubits[mi][mj];
        matched_qubit.state = NodeState::NoError;
        matched_qubit.offer_cache = HashMap::new();
        matched_qubit.match_with = None;
        matched_qubit.cost = matched_qubit.boundary_cost;
    }
    pub fn is_valid_i_j(&self, i: usize, j: usize) -> bool {
        if i >= self.d * 2 - 1 { return false }
        if j >= self.d * 2 - 1 { return false }
        return true
    }
    pub fn compare_i_j(i1: usize, j1: usize, i2: usize, j2: usize) -> isize {
        if i1 == i2 {
            if j1 == j2 { return 0 }
            if j1 < j2 { return -1 }
            else { return 1 }
        }
        if i1 < i2 { return -1 }
        return 1
    }
    pub fn cost_of_matching(i1: usize, j1: usize, i2: usize, j2: usize) -> f64 {
        let di = (i1 as isize - i2 as isize).abs();
        let dj = (j1 as isize - j2 as isize).abs();
        assert!(di % 2 == 0 && dj % 2 == 0, "cannot compute cost between different types of stabilizers");
        (di + dj) as f64 / 2.
    }
    pub fn reinitialize(&mut self) {
        let length = 2 * self.d - 1;
        for i in 0..length {
            for j in 0..length {
                let qubit = &mut self.qubits[i][j];
                qubit.error = ErrorType::I;
                qubit.mailbox.clear();
                qubit.out_queue.clear();
                qubit.active_timestamp = 0;
                qubit.offer_cache.clear();
                qubit.loop_cache.clear();
                qubit.state = NodeState::NoError;
                qubit.cost = 0.;
                qubit.broker_next_hop = None;
                qubit.match_with = None;
            }
        }
        self.message_count_single_round = 0;
        self.message_count = 0;
        self.has_potential_acceptance = false;
    }
    pub fn error_changed(&mut self) {
        let length = 2 * self.d - 1;
        for i in 0..length {
            for j in 0..length {
                let qubit = &self.qubits[i][j];
                if qubit.qubit_type == QubitType::StabZ || qubit.qubit_type == QubitType::StabX {
                    let mut error_count = 0;
                    for (xi, xj) in qubit.xor_data_qubits.iter() {
                        let target_qubit = &self.qubits[*xi][*xj];
                        match qubit.qubit_type {
                            QubitType::StabZ => {  // Z stabilizer detects X errors
                                match target_qubit.error {
                                    ErrorType::X | ErrorType::Y => { error_count += 1; }
                                    _ => { },
                                }
                            },
                            QubitType::StabX => {  // X stabilizer detects Z errors
                                match target_qubit.error {
                                    ErrorType::Z | ErrorType::Y => { error_count += 1; }
                                    _ => { },
                                }
                            },
                            _ => { },
                        }
                    }
                    let qubit = &mut self.qubits[i][j];
                    qubit.measurement = error_count % 2 == 1;
                    // update cost
                    if qubit.measurement {
                        match qubit.match_with {
                            None => { qubit.cost = qubit.boundary_cost; },
                            Some((mi, mj)) => { qubit.cost = Self::cost_of_matching(i, j, mi, mj); },
                        }
                    } else {
                        qubit.cost = 0.  // no error syndrome here, so the cost is 0
                    }
                }
            }
        }
    }
    pub fn match_pattern(&self) -> Vec::< Vec::< Option<(usize, usize)> > > {
        (0 .. 2 * self.d - 1).map(|i| {
            (0 .. 2 * self.d - 1).map(|j| {
                self.qubits[i][j].match_with
            }).collect()
        }).collect()
    }
    pub fn match_pattern_changed(&self, last: &Vec::< Vec::< Option<(usize, usize)> > >) -> bool {
        let length = 2 * self.d - 1;
        for i in 0..length {
            for j in 0..length {
                if last[i][j] != self.qubits[i][j].match_with {
                    return true
                }
            }
        }
        false
    }
    pub fn pseudo_parallel_resend_offer_run_to_stable(&mut self, max_cycles: usize) -> usize {
        let length = 2 * self.d - 1;
        let mut cycles = 0;
        // resend offer
        for i in 0..length {
            for j in 0..length {
                self.qubit_resend_offer(i, j);
                self.qubit_out_queue_send(i, j);
            }
        }
        let mut message_processed = 1;
        // loop until no message flying
        while message_processed > 0 && cycles < max_cycles {
            message_processed = 0;
            for i in 0..length {
                for j in 0..length {
                    message_processed += self.qubit_node_execute(i, j, true);
                }
            }
            for i in 0..length {
                for j in 0..length {
                    self.qubit_out_queue_send(i, j);
                }
            }
            cycles += 1;
            // println!("message_processed: {}", message_processed);
        }
        // println!("cycles: {}", cycles);
        cycles
    }
    pub fn pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(&mut self, max_resend: usize, max_cycles: usize) -> Result<usize, usize> {
        let mut match_pattern_changed = true;
        let mut cycles = 0;
        // loop until match pattern doesn't change
        let mut resend_rounds = 0;
        while match_pattern_changed && resend_rounds < max_resend && cycles < max_cycles {
            let last_match_pattern = self.match_pattern();
            match_pattern_changed = false;
            self.has_potential_acceptance = false;
            cycles += self.pseudo_parallel_resend_offer_run_to_stable(max_cycles - cycles);
            if self.disable_probabilistic_accept {  // use match pattern changed to judge stop point
                if self.match_pattern_changed(&last_match_pattern) {
                    match_pattern_changed = true;
                }
            } else {  // use `has_potential_acceptance` to judge stop point, because match pattern may not change in a single round
                if self.has_potential_acceptance {
                    match_pattern_changed = true;
                }
            }
            resend_rounds += 1;
        }
        if resend_rounds < max_resend {
            Ok(cycles)
        } else {
            Err(cycles)
        }
    }
    // return the cycles
    pub fn pseudo_parallel_execute_to_stable(&mut self) -> usize {
        self.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(usize::MAX, usize::MAX).unwrap()
    }
    pub fn origin_error_left_boundary_cardinality(&self) -> usize {
        let length = 2 * self.d - 1;
        let mut boundary_error_cnt = 0;
        for i in (0..length).step_by(2) {
            match self.qubits[i][0].error {
                ErrorType::X | ErrorType::Y => {
                    boundary_error_cnt += 1;
                },
                _ => { },
            }
        }
        boundary_error_cnt
    }
    pub fn origin_error_top_boundary_cardinality(&self) -> usize {
        let length = 2 * self.d - 1;
        let mut boundary_error_cnt = 0;
        for j in (0..length).step_by(2) {
            match self.qubits[0][j].error {
                ErrorType::X | ErrorType::Y => {
                    boundary_error_cnt += 1;
                },
                _ => { },
            }
        }
        boundary_error_cnt
    }
    pub fn has_logical_error(&self, error_type: ErrorType) -> bool {
        let length = 2 * self.d - 1;
        let half_length = self.d - 1;
        if error_type == ErrorType::X || error_type == ErrorType::Y {  // j = 0
            let mut boundary_error_cnt = self.origin_error_left_boundary_cardinality();
            for i in 0..length {
                for j in 0..half_length {
                    let qubit = &self.qubits[i][j];
                    if qubit.qubit_type == QubitType::StabZ && qubit.measurement == true && qubit.match_with.is_none() {
                        boundary_error_cnt += 1;
                    }
                }
            }
            if boundary_error_cnt % 2 == 1 { return true }
        }
        if error_type == ErrorType::Z || error_type == ErrorType::Y {  // i = 0
            let mut boundary_error_cnt = self.origin_error_top_boundary_cardinality();
            for i in 0..half_length {
                for j in 0..length {
                    let qubit = &self.qubits[i][j];
                    if qubit.qubit_type == QubitType::StabX && qubit.measurement == true && qubit.match_with.is_none() {
                        boundary_error_cnt += 1;
                    }
                }
            }
            if boundary_error_cnt % 2 == 1 { return true }
        }
        false
    }
    pub fn generate_depolarizing_random_errors<F>(&mut self, error_rate: f64, mut rng: F) -> usize where F: FnMut() -> f64 {
        let length = 2 * self.d - 1;
        let mut error_count = 0;
        for i in 0..length {
            for j in 0..length {
                if self.qubits[i][j].qubit_type != QubitType::Data { continue }
                let random_number = rng();
                if random_number < error_rate {
                    self.qubits[i][j].error = ErrorType::X;
                    error_count += 1;
                } else if random_number < 2. * error_rate {
                    self.qubits[i][j].error = ErrorType::Z;
                    error_count += 1;
                } else if random_number < 3. * error_rate {
                    self.qubits[i][j].error = ErrorType::Y;
                    error_count += 1;
                } else {
                    self.qubits[i][j].error = ErrorType::I;
                }
            }
        }
        error_count
    }
    pub fn generate_only_x_random_errors<F>(&mut self, error_rate: f64, mut rng: F) -> usize where F: FnMut() -> f64 {
        let length = 2 * self.d - 1;
        let mut error_count = 0;
        for i in 0..length {
            for j in 0..length {
                if self.qubits[i][j].qubit_type != QubitType::Data { continue }
                let random_number = rng();
                if random_number < error_rate {
                    self.qubits[i][j].error = ErrorType::X;
                    error_count += 1;
                } else {
                    self.qubits[i][j].error = ErrorType::I;
                }
            }
        }
        error_count
    }
    pub fn error_pattern(&self) -> Vec<String> {
        let length = 2 * self.d - 1;
        (0..length).map(|i| {
            (0..length).map(|j| {
                match self.qubits[i][j].error {
                    ErrorType::X => 'X',
                    ErrorType::Y => 'Y',
                    ErrorType::Z => 'Z',
                    _ => 'I',
                }
            }).collect()
        }).collect()
    }
    pub fn load_error_pattern(&mut self, error_pattern: &Vec<String>) {
        self.reinitialize();
        for (i, row) in error_pattern.iter().enumerate() {
            for (j, error) in row.chars().enumerate() {
                self.qubits[i][j].error = match error {
                    'I' => ErrorType::I,
                    'X' => ErrorType::X,
                    'Y' => ErrorType::Y,
                    'Z' => ErrorType::Z,
                    _ => { panic!("unknown error: {}", error) },
                };
            }
        }
        self.error_changed();
    }
    pub fn create_with_error_pattern(error_pattern: &Vec<String>) -> Self {
        let length = error_pattern.len();
        let d = (length + 1) / 2;
        let mut decoder = create_standard_planar_code_offer_decoder(d);
        decoder.load_error_pattern(error_pattern);
        decoder
    }
    /// potential improvement: train this function to get a better result
    fn accept_probability_next(disable_probabilistic_accept: bool, accept_probability: f64) -> f64 {
        if disable_probabilistic_accept { 
            1.  // always 100% accept
        } else if accept_probability > 0.5 {
            0.5
        } else {
            accept_probability * 0.8  // slowly degrade
        }
    }
    pub fn reproducible_error_generator_set_seed(&mut self, seed: u64) {
        self.reproducible_error_generator = Xoroshiro128StarStar::seed_from_u64(seed);
    }
}

/// create decoder for standard planar code
pub fn create_standard_planar_code_offer_decoder(d: usize) -> OfferDecoder {
    OfferDecoder {
        d: d,
        default_max_hop: d,
        qubits: (0 .. 2 * d - 1).map(|i| {
            (0 .. 2 * d - 1).map(|j| {
                let qubit_type = if (i + j) % 2 == 0 { QubitType::Data } else {
                    if i % 2 == 0 { QubitType::StabZ } else { QubitType::StabX }
                };
                let boundary_cost = if qubit_type == QubitType::Data { f64::NAN } else {
                    if qubit_type == QubitType::StabZ {
                        std::cmp::min((j + 1) / 2, d - (j + 1) / 2) as f64
                    } else {
                        std::cmp::min((i + 1) / 2, d - (i + 1) / 2) as f64
                    }
                };
                let mut neighbors = Vec::new();
                let mut xor_data_qubits = Vec::new();
                if qubit_type != QubitType::Data {
                    if i >= 2 { neighbors.push((i - 2, j, 1.)); }
                    if i + 2 < d * 2 - 1 { neighbors.push((i + 2, j, 1.)); }
                    if j >= 2 { neighbors.push((i, j - 2, 1.)); }
                    if j + 2 < d * 2 - 1 { neighbors.push((i, j + 2, 1.)); }

                    if i >= 1 { xor_data_qubits.push((i - 1, j)); }
                    if i + 1 < d * 2 - 1 { xor_data_qubits.push((i + 1, j)); }
                    if j >= 1 { xor_data_qubits.push((i, j - 1)); }
                    if j + 1 < d * 2 - 1 { xor_data_qubits.push((i, j + 1)); }
                }
                Qubit {
                    i: i,
                    j: j,
                    qubit_type: qubit_type,
                    error: ErrorType::I,
                    measurement: false,
                    // for stabilizers only
                    mailbox: Vec::new(),
                    out_queue: Vec::new(),
                    active_timestamp: 0,
                    offer_cache: HashMap::new(),
                    loop_cache: HashMap::new(),
                    state: NodeState::NoError,
                    boundary_cost: boundary_cost,
                    cost: 0.,
                    broker_next_hop: None,
                    match_with: None,
                    accept_probability: 1.,
                    // helper variables
                    neighbors: neighbors,
                    xor_data_qubits: xor_data_qubits,
                }
            }).collect()
        }).collect(),
        // statistics
        message_count_single_round: 0,
        message_count: 0,
        has_potential_acceptance: false,
        // random generator
        disable_probabilistic_accept: false,
        rng: thread_rng(),
        use_reproducible_error_generator: false,  // by default use thread_rng
        reproducible_error_generator: Xoroshiro128StarStar::seed_from_u64(0),  // change to other random seed when using reproducible error generator
    }
}
