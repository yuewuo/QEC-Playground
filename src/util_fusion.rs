use std::sync::{Arc, Mutex};
use std::collections::{HashMap};
use crate::util_macros::*;
use serde::{Serialize, Deserialize};
use super::decoder_mwpm::*;
use crate::model_graph::*;
use crate::noise_model::NoiseModel;
use crate::simulator::*;
use crate::types::QubitType;
use crate::fusion_blossom::util::*;
use crate::fusion_blossom::mwpm_solver::*;
use crate::fusion_blossom::visualize::*;


pub struct FusionBlossomSyndromeExporter {
    /// logger
    pub solver_error_pattern_logger: Mutex<fusion_blossom::mwpm_solver::SolverErrorPatternLogger>,
    /// filter stabilizers
    pub stabilizer_filter: FusionBlossomStabilizerFilter,
    /// vertex index map to position
    pub vertex_to_position_mapping: Vec<Position>,
    /// position map to vertex index
    pub position_to_vertex_mapping: HashMap<Position, usize>,
}

pub enum FusionBlossomStabilizerFilter {
    None,
    StabZOnly,
}

impl FusionBlossomStabilizerFilter {
    pub fn ignore_node(&self, node: &SimulatorNode) -> bool {
        match self {
            Self::None => false,
            Self::StabZOnly => node.qubit_type != QubitType::StabZ,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "python_binding", cfg_eval)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub struct FusionBlossomSyndromeExporterConfig {
    /// see [`MWPMDecoderConfig`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// combined probability can improve accuracy, but will cause probabilities differ a lot even in the case of i.i.d. noise model
    #[serde(alias = "ucp")]  // abbreviation
    #[serde(default = "mwpm_default_configs::use_combined_probability")]
    pub use_combined_probability: bool,
    /// the output filename
    pub filename: String,
    /// only export Z stabilizers
    #[serde(default = "fusion_blossom_syndrome_exporter_default_configs::only_stab_z")]
    pub only_stab_z: bool,
    #[serde(alias = "mhw")]  // abbreviation
    #[serde(default = "fusion_blossom_syndrome_exporter_default_configs::max_half_weight")]
    pub max_half_weight: usize,
}

pub mod fusion_blossom_syndrome_exporter_default_configs {
    pub fn only_stab_z() -> bool { false }
    pub fn max_half_weight() -> usize { 5000 }
}

impl FusionBlossomSyndromeExporter {

    pub fn new(config: &serde_json::Value, simulator: &mut Simulator, noise_model_graph: Arc<NoiseModel>, parallel_init: usize, use_brief_edge: bool) -> Self {
        let config: FusionBlossomSyndromeExporterConfig = serde_json::from_value(config.clone()).unwrap();
        let (initializer, positions, vertex_to_position_mapping, position_to_vertex_mapping, stabilizer_filter) = Self::construct_initializer(&config, simulator, noise_model_graph, parallel_init, use_brief_edge);
        let solver_error_pattern_logger = SolverErrorPatternLogger::new(&initializer, &positions, json!({ "filename": config.filename }));
        Self {
            solver_error_pattern_logger: Mutex::new(solver_error_pattern_logger),
            stabilizer_filter,
            vertex_to_position_mapping,
            position_to_vertex_mapping,
        }
    }
    pub fn add_syndrome(&self, sparse_measurement: &SparseMeasurement, sparse_detected_erasures: &SparseErasures) {
        use fusion_blossom::mwpm_solver::*;
        use fusion_blossom::util::*;
        let mut syndrome_pattern = SyndromePattern::new_empty();
        for defect_vertex in sparse_measurement.iter() {
            if self.position_to_vertex_mapping.contains_key(defect_vertex) {
                syndrome_pattern.defect_vertices.push(*self.position_to_vertex_mapping.get(defect_vertex).unwrap());
            }
        }
        assert!(sparse_detected_erasures.len() == 0, "unimplemented");
        self.solver_error_pattern_logger.lock().unwrap().solve_visualizer(&syndrome_pattern, None);
    }

    pub fn construct_initializer(config: &FusionBlossomSyndromeExporterConfig, simulator: &mut Simulator, noise_model_graph: Arc<NoiseModel>, parallel_init: usize, use_brief_edge: bool) -> (SolverInitializer, Vec<VisualizePosition>, Vec<Position>, HashMap<Position, usize>, FusionBlossomStabilizerFilter) {
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(simulator, noise_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
        let stabilizer_filter = if config.only_stab_z { FusionBlossomStabilizerFilter::StabZOnly } else { FusionBlossomStabilizerFilter::None };
        // generate model graph, and then generate a graph initializer and positions
        // when the syndrome is partial: e.g. only StabX or only StabZ, the graph initializer should also contain only part of it
        let mut initializer: SolverInitializer = SolverInitializer::new(0, vec![], vec![]);
        let mut positions: Vec<VisualizePosition> = vec![];
        let mut vertex_to_position_mapping = vec![];
        let mut position_to_vertex_mapping = HashMap::new();
        simulator_iter!(simulator, position, node, {  // first insert nodes and build mapping, different from decoder_fusion, here we add virtual nodes as well
            if position.t != 0 && node.gate_type.is_measurement() && !stabilizer_filter.ignore_node(node) {
                let vertex_index = initializer.vertex_num;
                if !simulator.is_node_real(position) {
                    initializer.virtual_vertices.push(vertex_index);
                }
                positions.push(VisualizePosition::new(position.i as f64, position.j as f64, position.t as f64 / simulator.measurement_cycles as f64 * 2.));
                vertex_to_position_mapping.push(position.clone());
                position_to_vertex_mapping.insert(position.clone(), vertex_index);
                initializer.vertex_num += 1;
            }
        });
        let mut weighted_edges_unscaled = Vec::<(usize, usize, f64)>::new();
        simulator_iter!(simulator, position, node, {  // then add edges and also virtual nodes
            if position.t != 0 && node.gate_type.is_measurement() && simulator.is_node_real(position) && !stabilizer_filter.ignore_node(node) {
                let model_graph_node = model_graph.get_node_unwrap(position);
                let vertex_index = position_to_vertex_mapping[&position];
                if let Some(model_graph_boundary) = &model_graph_node.boundary {
                    let virtual_position = model_graph_boundary.virtual_node.as_ref().expect("virtual boundary required to plot properly in fusion blossom");
                    let virtual_index = position_to_vertex_mapping[&virtual_position];
                    weighted_edges_unscaled.push((vertex_index, virtual_index, model_graph_boundary.weight));
                }
                for (peer_position, model_graph_edge) in model_graph_node.edges.iter() {
                    let peer_idx = position_to_vertex_mapping[peer_position];
                    if vertex_index < peer_idx {  // avoid duplicate edges
                        weighted_edges_unscaled.push((vertex_index, peer_idx, model_graph_edge.weight));
                    }
                }
            }
        });
        initializer.weighted_edges = {  // re-weight edges and parse to integer
            let mut maximum_weight = 0.;
            for (_, _, weight) in weighted_edges_unscaled.iter() {
                if weight > &maximum_weight {
                    maximum_weight = *weight;
                }
            }
            let scale: f64 = config.max_half_weight as f64 / maximum_weight;
            weighted_edges_unscaled.iter().map(|(a, b, weight)| (*a, *b, 2 * (weight * scale).ceil() as fusion_blossom::util::Weight)).collect()
        };
        (initializer, positions, vertex_to_position_mapping, position_to_vertex_mapping, stabilizer_filter)
    }

}

/// a visualize-able initializer
#[derive(Debug, Clone)]
pub struct SolverInitializerVis {
    pub initializer: SolverInitializer,
    pub positions: Vec<VisualizePosition>,
}

impl SolverInitializerVis {
    pub fn new(initializer: SolverInitializer, positions: Vec<VisualizePosition>) -> Self {
        Self { initializer, positions }
    }
    pub fn assert_eq(&self, other: &SolverInitializerVis) -> Result<(), String> {
        if self.initializer.vertex_num != other.initializer.vertex_num {
            return Err(format!("vertex_num differs {} != {}", self.initializer.vertex_num, other.initializer.vertex_num));
        }
        if self.initializer.weighted_edges.len() != other.initializer.weighted_edges.len() {
            return Err(format!("weighted edges length differs {} != {}", self.initializer.weighted_edges.len(), other.initializer.weighted_edges.len()));
        }
        for index in 0..self.initializer.weighted_edges.len() {
            if self.initializer.weighted_edges[index] != other.initializer.weighted_edges[index] {
                return Err(format!("the {}-th weighted edge differs: {:?} != {:?}"
                    , index, self.initializer.weighted_edges[index], other.initializer.weighted_edges[index]));
            }
        }
        if self.initializer.virtual_vertices.len() != other.initializer.virtual_vertices.len() {
            return Err(format!("virtual vertices length differs {} != {}", self.initializer.virtual_vertices.len(), other.initializer.virtual_vertices.len()));
        }
        for index in 0..self.initializer.virtual_vertices.len() {
            if self.initializer.virtual_vertices[index] != other.initializer.virtual_vertices[index] {
                return Err(format!("the {}-th virtual vertex differs: {:?} != {:?}"
                    , index, self.initializer.virtual_vertices[index], other.initializer.virtual_vertices[index]));
            }
        }
        if self.positions.len() != other.positions.len() {
            return Err(format!("positions length differs {} != {}", self.positions.len(), other.positions.len()));
        }
        for index in 0..self.positions.len() {
            let pos1 = &self.positions[index];
            let pos2 = &other.positions[index];
            if pos1.t != pos2.t || pos1.i != pos2.i || pos1.j != pos2.j {
                return Err(format!("the {}-th position differs: {:?} != {:?}" , index, pos1, pos2));
            }
        }
        debug_assert_eq!(self.initializer.weighted_edges, other.initializer.weighted_edges, "up to this step, they should be equal");
        debug_assert_eq!(self.initializer.virtual_vertices, other.initializer.virtual_vertices, "up to this step, they should be equal");
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SolverInitializerExtender {
    pub base: SolverInitializerVis,
    /// the number of noisy_measurements of the first
    pub noisy_measurements: usize,
    /// how many vertices in each measurement cycle
    pub vertex_num_cycle: usize,
    /// repeat region of `weighted_edges`
    pub edge_repeat_region: (usize, usize),
    /// repeat region of `virtual_vertices`, note that this repeat region may not align with `edge_repeat_region`
    pub virtual_repeat_region: (usize, usize),
    /// repeat the position
    pub position_repeat_region: (usize, usize),
    /// measurement cycle, `t` will be biased by repeat * measurement_delta_t
    pub measurement_delta_t: f64,
}

impl SolverInitializerExtender {

    pub fn new(first: SolverInitializerVis, second: SolverInitializerVis, noisy_measurements: usize) -> Self {
        assert!(second.initializer.weighted_edges.len() > first.initializer.weighted_edges.len(), "must differ");
        let edge_num_differ = second.initializer.weighted_edges.len() - first.initializer.weighted_edges.len();
        let edge_repeat_start = first.initializer.weighted_edges.len() / 2 - edge_num_differ / 2;
        let edge_repeat_end = edge_repeat_start + edge_num_differ;
        let (u1, v1, w1) = first.initializer.weighted_edges[edge_repeat_start];
        let (u2, v2, w2) = first.initializer.weighted_edges[edge_repeat_end];
        assert_eq!(w1, w2, "should be the same edge at different cycles, consider increasing T to eliminate boundary effects");
        assert_eq!(u2-u1, v2-v1, "should be the same edge at different cycles, consider increasing T to eliminate boundary effects");
        let vertex_num_cycle = u2 - u1;
        let virtual_num_differ = second.initializer.virtual_vertices.len() - first.initializer.virtual_vertices.len();
        let virtual_repeat_start = first.initializer.virtual_vertices.len() / 2 - virtual_num_differ / 2;
        let virtual_repeat_end = virtual_repeat_start + virtual_num_differ;
        assert_eq!(vertex_num_cycle, first.initializer.virtual_vertices[virtual_repeat_end] - first.initializer.virtual_vertices[virtual_repeat_start]);
        let position_repeat_start = first.positions.len() / 2 - vertex_num_cycle / 2;
        let position_repeat_end = position_repeat_start + vertex_num_cycle;
        assert_eq!(vertex_num_cycle, second.positions.len() - first.positions.len(), "position number mismatch");
        let measurement_delta_t = first.positions[position_repeat_end].t - first.positions[position_repeat_start].t;
        let extender = Self {
            base: first,
            noisy_measurements,
            vertex_num_cycle,
            edge_repeat_region: (edge_repeat_start, edge_repeat_end),
            virtual_repeat_region: (virtual_repeat_start, virtual_repeat_end),
            position_repeat_region: (position_repeat_start, position_repeat_end),
            measurement_delta_t,
        };
        // use the second simulator to verify the correctness (partially)
        second.assert_eq(&extender.generate(noisy_measurements + 1)).unwrap();
        // return the verified extender
        extender
    }

    pub fn generate(&self, noisy_measurements: usize) -> SolverInitializerVis {
        let base = &self.base;
        let vertex_num_cycle = self.vertex_num_cycle;
        let (edge_repeat_start, edge_repeat_end) = self.edge_repeat_region;
        let (virtual_repeat_start, virtual_repeat_end) = self.virtual_repeat_region;
        let (position_repeat_start, position_repeat_end) = self.position_repeat_region;
        let mut result = base.clone();
        assert!(noisy_measurements >= self.noisy_measurements);
        if noisy_measurements == self.noisy_measurements {
            return result
        }
        let repeat: usize = noisy_measurements - self.noisy_measurements;
        result.initializer.vertex_num += vertex_num_cycle * repeat;
        result.initializer.weighted_edges.drain(edge_repeat_end..);
        result.initializer.weighted_edges.reserve_exact(repeat * (edge_repeat_end - edge_repeat_start));
        result.initializer.virtual_vertices.drain(virtual_repeat_end..);
        result.initializer.virtual_vertices.reserve_exact(repeat * (virtual_repeat_end - virtual_repeat_start));
        result.positions.drain(position_repeat_end..);
        result.positions.reserve_exact(repeat * (position_repeat_end - position_repeat_start));
        for i in 1..repeat+1 {
            for index in edge_repeat_start..edge_repeat_end {
                let (u, v, w) = base.initializer.weighted_edges[index];
                result.initializer.weighted_edges.push((u + i * vertex_num_cycle, v + i * vertex_num_cycle, w));
            }
            for index in virtual_repeat_start..virtual_repeat_end {
                let v = base.initializer.virtual_vertices[index];
                result.initializer.virtual_vertices.push(v + i * vertex_num_cycle);
            }
            for index in position_repeat_start..position_repeat_end {
                let mut v = base.positions[index].clone();
                v.t += i as f64 * self.measurement_delta_t;
                result.positions.push(v);
            }
        }
        for index in edge_repeat_end..base.initializer.weighted_edges.len() {
            let (u, v, w) = base.initializer.weighted_edges[index];
            result.initializer.weighted_edges.push((u + repeat * vertex_num_cycle, v + repeat * vertex_num_cycle, w));
        }
        for index in virtual_repeat_end..base.initializer.virtual_vertices.len() {
            let v = base.initializer.virtual_vertices[index];
            result.initializer.virtual_vertices.push(v + repeat * vertex_num_cycle);
        }
        for index in position_repeat_end..base.positions.len() {
            let mut v = base.positions[index].clone();
            v.t += repeat as f64 * self.measurement_delta_t;
            result.positions.push(v);
        }
        result
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::code_builder::*;
    use crate::noise_model_builder::*;
    use crate::noise_model::*;

    #[test]
    fn initializer_extender() {  // cargo test initializer_extender -- --nocapture
        let di = 3;
        let dj = 3;
        let p = 0.001;
        let build_initializer = |noisy_measurements: usize| -> SolverInitializerVis {
            let config: FusionBlossomSyndromeExporterConfig = serde_json::from_value(json!({
                "filename": "",
                "only_stab_z": true,
                "use_combined_probability": false,
            })).unwrap();
            let mut simulator = Simulator::new(CodeType::RotatedPlanarCode, CodeSize::new(noisy_measurements, di, dj));
            let mut noise_model = NoiseModel::new(&simulator);
            NoiseModelBuilder::StimNoiseModel.apply(&mut simulator, &mut noise_model, &json!({}), p, 0.5, 0.);
            code_builder_sanity_check(&simulator).unwrap();
            noise_model_sanity_check(&simulator, &noise_model).unwrap();
            let (initializer, positions, ..) = FusionBlossomSyndromeExporter::construct_initializer(&config, &mut simulator, Arc::new(noise_model), 1, true);
            SolverInitializerVis::new(initializer, positions)
        };
        let noisy_measurements = 4;
        let first = build_initializer(noisy_measurements);
        let second = build_initializer(noisy_measurements + 1);
        let extender = SolverInitializerExtender::new(first, second, noisy_measurements);
        println!("extender built successfully");
        // test a larger instance
        let test_noisy_measurement = 7;
        let generated = extender.generate(test_noisy_measurement);
        let ground_truth = build_initializer(test_noisy_measurement);
        generated.assert_eq(&ground_truth).unwrap();
    }

}
