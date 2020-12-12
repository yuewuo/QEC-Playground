use super::libc;
use libc::{c_ulonglong, c_double, c_int};

#[link(name = "test")]
extern {
    fn square(value: c_ulonglong) -> c_ulonglong;
    fn square_all(length: c_ulonglong, input: *const c_double, output: *mut c_double);
}

pub fn safe_square(value: u64) -> u64 {
    unsafe { square(value) as u64 }
}

pub fn safe_square_all(input: Vec<f64>) -> Vec<f64> {
    let length = input.len();
    let mut output = Vec::with_capacity(length);
    unsafe {
        square_all(length as u64, input.as_ptr(), output.as_mut_ptr());
        output.set_len(length);
    }
    output
}

#[link(name = "blossomV")]
extern {
    fn minimum_weight_perfect_matching(node_num: c_int, edge_num: c_int, edges: *const c_int, weights: *const c_double, matched: *mut c_int);
}

pub fn maximum_weight_perfect_matching_compatible(node_num: usize, weighted_edges: Vec<(usize, usize, f64)>) -> std::collections::HashSet<(usize, usize)> {
    let edge_num = weighted_edges.len();
    let mut edges = Vec::with_capacity(2 * edge_num);
    let mut weights = Vec::with_capacity(edge_num);
    for i in 0..edge_num {
        let (i, j, weight) = weighted_edges[i];
        edges.push(i as c_int);
        edges.push(j as c_int);
        assert!(i < node_num && j < node_num);
        weights.push(- weight);  // blossom V is minimum weight perfect matching, this function is maximum
    }
    let mut output = Vec::with_capacity(node_num);
    unsafe {
        minimum_weight_perfect_matching(node_num as c_int, edge_num as c_int, edges.as_ptr(), weights.as_ptr(), output.as_mut_ptr());
        output.set_len(node_num);
    }
    let mut matched = std::collections::HashSet::new();
    for i in 0..node_num {
        if output[i] as usize > i {
            matched.insert((i, output[i] as usize));
        }
    }
    matched
}
