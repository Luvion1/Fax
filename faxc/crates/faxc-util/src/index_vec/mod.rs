//! IndexVec - A vector indexed by a specific type.
//!
//! This module provides [`IndexVec`], a typed vector that uses a custom index type
//! instead of `usize`. This provides type safety and prevents mixing up indices
//! from different collections.
//!
//! # Example
//!
//! ```
//! use faxc_util::index_vec::{IndexVec, Idx};
//!
//! #[derive(Clone, Copy, Debug, PartialEq, Eq)]
//! struct ExprId(u32);
//!
//! impl Idx for ExprId {
//!     fn from_usize(idx: usize) -> Self { ExprId(idx as u32) }
//!     fn index(self) -> usize { self.0 as usize }
//! }
//!
//! let mut exprs: IndexVec<ExprId, i32> = IndexVec::new();
//! let id = exprs.push(42);
//! let value = exprs[id];
//! assert_eq!(value, 42);
//! ```

use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

/// Trait for types that can be used as indices
///
/// This trait must be implemented by any type that wants to be used as an
/// index for [`IndexVec`]. The type must be convertible to and from `usize`.
///
/// # Requirements
///
/// - Must be `Copy` for efficient passing
/// - Must be `Eq + PartialEq` for comparison
/// - Must implement `from_usize` and `index` for conversion
///
/// # Example
///
/// ```
/// use faxc_util::index_vec::Idx;
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// struct MyIndex(u32);
///
/// impl Idx for MyIndex {
///     fn from_usize(idx: usize) -> Self {
///         assert!(idx <= u32::MAX as usize);
///         MyIndex(idx as u32)
///     }
///
///     fn index(self) -> usize {
///         self.0 as usize
///     }
/// }
/// ```
pub trait Idx: Copy + Eq + PartialEq {
    /// Convert from usize to index type
    ///
    /// # Panics
    ///
    /// Implementations may panic if the usize value is too large to fit
    /// in the index type.
    fn from_usize(idx: usize) -> Self;

    /// Convert index to usize for array indexing
    fn index(self) -> usize;
}

/// A vector indexed by a specific type
///
/// `IndexVec` is a wrapper around `Vec<T>` that uses a typed index `I`
/// instead of `usize`. This provides compile-time type safety, preventing
/// accidental mixing of indices from different collections.
///
/// # Type Parameters
///
/// - `I`: The index type (must implement [`Idx`] trait)
/// - `T`: The element type
///
/// # Size
///
/// `IndexVec` has the same size as `Vec<T>` (24 bytes on 64-bit systems).
/// The index type `I` is a zero-cost abstraction.
///
/// # Example
///
/// ```
/// use faxc_util::index_vec::{IndexVec, Idx};
///
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// struct BlockId(u32);
///
/// impl Idx for BlockId {
///     fn from_usize(idx: usize) -> Self { BlockId(idx as u32) }
///     fn index(self) -> usize { self.0 as usize }
/// }
///
/// let mut blocks: IndexVec<BlockId, String> = IndexVec::new();
/// let id1 = blocks.push("entry".to_string());
/// let id2 = blocks.push("exit".to_string());
///
/// assert_eq!(blocks[id1], "entry");
/// assert_eq!(blocks[id2], "exit");
/// ```
#[derive(Clone)]
pub struct IndexVec<I, T> {
    raw: Vec<T>,
    _marker: PhantomData<fn(&I)>,
}

impl<I, T> IndexVec<I, T> {
    /// Create an empty IndexVec
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let vec: IndexVec<usize, i32> = IndexVec::new();
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn new() -> Self {
        Self {
            raw: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Create an IndexVec with the specified capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let vec: IndexVec<usize, i32> = IndexVec::with_capacity(10);
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            raw: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    /// Returns the number of elements in the vector
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.len(), 2);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Returns true if the vector contains no elements
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let vec: IndexVec<usize, i32> = IndexVec::new();
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns the total capacity of the vector
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let vec: IndexVec<usize, i32> = IndexVec::with_capacity(10);
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.raw.capacity()
    }

    /// Reserve capacity for at least `additional` more elements
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let mut vec: IndexVec<usize, i32> = IndexVec::new();
    /// vec.reserve(10);
    /// assert!(vec.capacity() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.raw.reserve(additional)
    }

    /// Clear all elements from the vector, keeping the allocated capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(1);
    /// vec.clear();
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.raw.clear()
    }

    /// Get a slice view of the underlying data
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let mut vec: IndexVec<usize, i32> = IndexVec::new();
    /// vec.push(1);
    /// vec.push(2);
    /// assert_eq!(vec.as_slice(), &[1, 2]);
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        &self.raw
    }

    /// Get a mutable slice view of the underlying data
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::IndexVec;
    ///
    /// let mut vec: IndexVec<usize, i32> = IndexVec::new();
    /// vec.push(1);
    /// vec.as_mut_slice()[0] = 42;
    /// assert_eq!(vec.as_slice(), &[42]);
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.raw
    }
}

impl<I: Idx, T> IndexVec<I, T> {
    /// Push an element and return its typed index
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// let idx = vec.push(42);
    /// assert_eq!(vec[idx], 42);
    /// ```
    #[inline]
    pub fn push(&mut self, value: T) -> I {
        let index = self.raw.len();
        self.raw.push(value);
        I::from_usize(index)
    }

    /// Pop the last element and return it with its index
    ///
    /// Returns `None` if the vector is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(42);
    /// let (idx, val) = vec.pop().unwrap();
    /// assert_eq!(val, 42);
    /// assert!(vec.is_empty());
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<(I, T)> {
        self.raw.pop().map(|v| {
            let idx = I::from_usize(self.raw.len());
            (idx, v)
        })
    }

    /// Get a reference to the element at the given index
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// let idx = vec.push(42);
    /// assert_eq!(vec.get(idx), Some(&42));
    /// ```
    #[inline]
    pub fn get(&self, index: I) -> Option<&T> {
        self.raw.get(index.index())
    }

    /// Get a mutable reference to the element at the given index
    ///
    /// Returns `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// let idx = vec.push(42);
    /// *vec.get_mut(idx).unwrap() = 100;
    /// assert_eq!(vec[idx], 100);
    /// ```
    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.raw.get_mut(index.index())
    }

    /// Iterate over (index, value) pairs
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    ///
    /// for (idx, val) in vec.iter_enumerated() {
    ///     println!("{}: {}", idx.0, val);
    /// }
    /// ```
    #[inline]
    pub fn iter_enumerated(&self) -> impl Iterator<Item = (I, &T)> {
        self.raw
            .iter()
            .enumerate()
            .map(|(i, v)| (I::from_usize(i), v))
    }

    /// Iterate over indices only
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    ///
    /// let indices: Vec<_> = vec.indices().collect();
    /// assert_eq!(indices.len(), 2);
    /// ```
    #[inline]
    pub fn indices(&self) -> impl Iterator<Item = I> {
        (0..self.raw.len()).map(I::from_usize)
    }

    /// Consume the vector and iterate over (index, value) pairs
    ///
    /// This is the consuming version of `iter_enumerated`.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    ///
    /// let sum: i32 = vec.into_iter_enumerated().map(|(_, v)| v).sum();
    /// assert_eq!(sum, 30);
    /// ```
    #[inline]
    pub fn into_iter_enumerated(self) -> impl Iterator<Item = (I, T)> {
        self.raw
            .into_iter()
            .enumerate()
            .map(|(i, v)| (I::from_usize(i), v))
    }

    /// Remove an element at the given index, swapping the last element into its place
    ///
    /// This is O(1) but does not preserve ordering. Returns the removed element,
    /// or `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// let removed = vec.swap_remove(Id(1));
    /// assert_eq!(removed, Some(20));
    /// assert_eq!(vec.len(), 2);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: I) -> Option<T> {
        let idx = index.index();
        if idx < self.raw.len() {
            Some(self.raw.swap_remove(idx))
        } else {
            None
        }
    }

    /// Remove an element at the given index, shifting all subsequent elements
    ///
    /// This is O(n) but preserves ordering. Returns the removed element,
    /// or `None` if the index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// let removed = vec.remove(Id(1));
    /// assert_eq!(removed, Some(20));
    /// assert_eq!(vec.len(), 2);
    /// assert_eq!(vec[Id(1)], 30);
    /// ```
    #[inline]
    pub fn remove(&mut self, index: I) -> Option<T> {
        let idx = index.index();
        if idx < self.raw.len() {
            Some(self.raw.remove(idx))
        } else {
            None
        }
    }

    /// Truncate the vector to the first `len` elements
    ///
    /// If `len` is greater than the current length, this does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    /// vec.push(20);
    /// vec.push(30);
    ///
    /// vec.truncate(Id(2));
    /// assert_eq!(vec.len(), 2);
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: I) {
        self.raw.truncate(len.index())
    }

    /// Resize the vector to the specified length
    ///
    /// If the new length is greater than the current length, new elements
    /// are initialized with the provided value. If less, the vector is truncated.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    ///
    /// vec.resize(Id(3), 0);
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec[Id(2)], 0);
    /// ```
    #[inline]
    pub fn resize(&mut self, len: I, value: T)
    where
        T: Clone,
    {
        self.raw.resize(len.index(), value)
    }

    /// Resize the vector to the specified length using a closure
    ///
    /// If the new length is greater than the current length, new elements
    /// are created by calling the provided closure. If less, the vector is truncated.
    ///
    /// This is useful when the element type doesn't implement `Clone` or when
    /// each element needs to be initialized with a unique value.
    ///
    /// # Examples
    ///
    /// ```
    /// use faxc_util::index_vec::{IndexVec, Idx};
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// struct Id(u32);
    /// impl Idx for Id {
    ///     fn from_usize(i: usize) -> Self { Id(i as u32) }
    ///     fn index(self) -> usize { self.0 as usize }
    /// }
    ///
    /// let mut vec: IndexVec<Id, i32> = IndexVec::new();
    /// vec.push(10);
    ///
    /// let mut counter = 100;
    /// vec.resize_with(Id(3), || { counter += 1; counter });
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec[Id(1)], 101);
    /// assert_eq!(vec[Id(2)], 102);
    /// ```
    #[inline]
    pub fn resize_with<F>(&mut self, len: I, f: F)
    where
        F: FnMut() -> T,
    {
        self.raw.resize_with(len.index(), f)
    }
}

impl<I: Idx, T> Index<I> for IndexVec<I, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &T {
        &self.raw[index.index()]
    }
}

impl<I: Idx, T> IndexMut<I> for IndexVec<I, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut T {
        &mut self.raw[index.index()]
    }
}

impl<I, T> Default for IndexVec<I, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to define index types easily
///
/// This macro generates a newtype wrapper around `u32` that implements
/// the [`Idx`] trait, along with common derives for debugging and hashing.
///
/// # Examples
///
/// ```
/// use faxc_util::index_vec::{IndexVec, define_idx};
///
/// define_idx!(ExprId);
///
/// let mut vec: IndexVec<ExprId, i32> = IndexVec::new();
/// let idx = vec.push(42);
/// assert_eq!(vec[idx], 42);
/// ```
#[macro_export]
macro_rules! define_idx {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);

        impl $crate::index_vec::Idx for $name {
            fn from_usize(idx: usize) -> Self {
                assert!(idx <= u32::MAX as usize, "Index {} exceeds u32::MAX", idx);
                $name(idx as u32)
            }

            fn index(self) -> usize {
                self.0 as usize
            }
        }
    };
}

#[cfg(test)]
mod tests;
