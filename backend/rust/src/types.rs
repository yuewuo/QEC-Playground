#![allow(non_snake_case)]

use super::ndarray;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
#[derive(PartialEq)]
/// Z or X error of L*L qubits => array[L][L]
pub struct ZxError(ndarray::Array2<bool>);
// pub type ZxCorrection = ZxError;

#[derive(Debug)]
#[derive(PartialEq)]
/// a batch of `ZxError`
pub struct BatchZxError(ndarray::Array3<bool>);

#[derive(Debug)]
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
        assert_eq!(shape.len(), 2);
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
        assert_eq!(shape.len(), 3);
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
    pub fn new(array: ndarray::Array2<bool>) -> Self {
        let shape = array.shape();
        assert_eq!(shape.len(), 2);
        assert_eq!(shape[0], shape[1]);
        Self(array)
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
