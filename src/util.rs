#![allow(non_snake_case)]

use super::lazy_static::lazy_static;
use super::platform_dirs::AppDirs;
#[cfg(feature = "python_binding")]
use pyo3::prelude::*;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

/// filename should contain .py, folders should end with slash
#[allow(dead_code)]
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

// https://users.rust-lang.org/t/hashmap-performance/6476/8
// https://gist.github.com/arthurprs/88eef0b57b9f8341c54e2d82ec775698
// a much simpler but super fast hasher, only suitable for `ftqec::Index`!!!
pub mod simple_hasher {
    use std::hash::Hasher;
    #[derive(Default)]
    pub struct SimpleHasher(u64);

    #[inline]
    fn load_u64_le(buf: &[u8], len: usize) -> u64 {
        use std::ptr;
        debug_assert!(len <= buf.len());
        let mut data = 0u64;
        unsafe {
            ptr::copy_nonoverlapping(buf.as_ptr(), &mut data as *mut _ as *mut u8, len);
        }
        data.to_le()
    }

    impl Hasher for SimpleHasher {
        #[inline]
        fn finish(&self) -> u64 {
            self.0
        }

        #[inline]
        fn write(&mut self, bytes: &[u8]) {
            if self.0 != 0 {
                panic!("do not use SimpleHasher for struct other than ftqec::Index");
            }
            let value = load_u64_le(bytes, bytes.len());
            // println!("value: {}", value);
            *self = SimpleHasher(value);
        }
    }
}

#[allow(dead_code)]
pub const TEMPORARY_STORE_MAX_COUNT: usize = 10; // 100MB max, this option only applies to in memory temporary store; for file-based store, it will not delete any file for safety consideration

pub struct TemporaryStore {
    use_file: bool, // save data to file instead of in memory, this will also let data persist over program restart
    temporary_store_folder: PathBuf,
    memory_store: BTreeMap<usize, String>, // in memory store, will not be used if `use_file` is set to true
}

lazy_static! {
    // must use RwLock, because web request will lock as a reader, and tool.rs will also acquire a reader lock
    pub static ref TEMPORARY_STORE: RwLock<TemporaryStore> = RwLock::new(TemporaryStore {
        use_file: true,  // suitable for low memory machines, by default
        temporary_store_folder: AppDirs::new(Some("qec"), true).unwrap().data_dir.join("temporary-store"),
        memory_store: BTreeMap::new(),
    });
}

pub fn local_get_temporary_store(resource_id: usize) -> Option<String> {
    let temporary_store = TEMPORARY_STORE.read().unwrap();
    if temporary_store.use_file {
        match fs::create_dir_all(&temporary_store.temporary_store_folder) {
            Ok(_) => {}
            Err(_) => return None, // cannot open folder
        }
        match fs::read_to_string(temporary_store.temporary_store_folder.join(format!("{}.dat", resource_id))) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    } else {
        temporary_store.memory_store.get(&resource_id).cloned()
    }
}

pub fn local_put_temporary_store(value: String) -> Option<usize> {
    let mut temporary_store = TEMPORARY_STORE.write().unwrap();
    let mut insert_key = 1; // starting from 1
    if temporary_store.use_file {
        match fs::create_dir_all(&temporary_store.temporary_store_folder) {
            Ok(_) => {}
            Err(_) => return None, // cannot create folder
        }
        let paths = match fs::read_dir(&temporary_store.temporary_store_folder) {
            Ok(paths) => paths,
            Err(_) => return None, // cannot read folder
        };
        for path in paths {
            if path.is_err() {
                continue;
            }
            let path = path.unwrap().path();
            if path.extension() != Some(std::ffi::OsStr::new("dat")) {
                continue;
            }
            if let Some(file_stem) = path.file_stem() {
                if let Ok(this_key) = file_stem.to_string_lossy().parse::<usize>() {
                    if this_key >= insert_key {
                        insert_key = this_key + 1;
                    }
                }
            }
        }
        if fs::write(
            temporary_store.temporary_store_folder.join(format!("{}.dat", insert_key)),
            value.as_bytes(),
        )
        .is_err()
        {
            return None; // failed to write file
        }
    } else {
        let keys: Vec<usize> = temporary_store.memory_store.keys().cloned().collect();
        if !keys.is_empty() {
            insert_key = keys[keys.len() - 1] + 1
        }
        if keys.len() >= TEMPORARY_STORE_MAX_COUNT {
            // delete the first one
            temporary_store.memory_store.remove(&keys[0]);
        }
        temporary_store.memory_store.insert(insert_key, value);
    }
    Some(insert_key)
}

/**
 * If you want to modify a field of a Rust struct, it will return a copy of it to avoid memory unsafety.
 * Thus, typical way of modifying a python field doesn't work, e.g. `obj.a.b.c = 1` won't actually modify `obj`.
 * This helper class is used to modify a field easier; but please note this can be very time consuming if not optimized well.
 *
 * Example:
 * with PyMut(code, "vertices") as vertices:
 *     with fb.PyMut(vertices[0], "position") as position:
 *         position.i = 100
*/
#[cfg(feature = "python_binding")]
#[pyclass]
pub struct PyMut {
    /// the python object that provides getter and setter function for the attribute
    #[pyo3(get, set)]
    object: PyObject,
    /// the name of the attribute
    #[pyo3(get, set)]
    attr_name: String,
    /// the python attribute object that is taken from `object[attr_name]`
    #[pyo3(get, set)]
    attr_object: Option<PyObject>,
}

#[cfg(feature = "python_binding")]
#[pymethods]
impl PyMut {
    #[new]
    pub fn new(object: PyObject, attr_name: String) -> Self {
        Self {
            object,
            attr_name,
            attr_object: None,
        }
    }
    pub fn __enter__(&mut self) -> PyObject {
        assert!(self.attr_object.is_none(), "do not enter twice");
        Python::with_gil(|py| {
            let attr_object = self.object.getattr(py, self.attr_name.as_str()).unwrap();
            self.attr_object = Some(attr_object.clone_ref(py));
            attr_object
        })
    }
    pub fn __exit__(&mut self, _exc_type: PyObject, _exc_val: PyObject, _exc_tb: PyObject) {
        Python::with_gil(|py| {
            self.object
                .setattr(py, self.attr_name.as_str(), self.attr_object.take().unwrap())
                .unwrap()
        })
    }
}

#[cfg(feature = "python_binding")]
pub fn json_to_pyobject_locked<'py>(value: serde_json::Value, py: Python<'py>) -> PyObject {
    match value {
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(value) => value.to_object(py).into(),
        serde_json::Value::Number(value) => {
            if value.is_i64() {
                value.as_i64().to_object(py).into()
            } else {
                value.as_f64().to_object(py).into()
            }
        }
        serde_json::Value::String(value) => value.to_object(py).into(),
        serde_json::Value::Array(array) => {
            let elements: Vec<PyObject> = array.into_iter().map(|value| json_to_pyobject_locked(value, py)).collect();
            pyo3::types::PyList::new(py, elements).into()
        }
        serde_json::Value::Object(map) => {
            let pydict = pyo3::types::PyDict::new(py);
            for (key, value) in map.into_iter() {
                let pyobject = json_to_pyobject_locked(value, py);
                pydict.set_item(key, pyobject).unwrap();
            }
            pydict.into()
        }
    }
}

#[cfg(feature = "python_binding")]
pub fn json_to_pyobject(value: serde_json::Value) -> PyObject {
    Python::with_gil(|py| json_to_pyobject_locked(value, py))
}

#[cfg(feature = "python_binding")]
pub fn pyobject_to_json_locked<'py>(value: PyObject, py: Python<'py>) -> serde_json::Value {
    let value: &PyAny = value.as_ref(py);
    if value.is_none() {
        serde_json::Value::Null
    } else if value.is_instance_of::<pyo3::types::PyBool>().unwrap() {
        json!(value.extract::<bool>().unwrap())
    } else if value.is_instance_of::<pyo3::types::PyInt>().unwrap() {
        json!(value.extract::<i64>().unwrap())
    } else if value.is_instance_of::<pyo3::types::PyFloat>().unwrap() {
        json!(value.extract::<f64>().unwrap())
    } else if value.is_instance_of::<pyo3::types::PyString>().unwrap() {
        json!(value.extract::<String>().unwrap())
    } else if value.is_instance_of::<pyo3::types::PyList>().unwrap() {
        let elements: Vec<serde_json::Value> = value
            .extract::<Vec<PyObject>>()
            .unwrap()
            .into_iter()
            .map(|object| pyobject_to_json_locked(object, py))
            .collect();
        json!(elements)
    } else if value.is_instance_of::<pyo3::types::PyDict>().unwrap() {
        let map: &pyo3::types::PyDict = value.downcast().unwrap();
        let mut json_map = serde_json::Map::new();
        for (key, value) in map.iter() {
            json_map.insert(
                key.extract::<String>().unwrap(),
                pyobject_to_json_locked(value.to_object(py), py),
            );
        }
        serde_json::Value::Object(json_map)
    } else {
        unimplemented!("unsupported python type, should be (cascaded) dict, list and basic numerical types")
    }
}

#[cfg(feature = "python_binding")]
pub fn pyobject_to_json(value: PyObject) -> serde_json::Value {
    Python::with_gil(|py| pyobject_to_json_locked(value, py))
}

#[cfg(feature = "python_binding")]
#[pyfunction]
pub(crate) fn register(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyMut>()?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn temporary_store_read_files() {
        // cargo test temporary_store_read_files -- --nocapture
        let resource_id_1 = local_put_temporary_store("hello".to_string()).unwrap();
        let resource_id_2 = local_put_temporary_store("world".to_string()).unwrap();
        // println!("{:?}", resource_id_1);
        // println!("{:?}", resource_id_2);
        let read_1 = local_get_temporary_store(resource_id_1);
        let read_2 = local_get_temporary_store(resource_id_2);
        assert_eq!(read_1, Some("hello".to_string()));
        assert_eq!(read_2, Some("world".to_string()));
    }
}
