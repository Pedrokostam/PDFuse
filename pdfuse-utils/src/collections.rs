use crate::Indexed;

/// Checks whether the vector is sorted and if not - sorts it by index.
pub fn ensure_sorted<T>(vector:&mut [Indexed<T>])
where
    T: Sized,
{
    vector.sort_unstable_by_key(|x| x.index());
}

/// Asserts whether the collections is sorted by index in ascending manner.
///
/// # Panics
///
/// Panics if the collection is not sorted.
pub fn assert_sorted<T>(iterable: &[Indexed<T>]) {
    assert!(iterable.is_sorted_by_key(|x| x.index()));
}

/// Creates a [`Vec<Indexed<T>>`] from the iterable, where the index is equal to the index of the
/// item in the iterable.
pub fn indexise<T>(items: &[T]) -> Vec<Indexed<T>>
where
    T: Clone,
{
    items
        .iter()
        .enumerate()
        .map(|x: (usize, &T)| x.into())
        .collect()
}
