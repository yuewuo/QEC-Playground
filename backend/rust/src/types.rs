#![allow(non_snake_case)]
#[cfg(feature="python_binding")]
use pyo3::prelude::*;
use serde::{Serialize, Deserialize};

/// Qubit type, corresponds to `QTYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Copy)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum QubitType {
    Data,
    StabX,
    StabZ,
    StabXZZXLogicalX,
    StabXZZXLogicalZ,
    StabY,  // in tailored surface code
}

#[cfg(feature="python_binding")]
#[cfg_attr(feature = "python_binding", pymethods)]
impl QubitType {
    /// if measure in Z basis, it's prepared in |0> state, otherwise it's measuring X basis and prepared in |+> state; data qubit will return None
    pub fn is_measured_in_z_basis(&self) -> Option<bool> {
        match self {
            Self::Data => None,
            Self::StabZ => Some(true),
            Self::StabX | Self::StabXZZXLogicalX | Self::StabXZZXLogicalZ | Self::StabY => Some(false),
        }
    }
}

/// Error type, corresponds to `ETYPE` in `FaultTolerantView.vue`
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "python_binding", pyclass)]
pub enum ErrorType {
    I,
    X,
    Z,
    Y,
}

impl Default for ErrorType {
    fn default() -> Self {
        ErrorType::I  // default to identity
    }
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

impl ErrorType {
    /// multiply two pauli operator
    #[inline]
    pub fn multiply(&self, err: &Self) -> Self {
        // I've checked that the compiler is automatically generating good-quality assembly
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
    //#[staticmethod]
    pub fn all_possible_errors() -> Vec::<Self> {
        vec![Self::X, Self::Z, Self::Y]
    }
    //#[classmethod]
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
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CorrelatedPauliErrorType {
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

impl CorrelatedPauliErrorType {
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

impl std::fmt::Display for CorrelatedPauliErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", match self {
            Self::II => "II", Self::IX => "IX", Self::IZ => "IZ", Self::IY => "IY",
            Self::XI => "XI", Self::XX => "XX", Self::XZ => "XZ", Self::XY => "XY",
            Self::ZI => "ZI", Self::ZX => "ZX", Self::ZZ => "ZZ", Self::ZY => "ZY",
            Self::YI => "YI", Self::YX => "YX", Self::YZ => "YZ", Self::YY => "YY",
        }.to_string())
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct PauliErrorRates {
    #[serde(rename = "px")]
    pub error_rate_X: f64,
    #[serde(rename = "py")]
    pub error_rate_Y: f64,
    #[serde(rename = "pz")]
    pub error_rate_Z: f64,
}

impl PauliErrorRates {
    pub fn default() -> Self {
        Self::default_with_probability(0.)
    }
    pub fn default_with_probability(p: f64) -> Self {
        Self {
            error_rate_X: p,
            error_rate_Z: p,
            error_rate_Y: p,
        }
    }
    #[inline]
    pub fn error_probability(&self) -> f64 {
        self.error_rate_X + self.error_rate_Z + self.error_rate_Y
    }
    pub fn no_error_probability(&self) -> f64 {
        1. - self.error_probability()
    }
    pub fn error_rate(&self, error_type: &ErrorType) -> f64 {
        match error_type {
            ErrorType::I => self.no_error_probability(),
            ErrorType::X => self.error_rate_X,
            ErrorType::Z => self.error_rate_Z,
            ErrorType::Y => self.error_rate_Y,
        }
    }
}


#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CorrelatedPauliErrorRates {
    #[serde(rename = "pix")]
    pub error_rate_IX: f64,
    #[serde(rename = "piz")]
    pub error_rate_IZ: f64,
    #[serde(rename = "piy")]
    pub error_rate_IY: f64,
    #[serde(rename = "pxi")]
    pub error_rate_XI: f64,
    #[serde(rename = "pxx")]
    pub error_rate_XX: f64,
    #[serde(rename = "pxz")]
    pub error_rate_XZ: f64,
    #[serde(rename = "pxy")]
    pub error_rate_XY: f64,
    #[serde(rename = "pzi")]
    pub error_rate_ZI: f64,
    #[serde(rename = "pzx")]
    pub error_rate_ZX: f64,
    #[serde(rename = "pzz")]
    pub error_rate_ZZ: f64,
    #[serde(rename = "pzy")]
    pub error_rate_ZY: f64,
    #[serde(rename = "pyi")]
    pub error_rate_YI: f64,
    #[serde(rename = "pyx")]
    pub error_rate_YX: f64,
    #[serde(rename = "pyz")]
    pub error_rate_YZ: f64,
    #[serde(rename = "pyy")]
    pub error_rate_YY: f64,
}

impl CorrelatedPauliErrorRates {
    pub fn default() -> Self {
        Self::default_with_probability(0.)
    }
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
    pub fn error_rate(&self, error_type: &CorrelatedPauliErrorType) -> f64 {
        match error_type {
            CorrelatedPauliErrorType::II => self.no_error_probability(),
            CorrelatedPauliErrorType::IX => self.error_rate_IX,
            CorrelatedPauliErrorType::IZ => self.error_rate_IZ,
            CorrelatedPauliErrorType::IY => self.error_rate_IY,
            CorrelatedPauliErrorType::XI => self.error_rate_XI,
            CorrelatedPauliErrorType::XX => self.error_rate_XX,
            CorrelatedPauliErrorType::XZ => self.error_rate_XZ,
            CorrelatedPauliErrorType::XY => self.error_rate_XY,
            CorrelatedPauliErrorType::ZI => self.error_rate_ZI,
            CorrelatedPauliErrorType::ZX => self.error_rate_ZX,
            CorrelatedPauliErrorType::ZZ => self.error_rate_ZZ,
            CorrelatedPauliErrorType::ZY => self.error_rate_ZY,
            CorrelatedPauliErrorType::YI => self.error_rate_YI,
            CorrelatedPauliErrorType::YX => self.error_rate_YX,
            CorrelatedPauliErrorType::YZ => self.error_rate_YZ,
            CorrelatedPauliErrorType::YY => self.error_rate_YY,
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
    pub fn generate_random_error(&self, random_number: f64) -> CorrelatedPauliErrorType {
        let mut random_number = random_number;
        if random_number < self.error_rate_IX { return CorrelatedPauliErrorType::IX; } random_number -= self.error_rate_IX;
        if random_number < self.error_rate_IZ { return CorrelatedPauliErrorType::IZ; } random_number -= self.error_rate_IZ;
        if random_number < self.error_rate_IY { return CorrelatedPauliErrorType::IY; } random_number -= self.error_rate_IY;
        if random_number < self.error_rate_XI { return CorrelatedPauliErrorType::XI; } random_number -= self.error_rate_XI;
        if random_number < self.error_rate_XX { return CorrelatedPauliErrorType::XX; } random_number -= self.error_rate_XX;
        if random_number < self.error_rate_XZ { return CorrelatedPauliErrorType::XZ; } random_number -= self.error_rate_XZ;
        if random_number < self.error_rate_XY { return CorrelatedPauliErrorType::XY; } random_number -= self.error_rate_XY;
        if random_number < self.error_rate_ZI { return CorrelatedPauliErrorType::ZI; } random_number -= self.error_rate_ZI;
        if random_number < self.error_rate_ZX { return CorrelatedPauliErrorType::ZX; } random_number -= self.error_rate_ZX;
        if random_number < self.error_rate_ZZ { return CorrelatedPauliErrorType::ZZ; } random_number -= self.error_rate_ZZ;
        if random_number < self.error_rate_ZY { return CorrelatedPauliErrorType::ZY; } random_number -= self.error_rate_ZY;
        if random_number < self.error_rate_YI { return CorrelatedPauliErrorType::YI; } random_number -= self.error_rate_YI;
        if random_number < self.error_rate_YX { return CorrelatedPauliErrorType::YX; } random_number -= self.error_rate_YX;
        if random_number < self.error_rate_YZ { return CorrelatedPauliErrorType::YZ; } random_number -= self.error_rate_YZ;
        if random_number < self.error_rate_YY { return CorrelatedPauliErrorType::YY; }
        CorrelatedPauliErrorType::II
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CorrelatedErasureErrorRates {
    #[serde(rename = "pie")]
    pub error_rate_IE: f64,
    #[serde(rename = "pei")]
    pub error_rate_EI: f64,
    #[serde(rename = "pee")]
    pub error_rate_EE: f64,
}

impl CorrelatedErasureErrorRates {
    pub fn default() -> Self {
        Self::default_with_probability(0.)
    }
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
pub enum ErrorModelName {
    GenericBiasedWithBiasedCX,  // arXiv:2104.09539v1 Sec.IV.A
    GenericBiasedWithStandardCX,  // arXiv:2104.09539v1 Sec.IV.A
    ErasureOnlyPhenomenological,  // 100% erasure errors only on the data qubits before the gates happen and on the ancilla qubits after the gates finish
    PauliZandErasurePhenomenological,  // this error model is from https://arxiv.org/pdf/1709.06218v3.pdf
    OnlyGateErrorCircuitLevel,  // errors happen at 4 stages in each measurement round (although removed errors happening at initialization and measurement stage, measurement errors can still occur when curtain error applies on the ancilla after the last gate)
    OnlyGateErrorCircuitLevelCorrelatedErasure,  // the same as `OnlyGateErrorCircuitLevel`, just the erasures are correlated
    Arxiv200404693,  // Huang 2020 paper https://arxiv.org/pdf/2004.04693.pdf (note that periodic boundary condition is currently not supported)
    TailoredPhenomenological,  // arXiv:1907.02554v2 Biased noise models
}

impl From<String> for ErrorModelName {
    fn from(name: String) -> Self {
        match name.as_str() {
            "GenericBiasedWithBiasedCX" => Self::GenericBiasedWithBiasedCX,
            "GenericBiasedWithStandardCX" => Self::GenericBiasedWithStandardCX,
            "ErasureOnlyPhenomenological" => Self::ErasureOnlyPhenomenological,
            "PauliZandErasurePhenomenological" => Self::PauliZandErasurePhenomenological,
            "OnlyGateErrorCircuitLevel" => Self::OnlyGateErrorCircuitLevel,
            "OnlyGateErrorCircuitLevelCorrelatedErasure" => Self::OnlyGateErrorCircuitLevelCorrelatedErasure,
            "Arxiv200404693" => Self::Arxiv200404693,
            "TailoredPhenomenological" => Self::TailoredPhenomenological,
            _ => panic!("unrecognized error model"),
        }
    }
}

impl std::fmt::Display for ErrorModelName {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::GenericBiasedWithBiasedCX => "GenericBiasedWithBiasedCX",
            Self::GenericBiasedWithStandardCX => "GenericBiasedWithStandardCX",
            Self::ErasureOnlyPhenomenological => "ErasureOnlyPhenomenological",
            Self::PauliZandErasurePhenomenological => "PauliZandErasurePhenomenological",
            Self::OnlyGateErrorCircuitLevel => "OnlyGateErrorCircuitLevel",
            Self::OnlyGateErrorCircuitLevelCorrelatedErasure => "OnlyGateErrorCircuitLevelCorrelatedErasure",
            Self::Arxiv200404693 => "Arxiv200404693",
            Self::TailoredPhenomenological => "TailoredPhenomenological",
        })?;
        Ok(())
    }
}

#[cfg(feature="python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<ErrorType>()?;
    m.add_class::<QubitType>()?;
    Ok(())
}
