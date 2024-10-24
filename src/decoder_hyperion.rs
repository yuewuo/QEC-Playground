//! Hypergraph Minimum-Weight Parity Subgraph decoder (Hyperion)

use super::decoder_mwpm::*;
use super::model_graph::*;
use super::noise_model::*;
use super::simulator::*;
use crate::model_hypergraph::*;
use crate::mwpf::{bp::bp::*, mwpf_solver::*, util::*};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;

pub struct HyperionDecoder {
    /// model hypergraph
    pub model_hypergraph: Arc<ModelHypergraph>,
    /// save configuration for later usage
    pub config: HyperionDecoderConfig,
    /// (approximate) minimum-weight parity factor solver
    pub solver: SolverSerialJointSingleHair,
    /// the initializer of the solver, used for customized clone
    pub initializer: Arc<SolverInitializer>,
    /// bp decoder if in use
    pub bp_decoder: Option<BpDecoder>,
    /// initial log ratios for bp decoder if in use
    pub initial_log_ratios: Option<Vec<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HyperionDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")] // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")] // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    /// the maximum integer weight after scaling
    #[serde(alias = "mhw")] // abbreviation
    #[serde(default = "hyperion_default_configs::max_weight")]
    pub max_weight: usize,
    #[serde(default = "hyperion_default_configs::default_hyperion_config")]
    pub hyperion_config: serde_json::Value,
    #[serde(default = "hyperion_default_configs::substitute_with_simple_graph")]
    pub substitute_with_simple_graph: bool,
    #[serde(default = "hyperion_default_configs::use_bp")]
    pub use_bp: bool,
}

pub mod hyperion_default_configs {
    pub fn max_weight() -> usize {
        1000000
    }
    pub fn default_hyperion_config() -> serde_json::Value {
        json!({})
    }
    pub fn substitute_with_simple_graph() -> bool {
        false
    }
    pub fn use_bp() -> bool {
        false
    }
}

impl Clone for HyperionDecoder {
    fn clone(&self) -> Self {
        Self {
            model_hypergraph: self.model_hypergraph.clone(),
            config: self.config.clone(),
            solver: SolverSerialJointSingleHair::new(&self.initializer, self.config.hyperion_config.clone()),
            initializer: self.initializer.clone(),
            bp_decoder: self.bp_decoder.clone(),
            initial_log_ratios: self.initial_log_ratios.clone(),
        }
    }
}

impl HyperionDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(
        simulator: &Simulator,
        noise_model: Arc<NoiseModel>,
        decoder_configuration: &serde_json::Value,
        parallel: usize,
        use_brief_edge: bool,
    ) -> Self {
        // read attribute of decoder configuration
        let config: HyperionDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_hypergraph = ModelHypergraph::new(&simulator);
        if config.substitute_with_simple_graph {
            let mut model_graph = ModelGraph::new(&simulator);
            model_graph.build(
                &mut simulator,
                noise_model,
                &config.weight_function,
                parallel,
                config.use_combined_probability,
                use_brief_edge,
            );
            model_hypergraph.load_from_model_graph(&model_graph);
        } else {
            model_hypergraph.build(
                &mut simulator,
                Arc::clone(&noise_model),
                &config.weight_function,
                parallel,
                config.use_combined_probability,
                use_brief_edge,
            );
        }
        let model_hypergraph = Arc::new(model_hypergraph);
        let (vertex_num, weighted_edges) = model_hypergraph.generate_mwpf_hypergraph(config.max_weight);

        let check_size = weighted_edges.len();

        let initializer = Arc::new(SolverInitializer::new(vertex_num, weighted_edges));
        let solver = SolverSerialJointSingleHair::new(&initializer, config.hyperion_config.clone());

        let mut bp_decoder_option = None;
        let mut initial_log_ratios_option = None;

        if config.use_bp {
            let mut pcm = BpSparse::new(vertex_num, check_size, 0);
            let mut initial_log_ratios = Vec::with_capacity(check_size);
            let mut channel_probabilites = Vec::with_capacity(check_size);

            for (col_index, (defect_vertices, hyperedge_group)) in model_hypergraph.weighted_edges.iter().enumerate() {
                channel_probabilites.push(hyperedge_group.hyperedge.probability);
                for vertex_position in defect_vertices.0.iter() {
                    let row_index = model_hypergraph.vertex_indices.get(vertex_position).unwrap();
                    pcm.insert_entry(*row_index, col_index);
                }
                initial_log_ratios.push(hyperedge_group.hyperedge.weight as f64);
            }

            let bp_decoder = BpDecoder::new_3(pcm, channel_probabilites, 1).unwrap();

            bp_decoder_option = Some(bp_decoder);
            initial_log_ratios_option = Some(initial_log_ratios);
        }

        Self {
            model_hypergraph,
            config,
            solver,
            initializer,
            bp_decoder: bp_decoder_option,
            initial_log_ratios: initial_log_ratios_option,
        }
    }

    /// decode given measurement results
    #[allow(dead_code)]
    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        self.decode_with_erasure(sparse_measurement, &SparseErasures::new())
    }

    /// decode given measurement results and detected erasures
    pub fn decode_with_erasure(
        &mut self,
        sparse_measurement: &SparseMeasurement,
        sparse_detected_erasures: &SparseErasures,
    ) -> (SparseCorrection, serde_json::Value) {
        if !sparse_detected_erasures.is_empty() {
            unimplemented!()
        }
        // run decode
        let begin = Instant::now();

        let mut syndrome_array = vec![];
        if self.config.use_bp {
            syndrome_array = vec![0; self.model_hypergraph.vertex_indices.len()];
        }

        let defect_vertices: Vec<_> = sparse_measurement
            .iter()
            .map(|position| {
                let temp = *self
                    .model_hypergraph
                    .vertex_indices
                    .get(position)
                    .expect("measurement cannot happen at impossible position");
                if self.config.use_bp {
                    syndrome_array[temp] = 1;
                }
                temp
            })
            .collect();

        let syndrome_pattern = SyndromePattern::new(defect_vertices, vec![]);

        let decoder_begin = Instant::now();

        if self.config.use_bp {
            self.bp_decoder
                .as_mut()
                .unwrap()
                .set_log_domain_bp(&self.initial_log_ratios.as_ref().unwrap());

            // solve the bp and update weights
            self.bp_decoder.as_mut().unwrap().decode(&syndrome_array);
            let mut llrs = self.bp_decoder.as_ref().unwrap().log_prob_ratios.clone();

            // note: honestly, this is not really needed. But it is a good practice to keep the model graph consistent, comment out if need more speed
            // note: unsafe, but sound if only one decoder is using this

            self.solver.update_weights(&mut llrs);
        }

        let time_decode_bp = decoder_begin.elapsed().as_secs_f64();

        self.solver.solve(&syndrome_pattern);
        let subgraph = self.solver.subgraph();
        self.solver.clear();

        let time_decode_mwpf = decoder_begin.elapsed().as_secs_f64() - time_decode_bp;
        let time_decode = begin.elapsed().as_secs_f64();

        // build correction
        let begin = Instant::now();
        let mut correction = SparseCorrection::new();
        for &edge_index in subgraph.iter() {
            correction.extend(&self.model_hypergraph.weighted_edges[edge_index].1.hyperedge.correction);
        }
        let time_build_correction = begin.elapsed().as_secs_f64();
        (
            correction,
            json!({
                "time_decode": time_decode,
                "time_build_correction": time_build_correction,
                "time_decode_bp": time_decode_bp,
                "time_decode_mwpf": time_decode_mwpf,
            }),
        )
    }
}
