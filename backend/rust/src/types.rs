#![allow(non_snake_case)]

use super::ndarray;
use std::ops::{Deref, DerefMut};
use serde::Serialize;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// Z or X error of L*L qubits => array\[L\]\[L\]
pub struct ZxError(ndarray::Array2<bool>);
pub type ZxCorrection = ZxError;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// a batch of `ZxError`
pub struct BatchZxError(ndarray::Array3<bool>);

#[derive(Debug, Clone)]
#[derive(PartialEq)]
/// Z or X measurement of L*L qubits => array\[L+1\]\[L-1\] (half of it has no information, only array\[i\]\[j\] where i+j is odd has measurement result)
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
    #[allow(dead_code)]  // feature=noserver, avoid warning messages
    pub fn if_all_z_stabilizers_plus1(&self) -> bool {
        if_all_z_stabilizers_plus1(&self).is_ok()
    }
    #[allow(dead_code)]  // feature=noserver, avoid warning messages
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

/// Qubit type, corresponds to `QTYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum QubitType {
    Data,
    StabX,
    StabZ,
    StabXZZXLogicalX,
    StabXZZXLogicalZ,
}

impl QubitType {
    /// if measure in Z basis, it's prepared in |0> state, otherwise it's measuring X basis and prepared in |+> state; data qubit will return None
    pub fn is_measured_in_z_basis(&self) -> Option<bool> {
        match self {
            Self::Data => None,
            Self::StabZ => Some(true),
            Self::StabX | Self::StabXZZXLogicalX | Self::StabXZZXLogicalZ => Some(false),
        }
    }
}

/// Error type, corresponds to `ETYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone)]
pub enum ErrorType {
    I,
    X,
    Z,
    Y,
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::I => "I",
            Self::X => "X",
            Self::Z => "Z",
            Self::Y => "Y",
        })
    }
}

impl serde::Serialize for ErrorType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(format!("{}", self).as_str())
    }
}

impl ErrorType {
    pub fn multiply(&self, err: &Self) -> Self {
        match (self, err) {
            (Self::I, Self::I) => Self::I,
            (Self::I, Self::X) => Self::X,
            (Self::I, Self::Z) => Self::Z,
            (Self::I, Self::Y) => Self::Y,
            (Self::X, Self::I) => Self::X,
            (Self::X, Self::X) => Self::I,
            (Self::X, Self::Z) => Self::Y,
            (Self::X, Self::Y) => Self::Z,
            (Self::Z, Self::I) => Self::Z,
            (Self::Z, Self::X) => Self::Y,
            (Self::Z, Self::Z) => Self::I,
            (Self::Z, Self::Y) => Self::X,
            (Self::Y, Self::I) => Self::Y,
            (Self::Y, Self::X) => Self::Z,
            (Self::Y, Self::Z) => Self::X,
            (Self::Y, Self::Y) => Self::I,
        }
    }
    pub fn all_possible_errors() -> Vec::<Self> {
        vec![Self::X, Self::Z, Self::Y]
    }
    pub fn combine_probability(p_xyz_1: (f64, f64, f64), p_xyz_2: (f64, f64, f64)) -> (f64, f64, f64) {
        let (px1, py1, pz1) = p_xyz_1;
        let (px2, py2, pz2) = p_xyz_2;
        let pi1 = 1. - px1 - py1 - pz1;
        let pi2 = 1. - px2 - py2 - pz2;
        let px_combined = px1 * pi2 + py1 * pz2 + pz1 * py2 + pi1 * px2;
        let py_combined = py1 * pi2 + px1 * pz2 + pz1 * px2 + pi1 * py2;
        let pz_combined = pz1 * pi2 + px1 * py2 + py1 * px2 + pi1 * pz2;
        (px_combined, py_combined, pz_combined)
    }
}

/// Correlated error type for two qubit errors
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum CorrelatedErrorType {
    II,
    IX,
    IZ,
    IY,
    XI,
    XX,
    XZ,
    XY,
    ZI,
    ZX,
    ZZ,
    ZY,
    YI,
    YX,
    YZ,
    YY,
}

impl CorrelatedErrorType {
    pub fn my_error(&self) -> ErrorType {
        match self {
            Self::II | Self::IX | Self::IZ | Self::IY => ErrorType::I,
            Self::XI | Self::XX | Self::XZ | Self::XY => ErrorType::X,
            Self::ZI | Self::ZX | Self::ZZ | Self::ZY => ErrorType::Z,
            Self::YI | Self::YX | Self::YZ | Self::YY => ErrorType::Y,
        }
    }
    pub fn peer_error(&self) -> ErrorType {
        match self {
            Self::II | Self::XI | Self::ZI | Self::YI => ErrorType::I,
            Self::IX | Self::XX | Self::ZX | Self::YX => ErrorType::X,
            Self::IZ | Self::XZ | Self::ZZ | Self::YZ => ErrorType::Z,
            Self::IY | Self::XY | Self::ZY | Self::YY => ErrorType::Y,
        }
    }
    pub fn all_possible_errors() -> Vec::<Self> {
        vec![           Self::IX, Self::IZ, Self::IY, Self::XI, Self::XX, Self::XZ, Self::XY,
             Self::ZI, Self::ZX, Self::ZZ, Self::ZY, Self::YI, Self::YX, Self::YZ, Self::YY,]
    }
}

impl std::fmt::Display for CorrelatedErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::II => "II", Self::IX => "IX", Self::IZ => "IZ", Self::IY => "IY",
            Self::XI => "XI", Self::XX => "XX", Self::XZ => "XZ", Self::XY => "XY",
            Self::ZI => "ZI", Self::ZX => "ZX", Self::ZZ => "ZZ", Self::ZY => "ZY",
            Self::YI => "YI", Self::YX => "YX", Self::YZ => "YZ", Self::YY => "YY",
        }.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct CorrelatedErrorModel {
    pub error_rate_IX: f64,
    pub error_rate_IZ: f64,
    pub error_rate_IY: f64,
    pub error_rate_XI: f64,
    pub error_rate_XX: f64,
    pub error_rate_XZ: f64,
    pub error_rate_XY: f64,
    pub error_rate_ZI: f64,
    pub error_rate_ZX: f64,
    pub error_rate_ZZ: f64,
    pub error_rate_ZY: f64,
    pub error_rate_YI: f64,
    pub error_rate_YX: f64,
    pub error_rate_YZ: f64,
    pub error_rate_YY: f64,
}

impl CorrelatedErrorModel {
    // pub fn default() -> Self {
    //     Self::default_with_probability(0.)
    // }
    pub fn default_with_probability(p: f64) -> Self {
        Self {
            error_rate_IX: p,
            error_rate_IZ: p,
            error_rate_IY: p,
            error_rate_XI: p,
            error_rate_XX: p,
            error_rate_XZ: p,
            error_rate_XY: p,
            error_rate_ZI: p,
            error_rate_ZX: p,
            error_rate_ZZ: p,
            error_rate_ZY: p,
            error_rate_YI: p,
            error_rate_YX: p,
            error_rate_YZ: p,
            error_rate_YY: p,
        }
    }
    pub fn error_probability(&self) -> f64 {
                               self.error_rate_IX + self.error_rate_IZ + self.error_rate_IY
        + self.error_rate_XI + self.error_rate_XX + self.error_rate_XZ + self.error_rate_XY
        + self.error_rate_ZI + self.error_rate_ZX + self.error_rate_ZZ + self.error_rate_ZY
        + self.error_rate_YI + self.error_rate_YX + self.error_rate_YZ + self.error_rate_YY
    }
    pub fn no_error_probability(&self) -> f64 {
        1. - self.error_probability()
    }
    pub fn error_rate(&self, error_type: &CorrelatedErrorType) -> f64 {
        match error_type {
            CorrelatedErrorType::II => self.no_error_probability(),
            CorrelatedErrorType::IX => self.error_rate_IX,
            CorrelatedErrorType::IZ => self.error_rate_IZ,
            CorrelatedErrorType::IY => self.error_rate_IY,
            CorrelatedErrorType::XI => self.error_rate_XI,
            CorrelatedErrorType::XX => self.error_rate_XX,
            CorrelatedErrorType::XZ => self.error_rate_XZ,
            CorrelatedErrorType::XY => self.error_rate_XY,
            CorrelatedErrorType::ZI => self.error_rate_ZI,
            CorrelatedErrorType::ZX => self.error_rate_ZX,
            CorrelatedErrorType::ZZ => self.error_rate_ZZ,
            CorrelatedErrorType::ZY => self.error_rate_ZY,
            CorrelatedErrorType::YI => self.error_rate_YI,
            CorrelatedErrorType::YX => self.error_rate_YX,
            CorrelatedErrorType::YZ => self.error_rate_YZ,
            CorrelatedErrorType::YY => self.error_rate_YY,
        }
    }
    pub fn sanity_check(&self) {
        assert!(self.no_error_probability() >= 0., "sum of error rate should be no more than 1");
        assert!(self.error_rate_IX >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_IZ >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_IY >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_XI >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_XX >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_XZ >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_XY >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_ZI >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_ZX >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_ZZ >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_ZY >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_YI >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_YX >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_YZ >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_YY >= 0., "error rate should be greater than 0");
    }
    pub fn generate_random_error(&self, random_number: f64) -> CorrelatedErrorType {
        let mut random_number = random_number;
        if random_number < self.error_rate_IX { return CorrelatedErrorType::IX; } random_number -= self.error_rate_IX;
        if random_number < self.error_rate_IZ { return CorrelatedErrorType::IZ; } random_number -= self.error_rate_IZ;
        if random_number < self.error_rate_IY { return CorrelatedErrorType::IY; } random_number -= self.error_rate_IY;
        if random_number < self.error_rate_XI { return CorrelatedErrorType::XI; } random_number -= self.error_rate_XI;
        if random_number < self.error_rate_XX { return CorrelatedErrorType::XX; } random_number -= self.error_rate_XX;
        if random_number < self.error_rate_XZ { return CorrelatedErrorType::XZ; } random_number -= self.error_rate_XZ;
        if random_number < self.error_rate_XY { return CorrelatedErrorType::XY; } random_number -= self.error_rate_XY;
        if random_number < self.error_rate_ZI { return CorrelatedErrorType::ZI; } random_number -= self.error_rate_ZI;
        if random_number < self.error_rate_ZX { return CorrelatedErrorType::ZX; } random_number -= self.error_rate_ZX;
        if random_number < self.error_rate_ZZ { return CorrelatedErrorType::ZZ; } random_number -= self.error_rate_ZZ;
        if random_number < self.error_rate_ZY { return CorrelatedErrorType::ZY; } random_number -= self.error_rate_ZY;
        if random_number < self.error_rate_YI { return CorrelatedErrorType::YI; } random_number -= self.error_rate_YI;
        if random_number < self.error_rate_YX { return CorrelatedErrorType::YX; } random_number -= self.error_rate_YX;
        if random_number < self.error_rate_YZ { return CorrelatedErrorType::YZ; } random_number -= self.error_rate_YZ;
        if random_number < self.error_rate_YY { return CorrelatedErrorType::YY; }
        CorrelatedErrorType::II
    }
}

/// Correlated erasure error type for two qubit errors
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum CorrelatedErasureErrorType {
    II,
    IE,
    EI,
    EE,
}

impl CorrelatedErasureErrorType {
    pub fn my_error(&self) -> bool {
        match self {
            Self::II | Self::IE => false,
            Self::EI | Self::EE => true,
        }
    }
    pub fn peer_error(&self) -> bool {
        match self {
            Self::II | Self::EI => false,
            Self::IE | Self::EE => true,
        }
    }
    // pub fn all_possible_errors() -> Vec::<Self> {
    //     vec![Self::II, Self::IE, Self::EI, Self::EE]
    // }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct CorrelatedErasureErrorModel {
    pub error_rate_IE: f64,
    pub error_rate_EI: f64,
    pub error_rate_EE: f64,
}

impl CorrelatedErasureErrorModel {
    // pub fn default() -> Self {
    //     Self::default_with_probability(0.)
    // }
    pub fn default_with_probability(p: f64) -> Self {
        Self {
            error_rate_IE: p,
            error_rate_EI: p,
            error_rate_EE: p,
        }
    }
    pub fn error_probability(&self) -> f64 {
        self.error_rate_IE + self.error_rate_EI + self.error_rate_EE
    }
    pub fn no_error_probability(&self) -> f64 {
        1. - self.error_probability()
    }
    // pub fn error_rate(&self, error_type: &CorrelatedErasureErrorType) -> f64 {
    //     match error_type {
    //         CorrelatedErasureErrorType::II => self.no_error_probability(),
    //         CorrelatedErasureErrorType::IE => self.error_rate_IE,
    //         CorrelatedErasureErrorType::EI => self.error_rate_EI,
    //         CorrelatedErasureErrorType::EE => self.error_rate_EE,
    //     }
    // }
    pub fn sanity_check(&self) {
        assert!(self.no_error_probability() >= 0., "sum of error rate should be no more than 1");
        assert!(self.error_rate_IE >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_EI >= 0., "error rate should be greater than 0");
        assert!(self.error_rate_EE >= 0., "error rate should be greater than 0");
    }
    pub fn generate_random_erasure_error(&self, random_number: f64) -> CorrelatedErasureErrorType {
        let mut random_number = random_number;
        if random_number < self.error_rate_IE { return CorrelatedErasureErrorType::IE; } random_number -= self.error_rate_IE;
        if random_number < self.error_rate_EI { return CorrelatedErasureErrorType::EI; } random_number -= self.error_rate_EI;
        if random_number < self.error_rate_EE { return CorrelatedErasureErrorType::EE; }
        CorrelatedErasureErrorType::II
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DecoderType {
    MinimumWeightPerfectMatching,
    UnionFind,
    DistributedUnionFind,
}

impl From<String> for DecoderType {
    fn from(name: String) -> Self {
        match name.as_str() {
            "MWPM" | "MinimumWeightPerfectMatching" => Self::MinimumWeightPerfectMatching,
            "UF" | "UnionFind" => Self::UnionFind,
            "DUF" | "DistributedUnionFind" => Self::DistributedUnionFind,
            _ => panic!("unrecognized decoder type"),
        }
    }
}

impl std::fmt::Display for DecoderType {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::MinimumWeightPerfectMatching => "MinimumWeightPerfectMatching",
            Self::UnionFind => "UnionFind",
            Self::DistributedUnionFind => "DistributedUnionFind",
        })?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ErrorModel {
    GenericBiasedWithBiasedCX,  // arXiv:2104.09539v1 Sec.IV.A
    GenericBiasedWithStandardCX,  // arXiv:2104.09539v1 Sec.IV.A
    ErasureOnlyPhenomenological,  // 100% erasure errors only on the data qubits before the gates happen and on the ancilla qubits after the gates finish
    PauliZandErasurePhenomenological,  // this error model is from https://arxiv.org/pdf/1709.06218v3.pdf
    OnlyGateErrorCircuitLevel,  // errors happen at 4 stages in each measurement round (although removed errors happening at initialization and measurement stage, measurement errors can still occur when curtain error applies on the ancilla after the last gate)
    OnlyGateErrorCircuitLevelCorrelatedErasure,  // the same as `OnlyGateErrorCircuitLevel`, just the erasures are correlated
    Arxiv200404693,  // Huang 2020 paper https://arxiv.org/pdf/2004.04693.pdf (note that periodic boundary condition is currently not supported)
    TailoredYPhenomenological,  // arXiv:1907.02554v2 Biased noise models
    TailoredYCircuitLevel,  // circuit-level biased Y noise model
}

impl From<String> for ErrorModel {
    fn from(name: String) -> Self {
        match name.as_str() {
            "GenericBiasedWithBiasedCX" => Self::GenericBiasedWithBiasedCX,
            "GenericBiasedWithStandardCX" => Self::GenericBiasedWithStandardCX,
            "ErasureOnlyPhenomenological" => Self::ErasureOnlyPhenomenological,
            "PauliZandErasurePhenomenological" => Self::PauliZandErasurePhenomenological,
            "OnlyGateErrorCircuitLevel" => Self::OnlyGateErrorCircuitLevel,
            "OnlyGateErrorCircuitLevelCorrelatedErasure" => Self::OnlyGateErrorCircuitLevelCorrelatedErasure,
            "Arxiv200404693" => Self::Arxiv200404693,
            "TailoredYPhenomenological" => Self::TailoredYPhenomenological,
            "TailoredYCircuitLevel" => Self::TailoredYCircuitLevel,
            _ => panic!("unrecognized error model"),
        }
    }
}

impl std::fmt::Display for ErrorModel {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::GenericBiasedWithBiasedCX => "GenericBiasedWithBiasedCX",
            Self::GenericBiasedWithStandardCX => "GenericBiasedWithStandardCX",
            Self::ErasureOnlyPhenomenological => "ErasureOnlyPhenomenological",
            Self::PauliZandErasurePhenomenological => "PauliZandErasurePhenomenological",
            Self::OnlyGateErrorCircuitLevel => "OnlyGateErrorCircuitLevel",
            Self::OnlyGateErrorCircuitLevelCorrelatedErasure => "OnlyGateErrorCircuitLevelCorrelatedErasure",
            Self::Arxiv200404693 => "Arxiv200404693",
            Self::TailoredYPhenomenological => "TailoredYPhenomenological",
            Self::TailoredYCircuitLevel => "TailoredYCircuitLevel",
        })?;
        Ok(())
    }
}
