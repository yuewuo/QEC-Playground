use super::serde_json;
use super::ndarray;

/// load measurement or ground truth data from file
pub fn load(filepath: &str) -> (serde_json::Value, ndarray::Array3<bool>) {
    let head = serde_json::json!({
        "a": "b"
    });
    let data = ndarray::array![[[true],[false],[false],[true]]];
    (head, data)
}

/// save measurement or gound truth data to file
pub fn save(filepath: &str, head: &serde_json::Value, data: &ndarray::Array3<bool>) {

}
