#![allow(non_snake_case)]

use std::fs;
use super::platform_dirs::AppDirs;
use super::lazy_static::lazy_static;
use std::sync::{RwLock};
use std::collections::{BTreeMap};
use std::path::{Path, PathBuf};


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


    impl Default for SimpleHasher {

        #[inline]
        fn default() -> SimpleHasher {
            SimpleHasher(0)
        }
    }

    // impl SimpleHasher {
    //     #[inline]
    //     pub fn set_u64(&mut self, value: u64) {
    //         self.0 = value;
    //     }
    // }

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
pub const TEMPORARY_STORE_MAX_COUNT: usize = 10;  // 100MB max, this option only applies to in memory temporary store; for file-based store, it will not delete any file for safety consideration

pub struct TemporaryStore {
    use_file: bool,  // save data to file instead of in memory, this will also let data persist over program restart
    temporary_store_folder: PathBuf,
    memory_store: BTreeMap<usize, String>,  // in memory store, will not be used if `use_file` is set to true
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
            Ok(_) => { },
            Err(_) => { return None },  // cannot open folder
        }
        match fs::read_to_string(temporary_store.temporary_store_folder.join(format!("{}.dat", resource_id))) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    } else {
        match temporary_store.memory_store.get(&resource_id) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }
}

pub fn local_put_temporary_store(value: String) -> Option<usize> {
    let mut temporary_store = TEMPORARY_STORE.write().unwrap();
    let mut insert_key = 1;  // starting from 1
    if temporary_store.use_file {
        match fs::create_dir_all(&temporary_store.temporary_store_folder) {
            Ok(_) => { },
            Err(_) => { return None },  // cannot create folder
        }
        let paths = match fs::read_dir(&temporary_store.temporary_store_folder) {
            Ok(paths) => { paths },
            Err(_) => { return None },  // cannot read folder
        };
        for path in paths {
            if path.is_err() {
                continue
            }
            let path = path.unwrap().path();
            if path.extension() != Some(&std::ffi::OsStr::new("dat")) {
                continue
            }
            match path.file_stem() {
                Some(file_stem) => {
                    match file_stem.to_string_lossy().parse::<usize>() {
                        Ok(this_key) => {
                            if this_key >= insert_key {
                                insert_key = this_key + 1;
                            }
                        },
                        Err(_) => { },
                    }
                },
                None => { },
            }
        }
        if fs::write(temporary_store.temporary_store_folder.join(format!("{}.dat", insert_key)), value.as_bytes()).is_err() {
            return None;  // failed to write file
        }
    } else {
        let keys: Vec<usize> = temporary_store.memory_store.keys().cloned().collect();
        if keys.len() > 0 {
            insert_key = keys[keys.len() - 1] + 1
        }
        if keys.len() >= TEMPORARY_STORE_MAX_COUNT {  // delete the first one
            temporary_store.memory_store.remove(&keys[0]);
        }
        temporary_store.memory_store.insert(insert_key, value);
    }
    Some(insert_key)
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn temporary_store_read_files() {  // cargo test temporary_store_read_files -- --nocapture
        let resource_id_1 = local_put_temporary_store(format!("hello")).unwrap();
        let resource_id_2 = local_put_temporary_store(format!("world")).unwrap();
        // println!("{:?}", resource_id_1);
        // println!("{:?}", resource_id_2);
        let read_1 = local_get_temporary_store(resource_id_1);
        let read_2 = local_get_temporary_store(resource_id_2);
        assert_eq!(read_1, Some(format!("hello")));
        assert_eq!(read_2, Some(format!("world")));
    }
}
