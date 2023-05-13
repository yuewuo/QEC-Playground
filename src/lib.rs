#![cfg_attr(
    feature="python_binding",
    feature(cfg_eval)
)]

extern crate clap;
#[macro_use] extern crate serde_json;
extern crate ndarray;
extern crate rand;
extern crate actix_web;
extern crate actix_cors;
extern crate serde;
extern crate libc;
extern crate num_cpus;
extern crate petgraph;
extern crate pbr;
extern crate rand_core;
extern crate derivative;
extern crate derive_more;
extern crate lazy_static;
extern crate either;
extern crate shlex;
extern crate cfg_if;
#[cfg(feature="python_binding")]
#[macro_use] extern crate pyo3;
extern crate platform_dirs;
extern crate serde_hashkey;
extern crate float_cmp;
extern crate priority_queue;
extern crate float_ord;
extern crate parking_lot;
#[cfg(feature="fusion_blossom")]
extern crate fusion_blossom;
extern crate chrono;
extern crate urlencoding;
#[cfg(feature="hyperion")]
extern crate mwps;
#[macro_use] extern crate enum_dispatch;

pub mod util;
pub mod test;
pub mod tool;
pub mod types;
pub mod web;
pub mod cli;
pub mod blossom_v;
pub mod reproducible_rand;
// pub mod distributed_uf_decoder;  TODO: migrate back
// pub mod fpga_generator;  TODO: migrate back
// pub mod fast_benchmark;  TODO: migrate back
pub mod simulator;
pub mod code_builder;
#[macro_use] pub mod util_macros;
pub mod model_graph;
pub mod complete_model_graph;
pub mod noise_model;
pub mod decoder_mwpm;
pub mod decoder_tailored_mwpm;
pub mod decoder_union_find;
pub mod tailored_model_graph;
pub mod tailored_complete_model_graph;
pub mod noise_model_builder;
pub mod union_find;
pub mod erasure_graph;
#[cfg(feature="fusion_blossom")]
pub mod decoder_fusion;
pub mod visualize;
pub mod model_hypergraph;
#[cfg(feature="hyperion")]
pub mod decoder_hyper_union_find;
#[cfg(feature="python_binding")]
use pyo3::prelude::*;
pub mod simulator_compact;


#[cfg(feature="python_binding")]
#[pymodule]
fn qecp(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    simulator::register(py, m)?;
    types::register(py, m)?;
    code_builder::register(py, m)?;
    noise_model::register(py, m)?;
    noise_model_builder::register(py, m)?;
    visualize::register(py, m)?;
    util::register(py, m)?;
    let helper_code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/helper.py"));
    let helper_module = PyModule::from_code(py, helper_code, "helper", "helper")?;
    helper_module.add("visualizer_website", generate_visualizer_website(py))?;
    let bottle_code = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/bottle.py"));  // embed bottle
    helper_module.add_submodule(PyModule::from_code(py, bottle_code, "bottle", "bottle")?)?;
    m.add_submodule(helper_module)?;
    let helper_register = helper_module.getattr("register")?;
    helper_register.call1((m, ))?;
    Ok(())
}

#[cfg(feature="python_binding")]
macro_rules! include_visualize_file {
    ($mapping:ident, $filepath:expr) => {
        $mapping.insert($filepath.to_string(), include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/visualize/", $filepath)).to_string());
    };
    ($mapping:ident, $filepath:expr, $($other_filepath:expr),+) => {
        include_visualize_file!($mapping, $filepath);
        include_visualize_file!($mapping, $($other_filepath),+);
    };
}

#[cfg(feature="python_binding")]
fn generate_visualizer_website(py: Python<'_>) -> &pyo3::types::PyDict {
    use pyo3::types::IntoPyDict;
    let mut mapping = std::collections::BTreeMap::<String, String>::new();
    include_visualize_file!(mapping, "gui3d.js", "index.js", "patches.js", "cmd.js", "mocker.js");
    include_visualize_file!(mapping, "index.html", "icon.svg");
    include_visualize_file!(mapping, "package.json", "package-lock.json");
    mapping.into_py_dict(py)
}
