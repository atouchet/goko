//! # Point Cloud
//! Abstracts data access over several files and glues metadata files to vector data files

use std::convert::{TryInto, TryFrom};
use std::ops::Deref;
use std::convert::AsRef;
use std::marker::PhantomData;
use crate::PointRef;


impl<'a,T> PointRef<T> for &'a [T]
    where
    T: Send + Sync + Copy + 'static {
    type DenseIter =  std::iter::Copied<std::slice::Iter<'a,T>>;
    fn dense(&self) -> Vec<T> {
        Vec::from(*self)
    }
    fn dense_iter(&self) -> Self::DenseIter {
        self.iter().copied()
    }
}


#[derive(Debug)]
/// Enables iterating thru a sparse vector, like a dense vector without allocating anythin
pub struct SparseDenseIter<'a, T: std::fmt::Debug, S: std::fmt::Debug> {
    raw: RawSparse<T, S>,
    index: usize,
    sparse_index: usize,
    lifetime: PhantomData<&'a T>,
}

impl<'a, T, S> Iterator for SparseDenseIter<'a, T, S> 
    where
    T: std::fmt::Debug + Default + Copy,
    S: Ord + TryInto<usize> + std::fmt::Debug + Copy  {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let dim = self.raw.dim();
        if self.index < dim && self.sparse_index < self.raw.len {

            let raw_si = unsafe {
                *self.raw.indexes_ptr.add(self.sparse_index)
            };

            let si: usize = match raw_si.try_into() {
                Ok(si) => si,
                Err(_) => panic!("Could not covert a sparse index into a usize"),
            };

            if si == self.index  {
                let val = unsafe {
                    *self.raw.values_ptr.add(self.sparse_index)
                };
                self.sparse_index += 1;
                self.index += 1;
                
                Some(val)
            } else if self.index < dim {
                self.index += 1;
                Some(T::default())
            } else {
                None
            }
            
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let dim = self.raw.dim();
        (dim, Some(dim))
    }
}

#[derive(Debug)]
pub struct Sparse<CoefField: std::fmt::Debug, Index: std::fmt::Debug> {
    pub dim: Index,
    pub values: Vec<CoefField>,
    pub indexes: Vec<Index>,
}

#[derive(Debug)]
pub struct RawSparse<T, S> {
    dim: usize,
    values_ptr: *const T,
    indexes_ptr: *const S,
    len: usize,
}

#[derive(Debug)]
pub struct SparseRef<'a, T, S> {
    raw: RawSparse<T,S>,
    lifetime: PhantomData<&'a T>,
}

unsafe impl<T: Send, S: Send> Send for RawSparse<T,S> {}
unsafe impl<T: Sync, S: Sync> Sync for RawSparse<T,S> {}

impl<T: std::fmt::Debug, S: std::fmt::Debug + TryInto<usize>> RawSparse<T,S> {
    pub fn indexes<'a>(&'a self) -> &'a [S] {
        unsafe { std::slice::from_raw_parts::<'a>(self.indexes_ptr,self.len) }
    }

    pub fn values<'a>(&'a self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts::<'a>(self.values_ptr,self.len) }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }
}


impl<'a, T, S: TryInto<usize>> SparseRef<'a,T,S> {
    pub fn new<'b>(dim:usize,values: &'b [T], indexes: &'b [S]) -> SparseRef<'b,T,S> {
        let len = values.len();
        assert_eq!(indexes.len(),len,"Need the indexes and values to be of identical len");
        let indexes_ptr: *const S = indexes.as_ptr();
        let values_ptr: *const T = values.as_ptr();
        let raw = RawSparse {
            indexes_ptr,
            values_ptr,
            dim,
            len,
        };
        SparseRef {
            raw,
            lifetime: PhantomData,
        }
    }

    pub fn indexes(&self) -> &'a [S] {
        unsafe { std::slice::from_raw_parts::<'a>(self.raw.indexes_ptr,self.raw.len) }
    }

    pub fn values(&self) -> &'a [T] {
        unsafe { std::slice::from_raw_parts::<'a>(self.raw.values_ptr,self.raw.len) }
    }

    pub fn dim(&self) -> usize {
        self.raw.dim
    }
}


impl<'a, T, S> Deref for SparseRef<'a,T,S> {
    type Target = RawSparse<T,S>;
    fn deref(&self) -> &Self::Target {
        &self.raw
    }
}

impl<'a,S,T> PointRef<T> for SparseRef<'a, T, S> 
where
    S: TryInto<usize> + Ord + TryFrom<usize> + std::fmt::Debug + Copy + Send + Sync + 'static,
    T: std::fmt::Debug + Default + Copy + 'static + Send + Sync {
    type DenseIter = SparseDenseIter<'a, T, S>;

    fn dense(&self) -> Vec<T> {
        let dim = self.dim();
        let mut v = vec![T::default();dim];
        
        for (xi,i) in self.values().iter().zip(self.indexes()) {
            match (*i).try_into() {
                Ok(i) => {
                    let _index: usize = i; 
                    v[i] = *xi;
                }
                Err(_) => panic!("Could not covert a sparse index into a usize"),
            }
        }
        v
    }

    fn dense_iter(&self) -> SparseDenseIter<'a, T, S> {
        let raw = RawSparse {
            dim: self.raw.dim,
            values_ptr: self.raw.values_ptr,
            indexes_ptr: self.raw.indexes_ptr,
            len: self.raw.len,
        };
        SparseDenseIter {
            raw,
            index: 0,
            sparse_index: 0,
            lifetime: PhantomData,
        }
    }
}
