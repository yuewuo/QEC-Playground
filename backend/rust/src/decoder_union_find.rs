//! union-find decoder (weighted)
//! 

use serde::{Serialize, Deserialize};
use super::simulator::*;
use super::error_model::*;
use super::model_graph::*;
use super::complete_model_graph::*;
use super::serde_json;
use super::decoder_mwpm::*;
use super::union_find::*;
use std::sync::{Arc};
use std::time::Instant;
use std::collections::{BTreeSet, BTreeMap};

/// MWPM decoder, initialized and cloned for multiple threads
#[derive(Debug, Clone, Serialize)]
pub struct UnionFindDecoder {
    /// model graph is immutably shared, just in case need to print out real weights instead of scaled and truncated weights
    pub model_graph: Arc<ModelGraph>,
    /// index to position mapping (immutable shared), index is the one used in the union-find algorithm
    pub index_to_position: Arc<Vec<Position>>,
    /// decoder nodes, each corresponds to a node in the model graph; each instance needs to modify node information and thus not shared
    pub nodes: Vec::< Vec::< Vec::< Option< Box< UnionFindDecoderNode > > > > >,
    /// union-find algorithm
    pub union_find: UnionFind,
    /// recording the list of odd clusters to reduce iteration complexity
    pub odd_clusters: Vec<usize>,
    /// query whether an index is in `odd_clusters`, to avoid duplicate odd clusters in the list
    pub odd_clusters_set: BTreeSet<usize>,
    /// record the boundary nodes as an optimization, see <https://arxiv.org/pdf/1709.06218.pdf> Section "Boundary representation".
    /// even clusters should not be key in HashMap, and only real boundary should be in the `HashSet` value;
    /// those nodes without error syndrome also have entries in this HashMap, with the value of { itself }
    pub cluster_boundaries: BTreeMap<usize, (Vec<usize>, BTreeSet<usize>)>,
    /// trace: study the time consumption of each step
    pub time_uf_grow: f64,
    pub time_uf_merge: f64,
    pub time_uf_replace: f64,
    pub time_uf_update: f64,
    pub time_uf_remove: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UnionFindDecoderNode {
    /// the index used in union-find algorithm
    pub index: usize,

}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnionFindDecoderConfig {
    /// weight function, by default using [`WeightFunction::AutotuneImproved`]
    #[serde(alias = "wf")]  // abbreviation
    #[serde(default = "mwpm_default_configs::weight_function")]
    pub weight_function: WeightFunction,
    /// maximum weight will be 2 * max_half_weight, so that each time an edge can grow 1; by default is 1: unweighted union-find decoder
    #[serde(alias = "mhw")]  // abbreviation
    #[serde(default = "union_find_default_configs::max_half_weight")]
    pub max_half_weight: usize,
    /// real-weighted union-find decoder assuming weights are large integers, and try to handle them as real numbers (various growth step);
    /// by default is false: the original union-find decoder
    #[serde(alias = "rw")]  // abbreviation
    #[serde(default = "union_find_default_configs::real_weighted")]
    pub real_weighted: bool,
}

pub mod union_find_default_configs {
    pub fn max_half_weight() -> usize { 1 }
    pub fn real_weighted() -> bool { false }
}

impl UnionFindDecoder {
    /// create a new MWPM decoder with decoder configuration
    pub fn new(simulator: &Simulator, error_model: &ErrorModel, decoder_configuration: &serde_json::Value) -> Self {
        // read attribute of decoder configuration
        let config: UnionFindDecoderConfig = serde_json::from_value(decoder_configuration.clone()).unwrap();
        // build model graph
        let mut simulator = simulator.clone();
        let mut model_graph = ModelGraph::new(&simulator);
        model_graph.build(&mut simulator, &error_model, &config.weight_function);
        // build complete model graph
        
        Self {
            model_graph: Arc::new(model_graph),
        }
    }

    pub fn decode(&mut self, sparse_measurement: &SparseMeasurement) -> (SparseCorrection, serde_json::Value) {
        let mut correction = SparseCorrection::new();
        (correction, json!({

        }))
    }

}
