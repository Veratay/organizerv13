use std::{slice::{Iter,IterMut}, ops::{Index, IndexMut}, fmt::Debug};

pub struct IndexMap<T> {
    vec:Vec<(usize,T)>,
    last_idx:usize
}

impl<T> Debug for IndexMap<T> where T:Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vec.fmt(f)
    }
}

impl<T> Clone for IndexMap<T> where T:Clone {
    fn clone(&self) -> Self {
        Self { vec: self.vec.clone(), last_idx: self.last_idx }
    }
}

impl<T> IndexMap<T> {
    pub fn new() -> Self {
        Self {
            vec:Vec::new(),
            last_idx:0
        }
    }

    pub fn push(&mut self, entry:T) -> usize {
        self.last_idx += 1;
        self.vec.push((self.last_idx,entry));
        self.last_idx
    }

    pub fn iter(&self) -> Iter<'_, (usize, T)> {
        self.vec.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_,(usize, T)> {
        self.vec.iter_mut()
    }

    pub fn values(&self) -> Values<'_,T> {
        Values { inner: self.vec.iter() }
    }

    pub fn values_mut(&mut self) -> ValuesMut<'_,T> {
        ValuesMut { inner: self.vec.iter_mut() }
    }

    pub fn remove(&mut self, idx:usize) -> T {
        let i = match self.vec.binary_search_by_key(&idx, |&(u,_)| u) {
            Ok(i) => i,
            Err(_) => panic!("Removal idx does not exist")
        };
        self.vec.remove(i).1
    }
    pub fn try_remove(&mut self, idx:usize) -> Option<T> {
        let i = match self.vec.binary_search_by_key(&idx, |&(u,_)| u) {
            Ok(i) => i,
            Err(_) => return None
        };
        Some(self.vec.remove(i).1)
    }
}

impl<T> IndexMap<T> where T:Debug {
    
}

impl<T> Index<usize> for IndexMap<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        let i = self.vec.binary_search_by_key(&index, |(s,_)| *s).unwrap();
        &self.vec[i].1
    }
}

impl<T> IndexMut<usize> for IndexMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let i = self.vec.binary_search_by_key(&index, |(s,_)| *s).unwrap();
        &mut self.vec[i].1
    }
}

pub struct Values<'a, T> {
    inner: Iter<'a, (usize,T)>,
}

impl<'a, T> Iterator for Values<'a, T> {
    type Item = &'a T;
    
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.inner.next().map(|(_, v)| v)
    }
}

pub struct ValuesMut<'a,T> {
    inner: IterMut<'a, (usize,T)>,
}

impl<'a, T> Iterator for ValuesMut<'a, T> {
    type Item = &'a mut T;
    
    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        self.inner.next().map(|(_, v)| v)
    }
}
