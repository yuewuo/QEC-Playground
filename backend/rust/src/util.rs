use super::serde_json;
use super::ndarray;
use super::serde_json::{json, Value, Map};
use std::fs::File;
use std::io::prelude::*;

/// load measurement or ground truth data from file
pub fn load(filepath: &str) -> (Value, ndarray::Array3<bool>) {
    let head = json!({
        "a": "b"
    });
    let data = ndarray::array![[[true],[false],[false],[true]]];
    (head, data)
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
    let mut head: Map<String, Value> = serde_json::from_value(head.clone()).unwrap();
    head.insert("N".to_string(), json!(N));
    head.insert("L".to_string(), json!(L));
    let head: Value = serde_json::to_value(&head).unwrap();
    // write to file
    let mut f = File::create(filepath)?;
    serde_json::to_writer(&f, &head)?;
    f.write(b"\0")?;
    let cycle = (((L*L) as f64) / 8f64).ceil() as usize;
    let mut vec = vec![0u8; cycle];
    for i in 0..N {
        for item in &mut vec { *item = 0u8; }  // reset to 0
        let mut l = 0usize;
        for j in 0..L {
            for k in 0..L {
                if data[[i, j, k]] == true {
                    let byte_idx = l / 8;
                    let bit_idx = l % 8;
                    vec[byte_idx] |= 1 << bit_idx;
                    l += 1;
                }
            }
        }
    }
    f.write(&vec)?;
    Ok(())
}
