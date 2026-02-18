//! Tests for the IndexVec module.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TestId(u32);

impl Idx for TestId {
    fn from_usize(idx: usize) -> Self {
        assert!(idx <= u32::MAX as usize);
        TestId(idx as u32)
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

// ============================================================================
// BASIC OPERATIONS
// ============================================================================

#[test]
fn test_new_and_empty() {
    let vec: IndexVec<TestId, i32> = IndexVec::new();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 0);
}

#[test]
fn test_with_capacity() {
    let vec: IndexVec<TestId, i32> = IndexVec::with_capacity(10);
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
    assert_eq!(vec.capacity(), 10);
}

#[test]
fn test_push_and_index() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    let idx1 = vec.push(10);
    let idx2 = vec.push(20);
    let idx3 = vec.push(30);

    assert_eq!(vec[idx1], 10);
    assert_eq!(vec[idx2], 20);
    assert_eq!(vec[idx3], 30);
    assert_eq!(vec.len(), 3);
}

#[test]
fn test_pop() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);

    let (idx, val) = vec.pop().unwrap();
    assert_eq!(val, 20);
    assert_eq!(idx, TestId(1));
    assert_eq!(vec.len(), 1);

    let (idx, val) = vec.pop().unwrap();
    assert_eq!(val, 10);
    assert_eq!(idx, TestId(0));
    assert_eq!(vec.len(), 0);

    assert!(vec.pop().is_none());
}

#[test]
fn test_get_and_get_mut() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    let idx = vec.push(42);

    assert_eq!(vec.get(idx), Some(&42));
    assert_eq!(vec.get(TestId(100)), None);

    *vec.get_mut(idx).unwrap() = 100;
    assert_eq!(vec[idx], 100);
    assert_eq!(vec.get_mut(TestId(100)), None);
}

#[test]
fn test_as_slice() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    assert_eq!(vec.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_as_mut_slice() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(1);
    vec.push(2);

    vec.as_mut_slice()[0] = 10;
    vec.as_mut_slice()[1] = 20;

    assert_eq!(vec.as_slice(), &[10, 20]);
}

#[test]
fn test_reserve() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.reserve(100);
    assert!(vec.capacity() >= 100);
}

#[test]
fn test_clear() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(1);
    vec.push(2);

    vec.clear();
    assert!(vec.is_empty());
    assert_eq!(vec.len(), 0);
}

// ============================================================================
// ITERATION
// ============================================================================

#[test]
fn test_iter_enumerated() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let items: Vec<_> = vec.iter_enumerated().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], (TestId(0), &10));
    assert_eq!(items[1], (TestId(1), &20));
    assert_eq!(items[2], (TestId(2), &30));
}

#[test]
fn test_indices() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);

    let indices: Vec<_> = vec.indices().collect();
    assert_eq!(indices, vec![TestId(0), TestId(1)]);
}

#[test]
fn test_into_iter_enumerated() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let sum: i32 = vec.into_iter_enumerated().map(|(_, v)| v).sum();
    assert_eq!(sum, 60);
}

// ============================================================================
// REMOVAL OPERATIONS
// ============================================================================

#[test]
fn test_swap_remove() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let removed = vec.swap_remove(TestId(1));
    assert_eq!(removed, Some(20));
    assert_eq!(vec.len(), 2);
    // Last element (30) is swapped into position 1
    assert_eq!(vec[TestId(1)], 30);
}

#[test]
fn test_swap_remove_out_of_bounds() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);

    assert_eq!(vec.swap_remove(TestId(100)), None);
}

#[test]
fn test_swap_remove_empty() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    assert_eq!(vec.swap_remove(TestId(0)), None);
}

#[test]
fn test_remove() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let removed = vec.remove(TestId(1));
    assert_eq!(removed, Some(20));
    assert_eq!(vec.len(), 2);
    // Elements after removed one are shifted
    assert_eq!(vec[TestId(1)], 30);
}

#[test]
fn test_remove_out_of_bounds() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);

    assert_eq!(vec.remove(TestId(100)), None);
}

#[test]
fn test_remove_empty() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    assert_eq!(vec.remove(TestId(0)), None);
}

// ============================================================================
// RESIZE OPERATIONS
// ============================================================================

#[test]
fn test_truncate() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    vec.truncate(TestId(2));
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[TestId(0)], 10);
    assert_eq!(vec[TestId(1)], 20);
}

#[test]
fn test_truncate_no_op() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);

    vec.truncate(TestId(5));
    assert_eq!(vec.len(), 2);
}

#[test]
fn test_truncate_to_zero() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);

    vec.truncate(TestId(0));
    assert!(vec.is_empty());
}

#[test]
fn test_resize_grow() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);

    vec.resize(TestId(3), 0);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[TestId(0)], 10);
    assert_eq!(vec[TestId(1)], 0);
    assert_eq!(vec[TestId(2)], 0);
}

#[test]
fn test_resize_shrink() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    vec.resize(TestId(2), 0);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[TestId(0)], 10);
    assert_eq!(vec[TestId(1)], 20);
}

#[test]
fn test_resize_same_size() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);

    vec.resize(TestId(2), 0);
    assert_eq!(vec.len(), 2);
    assert_eq!(vec[TestId(0)], 10);
    assert_eq!(vec[TestId(1)], 20);
}

#[test]
fn test_resize_with_grow() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);

    let mut counter = 100;
    vec.resize_with(TestId(4), || {
        counter += 1;
        counter
    });

    assert_eq!(vec.len(), 4);
    assert_eq!(vec[TestId(0)], 10);
    assert_eq!(vec[TestId(1)], 101);
    assert_eq!(vec[TestId(2)], 102);
    assert_eq!(vec[TestId(3)], 103);
}

#[test]
fn test_resize_with_shrink() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    let mut called = false;
    vec.resize_with(TestId(2), || {
        called = true;
        0
    });

    assert_eq!(vec.len(), 2);
    assert!(!called); // Closure shouldn't be called when shrinking
}

#[test]
fn test_resize_with_non_clone() {
    // Test with a type that doesn't implement Clone
    let mut vec: IndexVec<TestId, Box<i32>> = IndexVec::new();
    vec.push(Box::new(10));

    let mut counter = 100;
    vec.resize_with(TestId(3), || {
        counter += 1;
        Box::new(counter)
    });

    assert_eq!(vec.len(), 3);
    assert_eq!(*vec[TestId(0)], 10);
    assert_eq!(*vec[TestId(1)], 101);
    assert_eq!(*vec[TestId(2)], 102);
}

// ============================================================================
// INDEX TRAITS
// ============================================================================

#[test]
fn test_index_trait() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(42);

    assert_eq!(vec[TestId(0)], 42);
}

#[test]
fn test_index_mut_trait() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    vec.push(42);

    vec[TestId(0)] = 100;
    assert_eq!(vec[TestId(0)], 100);
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_index_out_of_bounds() {
    let vec: IndexVec<TestId, i32> = IndexVec::new();
    let _ = vec[TestId(0)];
}

// ============================================================================
// DEFAULT
// ============================================================================

#[test]
fn test_default() {
    let vec: IndexVec<TestId, i32> = IndexVec::default();
    assert!(vec.is_empty());
}

// ============================================================================
// DEFINE_IDX MACRO
// ============================================================================

#[test]
fn test_define_idx_macro() {
    define_idx!(MacroTestId);

    let mut vec: IndexVec<MacroTestId, i32> = IndexVec::new();
    let idx = vec.push(42);
    assert_eq!(vec[idx], 42);
    assert_eq!(idx.0, 0);
}

// ============================================================================
// THREAD SAFETY
// ============================================================================

#[test]
fn test_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<IndexVec<TestId, i32>>();
    assert_sync::<IndexVec<TestId, i32>>();
}

#[test]
fn test_concurrent_push() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let vec = Arc::new(Mutex::new(IndexVec::<TestId, i32>::new()));
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let vec = Arc::clone(&vec);
            thread::spawn(move || {
                let mut vec = vec.lock().unwrap();
                vec.push(i);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let vec = vec.lock().unwrap();
    assert_eq!(vec.len(), 10);
}

// ============================================================================
// EDGE CASES
// ============================================================================

#[test]
fn test_empty_iterators() {
    let vec: IndexVec<TestId, i32> = IndexVec::new();

    assert_eq!(vec.iter_enumerated().count(), 0);
    assert_eq!(vec.indices().count(), 0);
}

#[test]
fn test_single_element() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::new();
    let idx = vec.push(42);

    assert_eq!(vec.len(), 1);
    assert_eq!(vec[idx], 42);
    assert_eq!(vec.get(idx), Some(&42));

    let (popped_idx, popped_val) = vec.pop().unwrap();
    assert_eq!(popped_idx, idx);
    assert_eq!(popped_val, 42);
    assert!(vec.is_empty());
}

#[test]
fn test_large_indices() {
    let mut vec: IndexVec<TestId, i32> = IndexVec::with_capacity(1000);
    for i in 0..1000 {
        vec.push(i as i32);
    }

    assert_eq!(vec.len(), 1000);
    assert_eq!(vec[TestId(0)], 0);
    assert_eq!(vec[TestId(999)], 999);
}

#[test]
fn test_clone() {
    let mut vec1: IndexVec<TestId, i32> = IndexVec::new();
    vec1.push(10);
    vec1.push(20);

    let vec2 = vec1.clone();
    assert_eq!(vec2[TestId(0)], 10);
    assert_eq!(vec2[TestId(1)], 20);

    // Modify vec1, ensure vec2 is unchanged
    vec1[TestId(0)] = 100;
    assert_eq!(vec1[TestId(0)], 100);
    assert_eq!(vec2[TestId(0)], 10);
}
