use super::ndarray;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
#[derive(PartialEq)]
/// Z or X error of L*L qubits => array[L][L]
pub struct ZxError(ndarray::Array2<bool>);

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

#[allow(non_snake_case)]
impl ZxError {
    pub fn new(array: ndarray::Array2<bool>) -> Self {
        let shape = array.shape();
        assert_eq!(shape.len(), 2);
        assert_eq!(shape[0], shape[1]);
        Self(array)
    }
    pub fn L(&self) -> usize {
        self.shape()[0]
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

#[allow(non_snake_case)]
impl BatchZxError {
    pub fn new(array: ndarray::Array3<bool>) -> Self {
        let shape = array.shape();
        assert_eq!(shape.len(), 3);
        assert_eq!(shape[1], shape[2]);
        Self(array)
    }
    pub fn new_N_L(N: usize, L: usize) -> Self {
        Self(ndarray::Array::from_shape_fn((N, L, L), |_| false))
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
        assert_eq!(shape[0] - 2, shape[1]);
        Self(array)
    }
}
