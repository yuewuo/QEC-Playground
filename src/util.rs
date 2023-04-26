#![allow(non_snake_case)]

use std::fs;
use super::platform_dirs::AppDirs;
use super::lazy_static::lazy_static;
use std::sync::{RwLock};
use std::collections::{BTreeMap};
use std::path::{Path, PathBuf};


/// filename should contain .py, folders should end with slash
#[allow(dead_code)]
pub fn getFileContentFromMultiplePlaces(folders: &Vec<String>, filename: &String) -> Result<String, String> {
    for folder in folders {
        let path = Path::new(folder).join(filename.as_str());
        if path.exists() {
            if let Some(path_str) = path.to_str() {
                let contents = fs::read_to_string(path_str);
                if let Ok(content) = contents {
                    return Ok(content);
                }
            }
        }
    }
    Err(format!("cannot find '{}' from folders {:?}", filename, folders))
}

// https://users.rust-lang.org/t/hashmap-performance/6476/8
// https://gist.github.com/arthurprs/88eef0b57b9f8341c54e2d82ec775698
// a much simpler but super fast hasher, only suitable for `ftqec::Index`!!!
pub mod simple_hasher {
    use std::hash::Hasher;
    pub struct SimpleHasher(u64);

    #[inline]
    fn load_u64_le(buf: &[u8], len: usize) -> u64 {
        use std::ptr;
        debug_assert!(len <= buf.len());
        let mut data = 0u64;
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), &mut data as *mut _ as *mut u8, len);
        }
        data.to_le()
    }


    impl Default for SimpleHasher {

        #[inline]
        fn default() -> SimpleHasher {
            SimpleHasher(0)
        }
    }

    // impl SimpleHasher {
    //     #[inline]
    //     pub fn set_u64(&mut self, value: u64) {
    //         self.0 = value;
    //     }
    // }

    impl Hasher for SimpleHasher {

        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            if self.0 != 0 {
                panic!("do not use SimpleHasher for struct other than ftqec::Index");
            }
            let value = load_u64_le(bytes, bytes.len());
            // println!("value: {}", value);
            *self = SimpleHasher(value);
        }
    }
}


#[allow(dead_code)]
pub const TEMPORARY_STORE_MAX_COUNT: usize = 10;  // 100MB max, this option only applies to in memory temporary store; for file-based store, it will not delete any file for safety consideration

pub struct TemporaryStore {
    use_file: bool,  // save data to file instead of in memory, this will also let data persist over program restart
    temporary_store_folder: PathBuf,
    memory_store: BTreeMap<usize, String>,  // in memory store, will not be used if `use_file` is set to true
}

lazy_static! {
    // must use RwLock, because web request will lock as a reader, and tool.rs will also acquire a reader lock
    pub static ref TEMPORARY_STORE: RwLock<TemporaryStore> = RwLock::new(TemporaryStore {
        use_file: true,  // suitable for low memory machines, by default
        temporary_store_folder: AppDirs::new(Some("qec"), true).unwrap().data_dir.join("temporary-store"),
        memory_store: BTreeMap::new(),
    });
}

pub fn local_get_temporary_store(resource_id: usize) -> Option<String> {
    let temporary_store = TEMPORARY_STORE.read().unwrap();
    if temporary_store.use_file {
        match fs::create_dir_all(&temporary_store.temporary_store_folder) {
            Ok(_) => { },
            Err(_) => { return None },  // cannot open folder
        }
        match fs::read_to_string(temporary_store.temporary_store_folder.join(format!("{}.dat", resource_id))) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    } else {
        match temporary_store.memory_store.get(&resource_id) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

pub fn local_put_temporary_store(value: String) -> Option<usize> {
    let mut temporary_store = TEMPORARY_STORE.write().unwrap();
    let mut insert_key = 1;  // starting from 1
    if temporary_store.use_file {
        match fs::create_dir_all(&temporary_store.temporary_store_folder) {
            Ok(_) => { },
            Err(_) => { return None },  // cannot create folder
        }
        let paths = match fs::read_dir(&temporary_store.temporary_store_folder) {
            Ok(paths) => { paths },
            Err(_) => { return None },  // cannot read folder
        };
        for path in paths {
            if path.is_err() {
                continue
            }
            let path = path.unwrap().path();
            if path.extension() != Some(&std::ffi::OsStr::new("dat")) {
                continue
            }
            match path.file_stem() {
                Some(file_stem) => {
                    match file_stem.to_string_lossy().parse::<usize>() {
                        Ok(this_key) => {
                            if this_key >= insert_key {
                                insert_key = this_key + 1;
                            }
                        },
                        Err(_) => { },
                    }
                },
                None => { },
            }
        }
        if fs::write(temporary_store.temporary_store_folder.join(format!("{}.dat", insert_key)), value.as_bytes()).is_err() {
            return None;  // failed to write file
        }
    } else {
        let keys: Vec<usize> = temporary_store.memory_store.keys().cloned().collect();
        if keys.len() > 0 {
            insert_key = keys[keys.len() - 1] + 1
        }
        if keys.len() >= TEMPORARY_STORE_MAX_COUNT {  // delete the first one
            temporary_store.memory_store.remove(&keys[0]);
        }
        temporary_store.memory_store.insert(insert_key, value);
    }
    Some(insert_key)
}

cfg_if::cfg_if! { if #[cfg(feature="fusion_blossom")] {

use std::sync::{Arc, Mutex};
use std::collections::{HashMap};
use crate::util_macros::*;
use serde::{Serialize, Deserialize};
use super::decoder_mwpm::*;
use crate::model_graph::*;
use crate::noise_model::NoiseModel;
use crate::simulator::*;
use crate::types::QubitType;

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
        use fusion_blossom::mwpm_solver::*;
        use fusion_blossom::util::*;
        use fusion_blossom::visualize::*;
        let config: FusionBlossomSyndromeExporterConfig = serde_json::from_value(config.clone()).unwrap();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(simulator, noise_model_graph, &config.weight_function, parallel_init, config.use_combined_probability, use_brief_edge);
        let stabilizer_filter = if config.only_stab_z { FusionBlossomStabilizerFilter::StabZOnly } else { FusionBlossomStabilizerFilter::None };
        // generate model graph, and then generate a graph initializer and positions
        // when the syndrome is partial: e.g. only StabX or only StabZ, the graph initializer should also contain only part of it
        let mut initializer = SolverInitializer::new(0, vec![], vec![]);
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
}

} }

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn temporary_store_read_files() {  // cargo test temporary_store_read_files -- --nocapture
        let resource_id_1 = local_put_temporary_store(format!("hello")).unwrap();
        let resource_id_2 = local_put_temporary_store(format!("world")).unwrap();
        // println!("{:?}", resource_id_1);
        // println!("{:?}", resource_id_2);
        let read_1 = local_get_temporary_store(resource_id_1);
        let read_2 = local_get_temporary_store(resource_id_2);
        assert_eq!(read_1, Some(format!("hello")));
        assert_eq!(read_2, Some(format!("world")));
    }
}
