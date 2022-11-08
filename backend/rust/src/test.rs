#![allow(non_snake_case)]

use super::clap;
#[cfg(feature="python_binding")]
use super::pyo3::prelude::*;
#[cfg(feature="python_binding")]
use super::pyo3::types::{IntoPyDict};
use super::blossom_v;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        Some(("debug_tests", _)) => {
            debug_tests()
        }
        Some(("archived_debug_tests", _)) => {
            archived_debug_tests()
        }
        Some(("all", _)) => {  // remember to add new test functions here
            debug_tests();
            archived_debug_tests();
        }
        _ => unreachable!()
    }
}

fn archived_debug_tests() {
    cfg_if::cfg_if! {
        if #[cfg(feature="python_binding")] {
            // call python networkx.algorithms.matching.max_weight_matching
            Python::with_gil(|py| {
                (|py: Python| -> PyResult<()> {
                    let networkx = py.import("networkx")?;
                    let G = networkx.call_method0("Graph")?;
                    let weighted_edges = vec![
                        (0, 1, -3.),
                        (1, 2, -2.),
                        (2, 0, -3.),
                        (0, 3, -1.),
                        (1, 4, -2.),
                        (2, 5, -1.),
                        (3, 4, 0.),
                        (3, 5, 0.),
                        (4, 5, 0.),
                    ].to_object(py);
                    G.call_method1("add_weighted_edges_from", (weighted_edges,))?;
                    let max_weight_matching = networkx.getattr("algorithms")?.getattr("matching")?.getattr("max_weight_matching")?;
                    let dict = vec![("maxcardinality", true)].into_py_dict(py);
                    let matched: std::collections::HashSet<(usize, usize)> = max_weight_matching.call((G,), Some(dict))?.extract()?;
                    println!("{:?}", matched);
                    Ok(())
                })(py).map_err(|e| {
                    e.print_and_set_sys_last_vars(py);
                })
            }).expect("python run failed");
        } else {
            println!("[error] compiling feature `python_binding` not enabled")
        }
    }
    {  // call blossom V matching
        let weighted_edges = vec![
            (0, 1, -3.),
            (1, 2, -2.),
            (2, 0, -3.),
            (0, 3, -1.),
            (1, 4, -2.),
            (2, 5, -1.),
            (3, 4, 0.),
            (3, 5, 0.),
            (4, 5, 0.),
        ];
        let matched = blossom_v::maximum_weight_perfect_matching_compatible(6, weighted_edges);
        println!("{:?}", matched);
    }
}

fn debug_tests() {
}
