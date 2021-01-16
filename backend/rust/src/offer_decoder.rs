use super::types::QubitType;
use super::types::ErrorType;
use std::collections::HashMap;

/// only for standard planar code
#[derive(Debug, Clone)]
pub struct OfferDecoder {
    pub d: usize,
    pub qubits: Vec< Vec<Qubit> >,
    // statistics
    pub message_count_single_round: usize,
    pub message_count: usize,
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
    pub state: NodeState,
    pub boundary_cost: f64,
    pub cost: f64,
    pub broker_next_hop: Option<(usize, usize)>,
    pub match_with: Option<(usize, usize)>,
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
    },
    // AugmentOffer,  // TODO: implement later
    AcceptOffer {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
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
    },
    AcceptBrokeredOffer {
        target: (usize, usize),
        source: (usize, usize),
    },
    BrokeredContract {
        target: (usize, usize),
        source: (usize, usize),
        broker: (usize, usize),
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
        while self.qubits[i][j].mailbox.len() > 0 {
            let qubit = &mut self.qubits[i][j];  // have to re-borrow it as mutable
            let message = qubit.mailbox.remove(0);  // take the first message in mailbox
            message_processed += 1;
            match message {
                Message::MatchOffer{ timestamp, source: (si, sj), broker: (bi, bj), cost } => {
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
                                    broker: (bi, bj),
                                    cost: cost - qubit.cost,  // minus the cost of matching pair
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
                                qubit.state = NodeState::WaitingContract;
                                qubit.out_queue.push(OutMessage {
                                    receiver: (bi, bj),  // send back to the last broker
                                    message: Message::AcceptOffer {
                                        target: (i, j),  // take this offer as target
                                        source: (si, sj),
                                        broker: (i, j),  // target is also the broker of this message
                                    },
                                });
                            }
                        }
                    }
                },
                Message::AcceptOffer{ target: (ti, tj), source: (si, sj), broker: (bi, bj) } => {
                    if si == i && sj == j {
                        if qubit.state == NodeState::SentOffer {
                            qubit.state = NodeState::Matched;
                            qubit.match_with = Some((bi, bj));  // always match with the first-hop broker
                            qubit.cost = Self::cost_of_matching(i, j, bi, bj);
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
                    assert_eq!(qubit.state, NodeState::WaitingContract, "This shoudn't happen! Contract is never sent to a node in state other than WaitingContract");
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
                    }
                    qubit.match_with = Some((bi, bj));
                    qubit.cost = Self::cost_of_matching(i, j, bi, bj);
                },
                Message::RefuseAcceptance{ target: (ti, tj), source: (si, sj) } => {
                    if ti == i && tj == j {
                        qubit.state = NodeState::SentOffer;
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
                            },
                        }
                    }
                },
                Message::BrokeredOffer{ timestamp, broker: (_bi, _bj), source: (si, sj), cost } => {
                    if qubit.state == NodeState::Matched {
                        let cached_offer = qubit.offer_cache.get(&(si, sj));
                        let not_caching_this_offer = match cached_offer {
                            Some(cached_offer) => {
                                cached_offer.timestamp < timestamp || cost < cached_offer.cost
                            },
                            None => true,
                        };
                        if not_caching_this_offer {  // ignore if already brokered a similar offer
                            if cost + qubit.boundary_cost < 0. {  // break this matched pair is an augmenting path
                                // take this offer
                                qubit.state = NodeState::WaitingContract;
                                qubit.out_queue.push(OutMessage {
                                    receiver: qubit.match_with.expect("exist"),
                                    message: Message::AcceptBrokeredOffer {
                                        target: (i, j),
                                        source: (si, sj),
                                    },
                                });
                            } else {  // propagate this offer to neighbors
                                for (ni, nj, neighbor_cost) in qubit.neighbors.iter() {
                                    qubit.out_queue.push(OutMessage {
                                        receiver: (*ni, *nj),
                                        message: Message::MatchOffer {
                                            timestamp: timestamp,
                                            source: (si, sj),
                                            broker: (i, j),  // I'm the broker (sink) of this offer
                                            cost: cost + *neighbor_cost,
                                        },
                                    });
                                }
                            }
                        }
                    } else {
                        // simply ignore this
                        // assert_eq!(qubit.state, NodeState::Matched, "why should an unmatched qubit receive a BrokeredOffer message?");
                    }
                },
                Message::AcceptBrokeredOffer{ source: (si, sj), target: (ti, tj) } => {
                    if qubit.state == NodeState::Matched {
                        let cached_offer = qubit.offer_cache.get(&(si, sj));
                        match cached_offer {
                            Some(cached_offer) => {
                                qubit.state = NodeState::WaitingContract;
                                qubit.out_queue.push(OutMessage {
                                    receiver: cached_offer.broker,  // send back to the last broker
                                    message: Message::AcceptOffer {
                                        target: (ti, tj),
                                        source: (si, sj),
                                        broker: (i, j),  // I'm the broker of this offer
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
                            qubit.state = NodeState::SentOffer;  // unlock and connect to boundary
                            qubit.cost = qubit.boundary_cost;
                            qubit.match_with = None;
                        },
                    }
                },
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
                        },
                    });
                }
                qubit.state = NodeState::SentOffer;  // offer sent and waiting for replies
            },
            NodeState::Matched => {
                // TODO: send offer from matched ones
                // smaller node is responsible for finding augmenting loop
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
            self.qubits[ri][rj].mailbox.push(out_message.message);
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
        let qubit2 = &mut self.qubits[i2][j2];
        qubit2.state = NodeState::Matched;
        qubit2.match_with = Some((i1, j1));
    }
    pub fn force_break_matched(&mut self, i: usize, j: usize) {
        let qubit = &mut self.qubits[i][j];
        if qubit.state != NodeState::Matched { return }   // no need to break
        qubit.state = NodeState::NoError;
        qubit.offer_cache = HashMap::new();
        qubit.match_with = None;
        let (mi, mj) = qubit.match_with.expect("matched qubit must have `match_with`");
        let matched_qubit = &mut self.qubits[mi][mj];
        matched_qubit.state = NodeState::NoError;
        matched_qubit.offer_cache = HashMap::new();
        matched_qubit.match_with = None;
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
                qubit.state = NodeState::NoError;
                qubit.cost = 0.;
                qubit.broker_next_hop = None;
                qubit.match_with = None;
            }
        }
        self.message_count_single_round = 0;
        self.message_count = 0;
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
    // return the cycles
    pub fn pseudo_parallel_execute_to_stable(&mut self) -> usize {
        let length = 2 * self.d - 1;
        let mut match_pattern_changed = true;
        let mut cycles = 0;
        // loop until match pattern doesn't change
        while match_pattern_changed {
            let last_match_pattern = self.match_pattern();
            match_pattern_changed = false;
            // resend offer
            for i in 0..length {
                for j in 0..length {
                    self.qubit_resend_offer(i, j);
                }
            }
            let mut message_processed = true;
            // loop until no message flying
            while message_processed {
                message_processed = false;
                for i in 0..length {
                    for j in 0..length {
                        if self.qubit_node_execute(i, j, false) > 0 {
                            message_processed = true
                        }
                        self.qubit_out_queue_send(i, j);
                    }
                }
                cycles += 1;
            }
            if self.match_pattern_changed(&last_match_pattern) {
                match_pattern_changed = true;
            }
        }
        cycles
    }
    pub fn has_logical_error(&self, error_type: ErrorType) -> bool {
        let length = 2 * self.d - 1;
        let half_length = self.d - 1;
        if error_type == ErrorType::X || error_type == ErrorType::Y {  // j = 0
            let mut boundary_error_cnt = 0;
            for i in (0..length).step_by(2) {
                match self.qubits[i][0].error {
                    ErrorType::X | ErrorType::Y => {
                        boundary_error_cnt += 1;
                    },
                    _ => { },
                }
            }
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
            let mut boundary_error_cnt = 0;
            for j in (0..length).step_by(2) {
                match self.qubits[0][j].error {
                    ErrorType::X | ErrorType::Y => {
                        boundary_error_cnt += 1;
                    },
                    _ => { },
                }
            }
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
}

/// create decoder for standard planar code
pub fn create_standard_planar_code_offer_decoder(d: usize) -> OfferDecoder {
    OfferDecoder {
        d: d,
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
                    state: NodeState::NoError,
                    boundary_cost: boundary_cost,
                    cost: 0.,
                    broker_next_hop: None,
                    match_with: None,
                    // helper variables
                    neighbors: neighbors,
                    xor_data_qubits: xor_data_qubits,
                }
            }).collect()
        }).collect(),
        // statistics
        message_count_single_round: 0,
        message_count: 0,
    }
}