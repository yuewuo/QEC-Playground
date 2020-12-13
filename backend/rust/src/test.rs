#![allow(non_snake_case)]

use super::clap;
use super::util;
use super::rand::prelude::*;
use super::serde_json;
use super::serde_json::{Value, Map};
use super::types::*;
use super::qec;
use super::pyo3::prelude::*;
use super::pyo3::types::{IntoPyDict};
use super::blossom_v;
use super::ftqec;

pub fn run_matched_test(matches: &clap::ArgMatches) {
    match matches.subcommand() {
        ("save_load", Some(_)) => {
            save_load()
        }
        ("perfect_measurement", Some(_)) => {
            perfect_measurement()
        }
        ("validate_correction", Some(_)) => {
            validate_correction()
        }
        ("naive_correction", Some(_)) => {
            naive_correction()
        }
        ("maximum_max_weight_matching_correction", Some(_)) => {
            maximum_max_weight_matching_correction()
        }
        ("debug_tests", Some(_)) => {
            debug_tests()
        }
        ("archived_debug_tests", Some(_)) => {
            archived_debug_tests()
        }
        ("all", Some(_)) => {  // remember to add new test functions here
            save_load();
            perfect_measurement();
            validate_correction();
            naive_correction();
            debug_tests();
        }
        _ => unreachable!()
    }
}

fn save_load() {
    let N = 16;
    let L = 5;
    let p = 1e-1;
    // generate some random data
    let mut data_ro = BatchZxError::new_N_L(N, L);
    let mut data = data_ro.view_mut();
    let mut rng = thread_rng();
    let mut error_cnt = 0;
    for i in 0..N {
        for j in 0..L {
            for k in 0..L {
                let is_error = rng.gen::<f64>() < p;
                if is_error {
                    error_cnt += 1;
                }
                data[[i, j, k]] = is_error;
            }
        }
    }
    assert_eq!(data_ro.N(), N);
    assert_eq!(data_ro.L(), L);
    let error_rate = error_cnt as f64 / ((N*L*L) as f64);
    println!("error/total: {}/{} = {}%", error_cnt, N*L*L, 100. * error_rate);
    // prepare the head
    let head = serde_json::json!({
        "p": p,
        "error_cnt": error_cnt,
        "error_rate": error_rate,
    });
    // save to file
    util::save("TEST_save_load.bin", &head, &data_ro).expect("save failed");
    // load from the same file
    let (head_r, data_r) = util::load("TEST_save_load.bin").expect("load failed");
    // check whether the file contains the same information
    let mut head_N_L: Map<String, Value> = serde_json::from_value(head.clone()).expect("head JSON error");
    head_N_L.insert("N".to_string(), json!(N));
    head_N_L.insert("L".to_string(), json!(L));
    let head_N_L: Value = serde_json::to_value(&head_N_L).expect("head JSON serialization error");
    assert_eq!(head_N_L, head_r);
    assert_eq!(data_ro, data_r);
}

fn perfect_measurement() {
    let L = 5;
    let mut x_error_ro = ZxError::new_L(L);
    let mut x_error = x_error_ro.view_mut();
    x_error[[1, 0]] = true;
    x_error[[3, 2]] = true;
    x_error[[3, 3]] = true;
    println!("x_error_ro:");
    x_error_ro.print();
    let zx_measurement= util::generate_perfect_measurements(&x_error_ro, &x_error_ro);
    println!("zx_measurement:");
    zx_measurement.print();
    // test rotation of measurement
    assert_eq!(zx_measurement.rotate_x2z().rotate_z2x(), zx_measurement);
    // test rotation of ZxError
    let rotated_clockwise = x_error_ro.rotate_x2z();
    println!("rotated_clockwise:");
    rotated_clockwise.print();
    let rotated_back = rotated_clockwise.rotate_z2x();
    assert_eq!(x_error_ro, rotated_back);
}

fn validate_correction() {
    let L = 5;
    let mut x_error_ro = ZxError::new_L(L);
    let mut x_error = x_error_ro.view_mut();
    x_error[[1, 0]] = true;
    x_error[[3, 2]] = true;
    x_error[[3, 3]] = true;
    println!("z_error_ro:");
    x_error_ro.print();
    // correction of the same as error must succeed
    let mut x_correction_ro = x_error_ro.clone();
    println!("x_correction_ro: (success)");
    x_correction_ro.print();
    assert_eq!(x_error_ro.validate_x_correction(&x_correction_ro), Ok(()));
    // if there is a -1 Z stabilizer, it fails
    let mut x_correction = x_correction_ro.view_mut();
    x_correction[[1, 0]] = false;  // does not correct because Z stabilizer at (1, 1) has -1 eigenstate
    println!("x_correction_ro: (Z stabilizer is at -1 eigenstate at (1,1))");
    x_correction_ro.print();
    assert_eq!(x_error_ro.validate_x_correction(&x_correction_ro), Err("Z stabilizer is at -1 eigenstate at (1,1)".to_string()));
    // if there is a logical operator, it also fails
    let mut x_correction = x_correction_ro.view_mut();
    for i in 1..L {
        x_correction[[1, i]] = true;
    }
    println!("x_correction_ro: (there is logical operator after correction)");
    x_correction_ro.print();
    assert_eq!(x_error_ro.validate_x_correction(&x_correction_ro), Err("there is X_L logical operator after correction".to_string()));
    
    // then try X stabilizers
    let z_error_ro = x_error_ro;  // use the same error pattern
    let mut z_correction_ro = z_error_ro.clone();
    assert_eq!(z_error_ro.validate_z_correction(&z_correction_ro), Ok(()));
    // // if there is a -1 X stabilizer, it fails
    let mut z_correction = z_correction_ro.view_mut();
    z_correction[[1, 0]] = false;  // does not correct because X stabilizer at (1, 0) has -1 eigenstate
    assert_eq!(z_error_ro.validate_z_correction(&z_correction_ro), Err("X stabilizer is at -1 eigenstate at (1,0)".to_string()));
    // if there is a logical operator, it also fails
    let mut z_correction = z_correction_ro.view_mut();
    for i in 0..L {
        z_correction[[i, 0]] = true;
    }
    z_correction[[1, 0]] = false;
    assert_eq!(z_error_ro.validate_z_correction(&z_correction_ro), Err("there is Z_L logical operator after correction".to_string()));
}

fn naive_correction() {
    let L = 5;
    let mut x_error_ro = ZxError::new_L(L);
    let mut x_error = x_error_ro.view_mut();
    x_error[[1, 0]] = true;
    x_error[[3, 2]] = true;
    x_error[[3, 3]] = true;
    println!("z_error_ro:");
    x_error_ro.print();
    let measurement = util::generate_perfect_measurements(&x_error_ro, &x_error_ro);
    println!("measurement:");
    measurement.print();
    let (x_correction, z_correction) = qec::naive_correction(&measurement);
    assert_eq!(x_error_ro.validate_x_correction(&x_correction), Ok(()));
    assert_eq!(x_error_ro.validate_z_correction(&z_correction), Ok(()));
}

fn maximum_max_weight_matching_correction() {
    Python::with_gil(|py| {
        (|py: Python| -> PyResult<()> {
            // prepare python library
            let networkx = py.import("networkx")?;
            let max_weight_matching = networkx.getattr("algorithms")?.getattr("matching")?.getattr("max_weight_matching")?;
            let maximum_max_weight_matching = |_node_num: usize, weighted_edges: Vec<(usize, usize, f64)>| -> std::collections::HashSet<(usize, usize)> {
                let G = networkx.call_method0("Graph").unwrap();
                let weighted_edges = weighted_edges.to_object(py);
                G.call_method1("add_weighted_edges_from", (weighted_edges,)).unwrap();
                let dict = vec![("maxcardinality", true)].into_py_dict(py);
                let matched: std::collections::HashSet<(usize, usize)> = max_weight_matching.call((G,), Some(dict)).unwrap().extract().unwrap();
                matched
            };
            // prepare error syndrome
            let L = 5;
            let mut x_error_ro = ZxError::new_L(L);
            let mut x_error = x_error_ro.view_mut();
            x_error[[1, 0]] = true;
            x_error[[3, 2]] = true;
            x_error[[3, 3]] = true;
            println!("z_error_ro:");
            x_error_ro.print();
            let measurement = util::generate_perfect_measurements(&x_error_ro, &x_error_ro);
            println!("measurement:");
            measurement.print();
            let (x_correction, z_correction) = qec::maximum_max_weight_matching_correction(&measurement, maximum_max_weight_matching);
            assert_eq!(x_error_ro.validate_x_correction(&x_correction), Ok(()));
            assert_eq!(x_error_ro.validate_z_correction(&z_correction), Ok(()));
            Ok(())
        })(py).map_err(|e| {
            e.print_and_set_sys_last_vars(py);
        })
    }).expect("python run failed");
}

fn archived_debug_tests() {
    {  // call python networkx.algorithms.matching.max_weight_matching
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
    }
    {  // test call c function
        println!("{}", blossom_v::safe_square(5));
        let input = vec![1., 2., 3., 4.];
        let output = blossom_v::safe_square_all(input);
        println!("{:?}", output);
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
    {
        let T = 4;
        let L = 4;
        let error_rate = 0.01;  // (1-3p)I + pX + pZ + pY
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(T, L);
        let nodes_count = model.count_nodes();
        assert_eq!(nodes_count, (6 * T + 1) * (2 * L - 1) * (2 * L - 1));
        // println!("{:?}", model);
        model.set_depolarizing_error(error_rate);
        let mut rng = thread_rng();
        let error_count = model.generate_random_errors(|| rng.gen::<f64>());
        println!("randomly generated error_count: {}/{}", error_count, nodes_count);
        {  // verify that any single error will only have at most error syndromes
            for t in 0..model.snapshot.len() {
                for i in 0..model.snapshot[t].len() {
                    for j in 0..model.snapshot[t][i].len() {
                        if model.snapshot[t][i][j].is_some() {
                            for error in [ftqec::ErrorType::X, ftqec::ErrorType::Z].iter() {
                                model.clear_error();
                                model.add_error_at(t, i, j, error);
                                assert_eq!(model.count_error(), 1);
                                model.propagate_error();
                                let mut measurement_error_count = 0;
                                model.iterate_measurement_errors(|_t, _i, _j, _node, _qubit_type| {
                                    measurement_error_count += 1;
                                });
                                assert!(measurement_error_count <= 2, "single qubit error should not cause more than 2 measurement errors");
                            }
                        }
                    }
                }
            }
            model.clear_error();
            println!("verified: any single qubit error only causes at most two measurement errors");
        }
        {  // build auxiliary information to assist decoding
            model.build_graph();
            let mut max_edge_count = 0;
            model.iterate_measurement_stabilizers(|_t, _i, _j, node, _qubit_type| {
                max_edge_count = std::cmp::max(max_edge_count, node.edges.len());
            });
            println!("maximum neighbor amount on a single stabilizer is {}", max_edge_count);
            assert!(max_edge_count <= 12, "verified: at most 12 neighbors in graph");
        }
    }
}
