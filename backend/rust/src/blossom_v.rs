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
