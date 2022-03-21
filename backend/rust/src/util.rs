#![allow(non_snake_case)]

use std::fs;
use std::path::Path;


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

// if even but the median is not unique, return None
pub fn find_strict_one_median(numbers: &mut Vec<usize>) -> Option<usize> {
    numbers.sort();
    if numbers.len() % 2 == 0 {
        let first = numbers[numbers.len() / 2 - 1];
        let second = numbers[numbers.len() / 2];
        if first != second {
            None
        } else {
            Some(first)
        }
    } else {
        Some(numbers[numbers.len() / 2])
    }
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

#[cfg(test)]
mod tests {

    use super::*;

    // use `cargo test util_find_strict_one_median -- --nocapture` to run specific test

    #[test]
    fn util_find_strict_one_median() {
        assert_eq!(find_strict_one_median(&mut vec![4,3,3,8]), None);
        assert_eq!(find_strict_one_median(&mut vec![4,3,4,8]), Some(4));
        assert_eq!(find_strict_one_median(&mut vec![5,3,7]), Some(5));
    }

}
