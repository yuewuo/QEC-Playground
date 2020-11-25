use super::serde_json;
use super::ndarray;
use super::serde_json::{json, Value, Map};
use std::fs::File;
use std::fs;
use std::io::prelude::*;

/// load measurement or ground truth data from file
#[allow(non_snake_case)]
pub fn load(filepath: &str) -> std::io::Result<(Value, ndarray::Array3<bool>)> {
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
    let mut data_ro = ndarray::Array::from_shape_fn((N, L, L), |_| false);
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
    Ok((head, data_ro))
}

/// save measurement or gound truth data to file
#[allow(non_snake_case)]
pub fn save(filepath: &str, head: &Value, data: &ndarray::Array3<bool>) -> std::io::Result<()> {
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
