#![allow(non_snake_case)]

use super::ndarray;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// Z or X error of L*L qubits => array[L][L]
pub struct ZxError(ndarray::Array2<bool>);
pub type ZxCorrection = ZxError;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// a batch of `ZxError`
pub struct BatchZxError(ndarray::Array3<bool>);

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// Z or X measurement of L*L qubits => array[L+1][L-1] (half of it has no information, only array[i][j] where i+j is odd has measurement result)
pub struct ZxMeasurement(ndarray::Array2<bool>);

impl Deref for ZxError {
    type Target = ndarray::Array2<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZxError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ZxError {
    pub fn new(array: ndarray::Array2<bool>) -> Self {
        let shape = array.shape();
        assert_eq!(shape[0], shape[1]);
        Self(array)
    }
    pub fn new_L(L: usize) -> Self {
        Self(ndarray::Array::from_elem((L, L), false))
    }
    pub fn L(&self) -> usize {
        self.shape()[0]
    }
    pub fn rotate_x2z(&self) -> Self {
        Self(rotate_array(self, true))
    }
    pub fn rotate_z2x(&self) -> Self {
        Self(rotate_array(self, false))
    }
    pub fn print(&self) {
        print_bool_matrix(self);
    }
    pub fn do_correction(&self, correction: &ZxCorrection) -> Self {
        let L = self.L();
        assert_eq!(L, correction.L());
        let mut ret_ro = Self::new_L(L);
        let mut ret = ret_ro.view_mut();
        for i in 0..L {
            for j in 0..L {
                ret[[i, j]] = self[[i, j]] ^ correction[[i, j]];
            }
        }
        ret_ro
    }
    pub fn validate_x_correction(&self, x_correction: &ZxCorrection) -> Result<(), String> {
        validate_x_correction(&self, x_correction)
    }
    pub fn validate_z_correction(&self, z_correction: &ZxCorrection) -> Result<(), String> {
        validate_z_correction(&self, z_correction)
    }
    pub fn if_all_z_stabilizers_plus1(&self) -> bool {
        if_all_z_stabilizers_plus1(&self).is_ok()
    }
    pub fn if_all_x_stabilizers_plus1(&self) -> bool {
        if_all_x_stabilizers_plus1(&self).is_ok()
    }
}

impl Deref for BatchZxError {
    type Target = ndarray::Array3<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BatchZxError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BatchZxError {
    pub fn new(array: ndarray::Array3<bool>) -> Self {
        let shape = array.shape();
        assert_eq!(shape[1], shape[2]);
        Self(array)
    }
    pub fn new_N_L(N: usize, L: usize) -> Self {
        Self(ndarray::Array::from_elem((N, L, L), false))
    }
    pub fn N(&self) -> usize {
        self.shape()[0]
    }
    pub fn L(&self) -> usize {
        self.shape()[1]
    }
}

impl Deref for ZxMeasurement {
    type Target = ndarray::Array2<bool>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZxMeasurement {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ZxMeasurement {
    // pub fn new(array: ndarray::Array2<bool>) -> Self {
    //     let shape = array.shape();
    //     assert_eq!(shape.len(), 2);
    //     assert_eq!(shape[0], shape[1]);
    //     assert_eq!(shape[0] >= 1, true);
    //     Self(array)
    // }
    pub fn new_L(L: usize) -> Self {
        Self(ndarray::Array::from_elem((L+1, L+1), false))
    }
    pub fn L(&self) -> usize {
        self.shape()[0] - 1  // because measurement is of size (L+1, L+1)
    }
    pub fn rotate_x2z(&self) -> Self {
        Self(rotate_array(self, true))
    }
    pub fn rotate_z2x(&self) -> Self {
        Self(rotate_array(self, false))
    }
    pub fn print(&self) {
        print_bool_matrix(self);
    }
}

pub fn rotate_array<T>(array: &ndarray::Array2<T>, clockwise: bool) -> ndarray::Array2<T> where T: Copy {
    let shape = array.shape();
    assert_eq!(shape[0], shape[1]);
    let L = shape[0];
    let mut rotated_ro = array.clone();
    let mut rotated = rotated_ro.view_mut();
    for i in 0..L {
        for j in 0..L {
            rotated[[i, j]] = if clockwise { array[[L-1-j, i]] } else { array[[j, L-1-i]] };
        }
    }
    rotated_ro
}

pub fn print_bool_matrix(array: &ndarray::Array2<bool>) {
    let shape = array.shape();
    let width = shape[0];
    let height = shape[1];
    print!("--");
    for i in 0..width {
        print!("{}", i % 10);
    }
    println!("-");
    for i in 0..height {
        print!("{}|", i % 10);
        for j in 0..width {
            print!("{}", if array[[i,j]] { "@" } else { " " });
        }
        println!("|");
    }
    for _ in 0..width+3 {
        print!("-");
    }
    println!("");
}

pub fn if_all_z_stabilizers_plus1(x_error: &ZxError) -> Result<(), String> {
    let L = x_error.L();
    for i in 0..L+1 {
        for j in 0..L+1 {
            if j != 0 && j != L && (i + j) % 2 == 0 {  // Z stabilizer only when i+j is even
                // XOR a(i-1,j-1), b(i-1,j), c(i,j-1), d(i,j) if exist
                let i_minus_exists = i > 0;
                let i_exists = i < L;
                let mut result = false;
                if i_minus_exists {
                    result ^= x_error[[i-1, j-1]] ^ x_error[[i-1, j]];
                }
                if i_exists {
                    result ^= x_error[[i, j-1]] ^ x_error[[i, j]];
                }
                if result {
                    return Err(format!("Z stabilizer is at -1 eigenstate at ({},{})", i, j).to_string())
                }
            }
        }
    }
    Ok(())
}

/// validate the correction from Z stabilizers, which correct X errors.
/// return `true` if the correction is successful, `false` if some of the stabilizers are not back to -1 eigenstate or it introduces X_L operator
pub fn validate_x_correction(x_error: &ZxError, x_correction: &ZxCorrection) -> Result<(), String> {
    let combined = x_error.do_correction(x_correction);
    if_all_z_stabilizers_plus1(&combined)?;
    // count the errors on the left side
    let mut count_left = 0;
    for i in 0..combined.L() {
        if combined[[i, 0]] {
            count_left += 1;
        }
    }
    if count_left % 2 == 1 {  // correction is successful only if count_left is even number
        return Err("there is X_L logical operator after correction".to_string())
    }
    Ok(())
}

pub fn if_all_x_stabilizers_plus1(z_error: &ZxError) -> Result<(), String> {
    let L = z_error.L();
    for i in 0..L+1 {
        for j in 0..L+1 {
            if i != 0 && i != L && (i + j) % 2 == 1 {  // X stabilizer only when i+j is odd
                // XOR a(i-1,j-1), b(i-1,j), c(i,j-1), d(i,j) if exist
                let j_minus_exists = j > 0;
                let j_exists = j < L;
                let mut result = false;
                if j_minus_exists {
                    result ^= z_error[[i-1, j-1]] ^ z_error[[i, j-1]];
                }
                if j_exists {
                    result ^= z_error[[i-1, j]] ^ z_error[[i, j]];
                }
                if result {
                    return Err(format!("X stabilizer is at -1 eigenstate at ({},{})", i, j).to_string())
                }
            }
        }
    }
    Ok(())
}

/// validate the correction from X stabilizers, which correct Z errors.
/// return `true` if the correction is successful, `false` if some of the stabilizers are not back to -1 eigenstate or it introduces X_L operator
pub fn validate_z_correction(z_error: &ZxError, z_correction: &ZxCorrection) -> Result<(), String> {
    let combined = z_error.do_correction(z_correction);
    if_all_x_stabilizers_plus1(&combined)?;
    // count the errors on the top side
    let mut count_top = 0;
    for i in 0..combined.L() {
        if combined[[0, i]] {
            count_top += 1;
        }
    }
    if count_top % 2 == 1 {  // correction is successful only if count_top is even number
        return Err("there is Z_L logical operator after correction".to_string())
    }
    Ok(())
}
