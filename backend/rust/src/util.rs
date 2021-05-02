#![allow(non_snake_case)]

use super::serde_json;
use super::ndarray;
use super::serde_json::{json, Value, Map};
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use super::types::*;
use std::path::Path;

/// load measurement or ground truth data from file
pub fn load(filepath: &str) -> std::io::Result<(Value, BatchZxError)> {
    let file_bytes = fs::read(filepath)?;
    let split_idx = file_bytes.iter().position(|&x| x == 0).expect("should split with \\0");
    let head_bytes = &file_bytes[..split_idx];
    let data_bytes = &file_bytes[split_idx+1..];
    let head: Value = serde_json::from_slice(head_bytes).expect("JSON deserialize error");
    let N = head.get("N").expect("mandatory field N").as_u64().expect("u64 N") as usize;
    let L = head.get("L").expect("mandatory field L").as_u64().expect("u64 L") as usize;
    assert!(N > 0 && L > 0);
    let cycle = (((L*L) as f64) / 8f64).ceil() as usize;
    assert!(data_bytes.len() > 0 && data_bytes.len() == cycle * N);
    // generate data
    let mut data_ro = ndarray::Array::from_elem((N, L, L), false);
    let mut data = data_ro.view_mut();
    for i in 0..N {
        let base_idx = i * cycle;
        let mut l = 0;
        for j in 0..L {
            for k in 0..L {
                let byte_idx = base_idx + l / 8;
                let bit_idx = l % 8;
                data[[i, j, k]] = 0 != (data_bytes[byte_idx] & (1 << bit_idx));
                l += 1;
            }
        }
    }
    Ok((head, BatchZxError::new(data_ro)))
}

/// save measurement or ground truth data to file
pub fn save(filepath: &str, head: &Value, data: &BatchZxError) -> std::io::Result<()> {
    // check input format
    assert_eq!(None, head.get("N"));
    assert_eq!(None, head.get("L"));
    let shape = data.shape();
    assert_eq!(shape.len(), 3);
    assert_eq!(shape[1], shape[2]);
    let N = shape[0];
    let L = shape[1];
    // modify head
    let mut head: Map<String, Value> = serde_json::from_value(head.clone()).expect("head JSON error");
    head.insert("N".to_string(), json!(N));
    head.insert("L".to_string(), json!(L));
    let head: Value = serde_json::to_value(&head).expect("head JSON serialization error");
    // write to file
    let mut f = File::create(filepath)?;
    serde_json::to_writer(&f, &head)?;
    f.write(b"\0")?;
    let cycle = (((L*L) as f64) / 8f64).ceil() as usize;
    let mut vec = vec![0u8; cycle * N];  // more memory but faster
    for i in 0..N {
        let mut l = 0usize;
        let base_idx = i * cycle;
        for j in 0..L {
            for k in 0..L {
                if data[[i, j, k]] == true {
                    let byte_idx = base_idx + l / 8;
                    let bit_idx = l % 8;
                    vec[byte_idx] |= 1 << bit_idx;
                }
                l += 1;
            }
        }
    }
    f.write(&vec)?;
    Ok(())
}

/// X errors are only detected by Z stabilizers
pub fn generate_perfect_measurements(x_error: &ZxError, z_error: &ZxError) -> ZxMeasurement {
    assert_eq!(x_error.shape(), z_error.shape());
    let L = x_error.L();
    let mut measurement_ro = ZxMeasurement::new_L(L);
    let mut measurement = measurement_ro.view_mut();
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
                measurement[[i, j]] = result;
            }
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
                measurement[[i, j]] = result;
            }
        }
    }
    measurement_ro
}

/// filename should contain .py, folders should end with slash
pub fn getFileContentFromMultiplePlaces(folders: &Vec<String>, filename: &String) -> Result<String, String> {
    for folder in folders {
        let path = Path::new(folder).join(filename.as_str());
        if path.exists() {
            if let Some(path_str) = path.to_str() {
                let contents = fs::read_to_string(path_str);
                if let Ok(content) = contents {
                    return Ok(content);
                }
            }
        }
    }
    Err(format!("cannot find '{}' from folders {:?}", filename, folders))
}
