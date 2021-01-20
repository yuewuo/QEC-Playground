#![allow(non_snake_case)]

use super::clap;
use super::util;
use super::rand::prelude::*;
use super::serde_json;
use super::serde_json::{Value, Map, json};
use super::types::*;
use super::qec;
use super::pyo3::prelude::*;
use super::pyo3::types::{IntoPyDict};
use super::blossom_v;
use super::ftqec;
use super::types::QubitType;
use super::types::ErrorType;
use super::offer_decoder;

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
        ("offer_decoder_study", Some(matches)) => {
            let d = value_t!(matches, "d", usize).expect("required");
            let p = value_t!(matches, "p", f64).expect("required");
            let count = value_t!(matches, "count", usize).unwrap_or(1);
            let max_resend = value_t!(matches, "max_resend", usize).unwrap_or(usize::MAX);
            let max_cycles = value_t!(matches, "max_cycles", usize).unwrap_or(usize::MAX);
            let print_error_pattern_to_find_infinite_loop = matches.is_present("print_error_pattern_to_find_infinite_loop");
            offer_decoder_study(d, p, count, max_resend, max_cycles, print_error_pattern_to_find_infinite_loop);
        }
        ("all", Some(_)) => {  // remember to add new test functions here
            save_load();
            perfect_measurement();
            validate_correction();
            naive_correction();
            maximum_max_weight_matching_correction();
            debug_tests();
            archived_debug_tests();
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

fn offer_decoder_study(d: usize, p: f64, count: usize, max_resend: usize, max_cycles: usize, print_error_pattern_to_find_infinite_loop: bool) {
    let mut cases = 0;
    let mut rng = thread_rng();
    // create offer decoder instance
    let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(d);
    // create MWPM decoder instance
    let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, d);  // single layer planar code
    model.set_depolarizing_error_with_perfect_initialization(p);
    model.iterate_snapshot_mut(|t, _i, _j, node| {  // the same error model as in `decoder`
        if t == 6 && node.qubit_type == QubitType::Data {
            node.error_rate_x = p;
            node.error_rate_z = p;
            node.error_rate_y = p;
        }
    });
    model.build_graph();
    model.optimize_correction_pattern();
    model.build_exhausted_path_autotune();
    while cases < count {
        decoder.reinitialize();
        let error_count = decoder.generate_only_x_random_errors(p, || rng.gen::<f64>());
        if error_count == 0 {
            continue
        }
        decoder.error_changed();
        if print_error_pattern_to_find_infinite_loop {
            println!("{:?}", decoder.error_pattern());  // to find infinite looping case
        }
        let cycles = decoder.pseudo_parallel_execute_to_stable_with_max_resend_max_cycles(max_resend, max_cycles);
        match cycles {
            Ok(cycles) => {
                if decoder.has_logical_error(ErrorType::X) == true {
                    // judge if MWPM generates no logical error, if so, output the case
                    let mwpm_has_logical_error = {
                        // copy the error pattern into MWPM decoder
                        model.iterate_snapshot_mut(|t, i, j, node| {
                            if t == 6 && node.qubit_type == QubitType::Data {
                                node.error = decoder.qubits[i][j].error.clone();
                            }
                        });
                        model.propagate_error();
                        let measurement = model.generate_measurement();
                        let correction = model.decode_MWPM(&measurement);
                        let validation_ret = model.validate_correction_on_boundary(&correction);
                        validation_ret.is_err()
                    };
                    if !mwpm_has_logical_error {  // output the case
                        println!("[offer decoder fails but MWPM decoder succeeds]");
                        println!("{}", json!({
                            "cycles": cycles,
                            "error": decoder.error_pattern(),
                        }).to_string());
                        cases += 1;
                    }
                }
            },
            Err(cycles) => {
                println!("[exceed max resend]");
                println!("{}", json!({
                    "cycles": cycles,
                    "error": decoder.error_pattern(),
                }).to_string());
                cases += 1;
            }
        }
    }
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
    {
        let T = 4;
        let L = 4;
        let error_rate = 0.01;  // (1-3p)I + pX + pZ + pY
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(T, L);
        model.set_depolarizing_error(error_rate);
        model.build_graph();
        model.optimize_correction_pattern();
        model.build_exhausted_path_equally_weighted();
        // println!("exhausted of Z stabilizer at [6][0][1]: {:?}", model.snapshot[6][0][1].as_ref().expect("exist").exhausted_map);
    }
    {
        let MeasurementRounds = 4;
        let L = 4;
        let error_rate = 0.01;  // (1-3p)I + pX + pZ + pY
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(MeasurementRounds, L);
        let nodes_count = model.count_nodes();
        let T = model.T;
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
                            for error in [ErrorType::X, ErrorType::Z].iter() {
                                model.clear_error();
                                model.add_error_at(t, i, j, error);
                                assert_eq!(model.count_error(), 1);
                                model.propagate_error();
                                let mut measurement_error_count = 0;
                                model.iterate_measurement_errors(|_t, _i, _j, _node| {
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
            model.optimize_correction_pattern();
            let mut max_edge_count = 0;
            model.iterate_measurement_stabilizers(|_t, _i, _j, node| {
                max_edge_count = std::cmp::max(max_edge_count, node.edges.len());
            });
            println!("maximum neighbor amount on a single stabilizer is {}", max_edge_count);
            assert!(max_edge_count <= 12, "verified: at most 12 neighbors in graph");
            // build exhausted path helps to speed up decoder
            model.build_exhausted_path_autotune();
            // println!("exhausted of Z stabilizer at [12][0][1]: {:?}", model.snapshot[12][0][1].as_ref().expect("exist").exhausted_map);
            // println!("{:?}", model.get_correction_two_nodes(ftqec::Index::new(12, 0, 1), ftqec::Index::new(24, 4, 1)));
            let _correction = model.get_correction_two_nodes(&ftqec::Index::new(12, 0, 1), &ftqec::Index::new(12, 4, 1));
            // println!("{:?}", _correction);
        }
        {  // decode the generated error
            /*
             * add error at Index { t: 4, i: 2, j: 6 } Y
             * add error at Index { t: 20, i: 1, j: 5 } X
             * add error at Index { t: 23, i: 6, j: 4 } Y
             */
            model.clear_error();
            model.add_error_at(4, 2, 6, &ErrorType::Y);
            model.add_error_at(20, 1, 5, &ErrorType::X);
            model.add_error_at(23, 6, 4, &ErrorType::Y);
            // {  // generate random error and hard-code it just like above
            //     model.generate_random_errors(|| rng.gen::<f64>());
            //     model.iterate_snapshot(|t, i, j, node| {
            //         if node.error != ErrorType::I {
            //             println!("add error at {:?} {:?}", ftqec::Index::new(t, i, j), node.error);
            //         }
            //     });
            // }
            model.propagate_error();
            let measurement = model.generate_measurement();
            // println!("{:?}", measurement);
            // actually one can use another model to decode, if you're not comfortable with passing all internal error information into decoder
            let correction = model.decode_MWPM(&measurement);
            // println!("{:?}", correction);
            let mut corrected = model.get_data_qubit_error_pattern();
            // println!("error pattern: {:?}", corrected);
            corrected.combine(&correction);  // apply correction to error pattern
            // println!("corrected: {:?}", corrected);
            println!("validate bottom layer: {:?}", model.validate_correction_on_bottom_layer(&correction));
            println!("validate top layer: {:?}", model.validate_correction_on_top_layer(&correction));
            println!("validate all layers: {:?}", model.validate_correction_on_all_layers(&correction));
        }
    }
    {  // find one example for each 12 boundaries
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(3, 4);
        let very_small_error_rate = 0.0001;
        model.set_depolarizing_error(very_small_error_rate);
        model.build_graph();
        model.optimize_correction_pattern();
        model.build_exhausted_path_autotune();
        let error_source = ftqec::Index::new(18, 2, 3);
        let error_target = vec![
            (ftqec::Index::new(18, 2, 1), "left"),
            (ftqec::Index::new(18, 2, 5), "right"),
            (ftqec::Index::new(18, 0, 3), "front"),
            (ftqec::Index::new(18, 4, 3), "back"),
            (ftqec::Index::new(12, 2, 3), "bottom"),
            (ftqec::Index::new(24, 2, 3), "top"),
            (ftqec::Index::new(12, 0, 3), "bottom front"),
            (ftqec::Index::new(12, 2, 1), "bottom left"),
            (ftqec::Index::new(12, 0, 5), "bottom front right"),
            (ftqec::Index::new(24, 4, 3), "top back"),
            (ftqec::Index::new(24, 2, 5), "top right"),
            (ftqec::Index::new(24, 4, 1), "top back left"),
        ];
        for (target, name) in error_target.iter() {
            let mut found_error = None;
            let mut propagated_to = Vec::new();
            for t in 0..model.snapshot.len() {
                for i in 0..model.snapshot[t].len() {
                    for j in 0..model.snapshot[t][i].len() {
                        if model.snapshot[t][i][j].is_some() {
                            for error in [ErrorType::X, ErrorType::Z].iter() {
                                model.clear_error();
                                model.add_error_at(t, i, j, error);
                                model.propagate_error();
                                let mut measurement_errors = Vec::new();
                                model.iterate_measurement_errors(|t, i, j, _node| {
                                    measurement_errors.push(ftqec::Index::new(t, i, j));
                                });
                                assert!(measurement_errors.len() <= 2, "single qubit error should not cause more than 2 measurement errors");
                                if measurement_errors.len() == 2 {
                                    let matched = (error_source == measurement_errors[0] && *target == measurement_errors[1]) ||
                                        (error_source == measurement_errors[1] && *target == measurement_errors[0]);
                                    if matched {
                                        let mut this_propagated_to = Vec::new();
                                        let width = 2 * model.L - 1;
                                        let mut has_error = ndarray::Array::from_elem((width, width), false);
                                        let mut has_error_mut = has_error.view_mut();
                                        model.iterate_snapshot(|_t, i, j, node| {
                                            if node.propagated != ErrorType::I {
                                                has_error_mut[[i, j]] = true;
                                            }
                                        });
                                        for i in 0..width {
                                            for j in 0..width {
                                                if has_error[[i, j]] {
                                                    this_propagated_to.push(vec![i, j]);
                                                }
                                            }
                                        }
                                        // optimize for propagating to less qubits (for the ease of drawing figure)
                                        if found_error.is_none() || this_propagated_to.len() < propagated_to.len() {
                                            found_error = Some(ftqec::Index::new(t, i, j));
                                            propagated_to = this_propagated_to;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // println!("target {:?}: single error at {:?}", *target, found_error);
            let found_error = found_error.unwrap();
            let cost = model.snapshot[error_source.t][error_source.i][error_source.j].as_ref().unwrap().exhausted_map[target].cost;
            let probability = (-cost).exp();
            let case_count = (probability / very_small_error_rate).round() as usize;
            println!("[[{}, {}, {}], [{}, {}, {}], {}, {}, {:?}],  // {}", target.t, target.i, target.j, found_error.t, found_error.i, found_error.j,
                probability, case_count, propagated_to, name);
        }
    }
    {  // test functionality after adding the perfect measurement layer on top
        let MeasurementRounds = 2;
        let L = 4;
        let error_rate = 0.01;  // (1-3p)I + pX + pZ + pY
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(MeasurementRounds, L);
        model.set_depolarizing_error(error_rate);
        model.build_graph();
        model.optimize_correction_pattern();
        model.build_exhausted_path_equally_weighted();
        println!("model.snapshot.len(): {}", model.snapshot.len());
        let default_correction = model.generate_default_correction();
        println!("default_correction.x.shape(): {:?}", default_correction.x.shape());
        let measurement = model.generate_measurement();
        println!("measurement.shape(): {:?}", measurement.shape());
        // println!("exhausted of Z stabilizer at [6][0][1]: {:?}", model.snapshot[6][0][1].as_ref().expect("exist").exhausted_map);
    }
    {  // test sparse correction functionality
        let mut correction = ftqec::Correction::new_all_false(3, 3, 3);
        let mut x_mut = correction.x.view_mut();
        let mut z_mut = correction.z.view_mut();
        x_mut[[0, 1, 1]] = true;
        x_mut[[2, 0, 2]] = true;
        z_mut[[0, 0, 1]] = true;
        z_mut[[1, 0, 1]] = true;
        z_mut[[2, 0, 1]] = true;
        println!("correction: {:?}", correction);
        let sparse_correction = ftqec::SparseCorrection::from(&correction);
        println!("sparse_correction: {:?}", sparse_correction);
        let back_correction = ftqec::Correction::from(&sparse_correction);
        assert_eq!(back_correction, correction, "they should be the same");
    }
    {  // test only perfect measurement
        let mut model = ftqec::PlanarCodeModel::new_standard_planar_code(0, 3);
        let p = 0.1;
        model.set_depolarizing_error_with_perfect_initialization(p);
        model.iterate_snapshot_mut(|t, _i, _j, node| {
            if t == 6 && node.qubit_type == QubitType::Data {
                println!("set error rate at {} {} {}", t, _i, _j);
                node.error_rate_x = p;
                node.error_rate_z = p;
                node.error_rate_y = p;
            }
        });
        model.build_graph();
        model.optimize_correction_pattern();
        model.build_exhausted_path_autotune();
        {  // add errors
            // {  // no logical error
            //     model.snapshot[6][0][0].as_mut().unwrap().error = ErrorType::X;
            // }
            // {  // has logical error
            //     model.snapshot[6][0][0].as_mut().unwrap().error = ErrorType::X;
            //     model.snapshot[6][0][2].as_mut().unwrap().error = ErrorType::X;
            // }
            {  // random error
                let mut rng = thread_rng();
                model.generate_random_errors(|| rng.gen::<f64>());
            }
        }
        model.propagate_error();
        let measurement = model.generate_measurement();
        println!("{:?}", measurement);
        let correction = model.decode_MWPM(&measurement);
        println!("correction: {:?}", correction);
        let mut corrected = model.get_data_qubit_error_pattern();
        println!("error pattern: {:?}", corrected);
        corrected.combine(&correction);  // apply correction to error pattern
        println!("corrected: {:?}", corrected);
        let validation_ret = model.validate_correction_on_boundary(&correction);
        println!("{:?}", validation_ret);
    }
    {  // test offer decoder
        let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(7);
        decoder.reinitialize();
        {  // test augmenting path, in this case, qubits[4][3] should establish a augmenting path to [8][11] so that cost is minimized
            decoder.qubits[6][4].error = ErrorType::X;
            decoder.qubits[8][8].error = ErrorType::X;
            decoder.qubits[8][12].error = ErrorType::X;
            decoder.force_match_qubits(6, 5, 8, 7);
            decoder.force_match_qubits(8, 9, 8, 11);
        }
        decoder.error_changed();
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        assert_eq!(false, decoder.has_logical_error(ErrorType::X));
        decoder.reinitialize();
        {  // has x logical error
            decoder.qubits[0][2].error = ErrorType::X;
            decoder.qubits[0][4].error = ErrorType::X;
            decoder.qubits[0][6].error = ErrorType::X;
            decoder.qubits[0][8].error = ErrorType::X;
            decoder.qubits[0][10].error = ErrorType::X;
        }
        decoder.error_changed();
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        assert_eq!(true, decoder.has_logical_error(ErrorType::X));
    }
    {  // test offer decoder error case
        let d = 5;
        let mut decoder = offer_decoder::create_standard_planar_code_offer_decoder(d);
        decoder.reinitialize();
        decoder.qubits[1][3].error = ErrorType::X;
        decoder.qubits[3][1].error = ErrorType::X;
        decoder.error_changed();
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        assert_eq!(false, decoder.has_logical_error(ErrorType::X));
    }
    {  // test offer decoder error case
        let error_pattern_origin = ["IIIIIIIII","IIIIIIIXI","IIIIIIIII","IIIIIXIII","XIIIIIIII","IIIIIIIII","IIIIIIIII","IIIIIIIII","IIIIIIIII"];
        let error_pattern: Vec<String> = error_pattern_origin.iter().map(|e| e.to_string()).collect();
        let mut decoder = offer_decoder::OfferDecoder::create_with_error_pattern(&error_pattern);
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        assert_eq!(false, decoder.has_logical_error(ErrorType::X));
    }
    {  // dead lock cases:
        let error_pattern_origin = ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIXIIIIIII","IIXIIIIIXIIIIIIII","IIIIIIIIIXIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII"];
        let error_pattern: Vec<String> = error_pattern_origin.iter().map(|e| e.to_string()).collect();
        let mut decoder = offer_decoder::OfferDecoder::create_with_error_pattern(&error_pattern);
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        println!("has logical error: {}", decoder.has_logical_error(ErrorType::X));
        // similar cases:
        // ["IIXIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIXIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIXI","IIIIIIIIIIIIXIIII","XIIIIIIIIIIIIIIII","IIIIXIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIXIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
        // ["IIXIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIXIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIXI","IIIIIIIIIIIIXIIII","XIIIIIIIIIIIIIIII","IIIIXIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIXIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIXIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIXIIIIIIIIII","IIIIIIIIIXIIIIIXI","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIXI","IIIIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIXIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIXIIIIIII","IIXIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIXIIIIII","IIIIIIIIIIIIXIIII","IIIXIXIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIXII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIXIIIIIIIIIIIIII","IIIIIIIXIXIIIIIII","IIIIIIIIIIIIIIXII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIXIIIIIIIIIIIII","IIIXIXIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIXIIIIIIIXIIIX","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIXX","IXIIIIIIIIIIIXIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIXIIIIIIIXIII","IIIIIIIIXXIIIIXII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIXI"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIXIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIXIIIIIIIIIII","IIXIIIIIIIIIIIIII","IIIIIIIXIIIIIIIII","IIIIIIIIIIIIXIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIXII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
        // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIXIXIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IXIIIIIIIIIIIIIII","IIXIIIIIIIXIIIIII"]
    }
    {  // augmenting loop may cause infinite loop
        let error_pattern_origin = ["XIIIIIIIIIXIIIIII", "IIIIIIIIIIXIIIIII", "IIIIIIIIIIIIIIIXI", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIXIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIXIIIIIII", "IIIIIIIIIIIIXIIII", "IIIIIXIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII", "IIIIIIIIIIIIIIIII"];
        let error_pattern: Vec<String> = error_pattern_origin.iter().map(|e| e.to_string()).collect();
        let mut decoder = offer_decoder::OfferDecoder::create_with_error_pattern(&error_pattern);
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        println!("has logical error: {}", decoder.has_logical_error(ErrorType::X));
    }
}

fn debug_tests() {
    // {  // augmenting loop will degrade performance
    //     let error_pattern_origin = ["IXIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","XIXIIIIIIIIIXIIIX","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIXIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"];
    //     let error_pattern: Vec<String> = error_pattern_origin.iter().map(|e| e.to_string()).collect();
    //     let mut decoder = offer_decoder::OfferDecoder::create_with_error_pattern(&error_pattern);
    //     let cycles = decoder.pseudo_parallel_execute_to_stable();
    //     let match_pattern = decoder.match_pattern();
    //     println!("match_pattern: {:?}", match_pattern);
    //     println!("cycles: {}", cycles);
    //     println!("has logical error: {}", decoder.has_logical_error(ErrorType::X));
    //     // similar cases:
    //     // ["IIXIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIXIII","IIIIIIIIIXIIIIIII","XIIIIIXIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIXIIIXII","IIIIIIIIXIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
    //     // ["IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IXIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIIX","IIIIIIIIIXIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIXIIIIIIII","XIIIIIIIIIXIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIXIIIIIII","IIIIIIIIIIIIIIIII","IIIIIIIIIIIIIIIII"]
    // }
    { // debug infinite loop
        let error_pattern_origin = ["IIIIIIIII", "IIIXIIIII", "IIIIIIXII", "IIIIIIIII", "IIIIIIIII", "IIIIIXIII", "IIIIIIIII", "IIIXIIIII", "IIIIIIIII"];
        let error_pattern: Vec<String> = error_pattern_origin.iter().map(|e| e.to_string()).collect();
        let mut decoder = offer_decoder::OfferDecoder::create_with_error_pattern(&error_pattern);
        let cycles = decoder.pseudo_parallel_execute_to_stable();
        let match_pattern = decoder.match_pattern();
        println!("match_pattern: {:?}", match_pattern);
        println!("cycles: {}", cycles);
        println!("has logical error: {}", decoder.has_logical_error(ErrorType::X));
    }
}
