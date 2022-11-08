use std::iter::FromIterator;
use super::serde::{Serialize, Deserialize};
use super::either::Either;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnionFindGeneric<NodeType: UnionNodeTrait> {
    /// tree structure, each node has a parent
    pub link_parent: Vec<usize>,
    /// the node information, has the same length as `link_parent`
    pub payload: Vec<NodeType>,
    /// internal cache of parent list when calling `find`
    find_parent_list: Vec<usize>,
}

pub trait UnionNodeTrait {
    fn union(left: &Self, right: &Self) -> Either<Self, Self> where Self: Sized;
    fn clear(&mut self);
}

/* copy this part and modify as you like */

pub type DefaultUnionFind = UnionFindGeneric<DefaultUnionNode>;

#[derive(Copy, Debug, Serialize, Deserialize, Clone)]
pub struct DefaultUnionNode {
    pub set_size: usize,
    pub cardinality: usize,
    pub is_touching_boundary: bool,
}

impl UnionNodeTrait for DefaultUnionNode {

    #[inline]
    fn union(left: &Self, right: &Self) -> Either<Self, Self> {
        let lsize = left.set_size;
        let rsize = right.set_size;
        let result = Self {
            set_size: lsize + rsize,
            cardinality: left.cardinality + right.cardinality,
            is_touching_boundary: left.is_touching_boundary || right.is_touching_boundary,
        };
        if lsize >= rsize {
            Either::Left(result)
        } else {
            Either::Right(result)
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.set_size = 1;
        self.cardinality = 0;
        self.is_touching_boundary = false;
    }

}

impl Default for DefaultUnionNode {
    #[inline]
    fn default() -> Self {
        Self {
            set_size: 1,
            cardinality: 0,  // by default the cardinality is 0, set to 1 if needed
            is_touching_boundary: false,  // is already touching the boundary
        }
    }
}

/* copy the above and modify as you like */

impl<U: UnionNodeTrait> FromIterator<U> for UnionFindGeneric<U> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = U>>(iterator: T) -> UnionFindGeneric<U> {
        let mut uf = UnionFindGeneric::<U> {
            link_parent: vec![],
            payload: vec![],
            find_parent_list: Vec::new(),
        };
        uf.extend(iterator);
        uf
    }
}

impl<U: UnionNodeTrait> Extend<U> for UnionFindGeneric<U> {
    #[inline]
    fn extend<T: IntoIterator<Item = U>>(&mut self, iterable: T) {
        let len = self.payload.len();
        let payload = iterable.into_iter();
        self.payload.extend(payload);

        let new_len = self.payload.len();
        self.link_parent.extend(len..new_len);

        self.find_parent_list.reserve(self.link_parent.len());
    }
}

impl<U: UnionNodeTrait + Default> UnionFindGeneric<U> {
    #[inline]
    #[allow(dead_code)]
    pub fn new(len: usize) -> Self {
        Self::from_iter((0..len).map(|_| Default::default()))
    }
}

impl<U: UnionNodeTrait> UnionFindGeneric<U> {
    #[inline]
    #[allow(dead_code)]
    pub fn size(&self) -> usize {
        self.payload.len()
    }

    #[inline]
    #[allow(dead_code)]
    pub fn insert(&mut self, data: U) -> usize {
        let key = self.payload.len();
        self.link_parent.push(key);
        self.payload.push(data);
        key
    }

    #[inline(never)]
    pub fn union(&mut self, key0: usize, key1: usize) -> bool {
        let k0 = self.find(key0);
        let k1 = self.find(key1);
        if k0 == k1 {
            return false;
        }

        let (parent, child, val) = match U::union(&self.payload[k0], &self.payload[k1]) {
            Either::Left(val) => (k0, k1, val),
            Either::Right(val) => (k1, k0, val),
        };
        self.payload[parent] = val;
        self.link_parent[child] = parent;

        true
    }

    #[inline(never)]
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

    #[inline(never)]
    pub fn immutable_find(&self, key: usize) -> usize {
        let mut k = key;
        let mut p = self.link_parent[k];
        while p != k {
            k = p;
            p = self.link_parent[p];
        }
        k
    }

    #[inline(never)]
    pub fn get(&mut self, key: usize) -> &U {
        let root_key = self.find(key);
        &self.payload[root_key]
    }

    #[inline(never)]
    #[allow(dead_code)]
    pub fn immutable_get(&self, key: usize) -> &U {
        let root_key = self.immutable_find(key);
        &self.payload[root_key]
    }

    #[inline(never)]
    pub fn get_mut(&mut self, key: usize) -> &mut U {
        let root_key = self.find(key);
        &mut self.payload[root_key]
    }

    pub fn clear(&mut self) {
        debug_assert!(self.payload.len() == self.link_parent.len());
        for i in 0..self.link_parent.len() {
            self.link_parent[i] = i;
            let node = &mut self.payload[i];
            node.clear();
        }
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn union_find_decoder_test_basic_algorithm() {  // cargo test union_find_decoder_test_basic_algorithm -- --nocapture
        let mut uf = DefaultUnionFind::new(100);
        // test from https://github.com/gifnksm/union-find-rs/blob/master/src/tests.rs
        assert_eq!(1, uf.get(0).set_size);
        assert_eq!(1, uf.get(1).set_size);
        assert!(uf.find(0) != uf.find(1));
        assert!(uf.immutable_find(0) != uf.immutable_find(1));
        assert!(uf.find(1) != uf.find(2));
        assert!(uf.immutable_find(1) != uf.immutable_find(2));
        assert!(uf.union(0, 1));
        assert!(uf.find(0) == uf.find(1));
        assert!(uf.immutable_find(0) == uf.immutable_find(1));
        assert_eq!(2, uf.get(0).set_size);
        assert_eq!(2, uf.get(1).set_size);
        assert_eq!(1, uf.get(2).set_size);
        assert!(!uf.union(0, 1));
        assert_eq!(2, uf.get(0).set_size);
        assert_eq!(2, uf.get(1).set_size);
        assert_eq!(1, uf.get(2).set_size);
        assert!(uf.union(1, 2));
        assert_eq!(3, uf.get(0).set_size);
        assert_eq!(3, uf.get(1).set_size);
        assert_eq!(3, uf.get(2).set_size);
        assert!(uf.immutable_find(0) == uf.immutable_find(1));
        assert!(uf.find(0) == uf.find(1));
        assert!(uf.immutable_find(2) == uf.immutable_find(1));
        assert!(uf.find(2) == uf.find(1));
        let k100 = uf.insert(DefaultUnionNode::default());
        assert_eq!(k100, 100);
        let _ = uf.union(k100, 0);
        assert_eq!(4, uf.get(100).set_size);
        assert_eq!(101, uf.size());
    }
    
}
