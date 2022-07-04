use super::cfg_if;
use super::libc;
use libc::{c_int};
use std::collections::BTreeSet;


cfg_if::cfg_if! {
    if #[cfg(feature="blossom_v")] {

        #[link(name = "blossomV")]
        extern {
            fn minimum_weight_perfect_matching(node_num: c_int, edge_num: c_int, edges: *const c_int, weights: *const c_int, matched: *mut c_int);
        }

    } else {

        fn minimum_weight_perfect_matching(_node_num: c_int, _edge_num: c_int, _edges: *const c_int, _weights: *const c_int, _matched: *mut c_int) {
            unimplemented!("need blossom V library, see README.md")
        }

    }
}

pub fn safe_minimum_weight_perfect_matching_integer_weights(node_num: usize, input_weighted_edges: Vec<(usize, usize, c_int)>) -> Vec<usize> {
    // reverse the nodes' indices
    let weighted_edges = if cfg!(feature="MWPM_reverse_order") {
        input_weighted_edges.iter().map(|(a, b, cost)| {
            (node_num - 1 - a, node_num - 1 - b, *cost)
        }).collect()
    } else {
        input_weighted_edges
    };
    // normal matching
    let edge_num = weighted_edges.len();
    let mut edges = Vec::with_capacity(2 * edge_num);
    let mut weights = Vec::with_capacity(edge_num);
    debug_assert!({
        let mut existing_edges = BTreeSet::new();
        let mut sanity_check_passed = true;
        for idx in 0..edge_num {
            let (i, j, _weight) = weighted_edges[idx];
            if i == j {
                eprintln!("invalid edge between the same vertex {}", i);
                sanity_check_passed = false;
            }
            let left: usize = if i < j { i } else { j };
            let right: usize = if i < j { j } else { i };
            if existing_edges.contains(&(left, right)) {
                eprintln!("duplicate edge between the vertices {} and {}", i, j);
                sanity_check_passed = false;
            }
            existing_edges.insert((left, right));
        }
        sanity_check_passed
    });
    for idx in 0..edge_num {
        let (i, j, weight) = weighted_edges[idx];
        edges.push(i as c_int);
        edges.push(j as c_int);
        assert!(i < node_num && j < node_num);
        weights.push(weight);
    }
    let mut output = Vec::with_capacity(node_num);
    unsafe {
        minimum_weight_perfect_matching(node_num as c_int, edge_num as c_int, edges.as_ptr(), weights.as_ptr(), output.as_mut_ptr());
        output.set_len(node_num);
    }
    let output: Vec<usize> = output.iter().map(|x| *x as usize).collect();
    // recover the nodes' indices
    if cfg!(feature="MWPM_reverse_order") {
        let mut result = output.iter().map(|a| {
            node_num - 1 - a
        }).collect::<Vec<_>>();
        result.reverse();
        result
    } else {
        output
    }
}

pub fn safe_minimum_weight_perfect_matching(node_num: usize, input_weighted_edges: Vec<(usize, usize, f64)>) -> Vec<usize> {
    // scale all edges to integer values
    let mut maximum_weight = 0.;
    for (_, _, weight) in input_weighted_edges.iter() {
        if weight > &maximum_weight {
            maximum_weight = *weight;
        }
    }
    let scale: f64 = (c_int::MAX as f64) / 10. / ((node_num + 1) as f64) / maximum_weight;
    let mut integer_weighted_edges = Vec::<(usize, usize, c_int)>::with_capacity(input_weighted_edges.len());
    for (i, j, weight) in input_weighted_edges.into_iter() {
        integer_weighted_edges.push((i, j, (weight * scale).ceil() as c_int));
    }
    safe_minimum_weight_perfect_matching_integer_weights(node_num, integer_weighted_edges)
}

// important: only takes non-positive inputs
pub fn maximum_weight_perfect_matching_compatible(node_num: usize, weighted_edges: Vec<(usize, usize, f64)>) -> std::collections::HashSet<(usize, usize)> {
    // blossom V is minimum weight perfect matching, this function is maximum
    let weighted_edges: Vec::<(usize, usize, f64)> = weighted_edges.iter().map(|(a, b, w)| (*a, *b, -*w)).collect();
    let output = safe_minimum_weight_perfect_matching(node_num, weighted_edges);
    let mut matched = std::collections::HashSet::new();
    for i in 0..node_num {
        if output[i] as usize > i {
            matched.insert((i, output[i] as usize));
        }
    }
    matched
}

