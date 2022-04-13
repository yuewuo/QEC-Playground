use std::iter::FromIterator;
use super::serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnionFind {
    /// tree structure, each node has a parent
    pub link_parent: Vec<usize>,
    /// the node information, has the same length as `link_parent`
    pub payload: Vec<Option<UnionNode>>,
    /// internal cache of parent list when calling `find`
    find_parent_list: Vec<usize>,
}

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub struct UnionNode {
    pub set_size: usize,
    pub cardinality: usize,
    pub is_touching_boundary: bool,
}

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub enum UnionResult {
    Left(UnionNode),
    Right(UnionNode),
}

impl UnionNode {
    #[inline]
    pub fn union(left: &Self, right: &Self) -> UnionResult {
        let lsize = left.size();
        let rsize = right.size();
        let result = UnionNode {
            set_size: lsize + rsize,
            cardinality: left.cardinality + right.cardinality,
            is_touching_boundary: left.is_touching_boundary || right.is_touching_boundary,
        };
        if lsize >= rsize {
            UnionResult::Left(result)
        } else {
            UnionResult::Right(result)
        }
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.set_size
    }

}

impl Default for UnionNode {
    #[inline]
    fn default() -> UnionNode {
        UnionNode {
            set_size: 1,
            cardinality: 0,  // by default the cardinality is 0, set to 1 if needed
            is_touching_boundary: false,  // is already touching the boundary
        }
    }
}

impl FromIterator<UnionNode> for UnionFind {
    #[inline]
    fn from_iter<T: IntoIterator<Item = UnionNode>>(iterator: T) -> UnionFind {
        let mut uf = UnionFind {
            link_parent: vec![],
            payload: vec![],
            find_parent_list: Vec::new(),
        };
        uf.extend(iterator);
        uf
    }
}

impl Extend<UnionNode> for UnionFind {
    #[inline]
    fn extend<T: IntoIterator<Item = UnionNode>>(&mut self, iterable: T) {
        let len = self.payload.len();
        let payload = iterable.into_iter().map(Some);
        self.payload.extend(payload);

        let new_len = self.payload.len();
        self.link_parent.extend(len..new_len);

        self.find_parent_list.reserve(self.link_parent.len());
    }
}

impl UnionFind {
    #[inline]
    #[allow(dead_code)]
    pub fn new(len: usize) -> Self {
        Self::from_iter((0..len).map(|_| Default::default()))
    }

    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn insert(&mut self, data: UnionNode) -> usize {
        let key = self.payload.len();
        self.link_parent.push(key);
        self.payload.push(Some(data));
        key
    }

    #[inline]
    pub fn union(&mut self, key0: usize, key1: usize) -> bool {
        let k0 = self.find(key0);
        let k1 = self.find(key1);
        if k0 == k1 {
            return false;
        }

        let (parent, child, val) = match UnionNode::union(self.payload[k0].as_ref().unwrap(), self.payload[k1].as_ref().unwrap()) {
            UnionResult::Left(val) => (k0, k1, val),
            UnionResult::Right(val) => (k1, k0, val),
        };
        self.payload[parent] = Some(val);
        self.link_parent[child] = parent;

        true
    }

    #[inline]
    pub fn find(&mut self, key: usize) -> usize {
        let mut k = key;
        let mut p = self.link_parent[k];
        while p != k {
            self.find_parent_list.push(k);
            k = p;
            p = self.link_parent[p];
        }
        let root = k;
        for k in self.find_parent_list.iter() {
            self.link_parent[*k] = root;  // path compression
        }
        self.find_parent_list.clear();
        root
    }

    #[inline]
    pub fn immutable_find(&self, key: usize) -> usize {
        let mut k = key;
        let mut p = self.link_parent[k];
        while p != k {
            k = p;
            p = self.link_parent[p];
        }
        k
    }

    #[inline]
    pub fn get(&mut self, key: usize) -> &UnionNode {
        let root_key = self.find(key);
        self.payload[root_key].as_ref().unwrap()
    }

    #[inline]
    pub fn immutable_get(&self, key: usize) -> &UnionNode {
        let root_key = self.immutable_find(key);
        self.payload[root_key].as_ref().unwrap()
    }

    #[inline]
    pub fn get_mut(&mut self, key: usize) -> &mut UnionNode {
        let root_key = self.find(key);
        self.payload[root_key].as_mut().unwrap()
    }

    pub fn clear(&mut self) {
        debug_assert!(self.payload.len() == self.link_parent.len());
        for i in 0..self.link_parent.len() {
            self.link_parent[i] = i;
            let node = self.payload[i].as_mut().unwrap();
            node.set_size = 1;
            node.cardinality = 0;
            node.is_touching_boundary = false;
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_find_decoder_test_basic_algorithm() {  // cargo test union_find_decoder_test_basic_algorithm -- --nocapture
        let mut uf = UnionFind::new(100);
        // test from https://github.com/gifnksm/union-find-rs/blob/master/src/tests.rs
        assert_eq!(1, uf.get(0).size());
        assert_eq!(1, uf.get(1).size());
        assert!(uf.find(0) != uf.find(1));
        assert!(uf.immutable_find(0) != uf.immutable_find(1));
        assert!(uf.find(1) != uf.find(2));
        assert!(uf.immutable_find(1) != uf.immutable_find(2));
        assert!(uf.union(0, 1));
        assert!(uf.find(0) == uf.find(1));
        assert!(uf.immutable_find(0) == uf.immutable_find(1));
        assert_eq!(2, uf.get(0).size());
        assert_eq!(2, uf.get(1).size());
        assert_eq!(1, uf.get(2).size());
        assert!(!uf.union(0, 1));
        assert_eq!(2, uf.get(0).size());
        assert_eq!(2, uf.get(1).size());
        assert_eq!(1, uf.get(2).size());
        assert!(uf.union(1, 2));
        assert_eq!(3, uf.get(0).size());
        assert_eq!(3, uf.get(1).size());
        assert_eq!(3, uf.get(2).size());
        assert!(uf.immutable_find(0) == uf.immutable_find(1));
        assert!(uf.find(0) == uf.find(1));
        assert!(uf.immutable_find(2) == uf.immutable_find(1));
        assert!(uf.find(2) == uf.find(1));
        let k100 = uf.insert(UnionNode::default());
        assert_eq!(k100, 100);
        let _ = uf.union(k100, 0);
        assert_eq!(4, uf.get(100).size());
        assert_eq!(101, uf.size());
    }
    
}
