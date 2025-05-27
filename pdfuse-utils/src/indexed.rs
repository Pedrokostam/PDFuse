use std::{
    fmt,
    ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct Indexed<T> {
    index: usize,
    value: T,
}
impl<T> fmt::Display for Indexed<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Index: {}, Value: {}", self.index, self.value)
    }
}
impl<T> Clone for Indexed<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            value: self.value.clone(),
        }
    }
}
impl<T> PartialEq for Indexed<T>{
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl<T> Eq for Indexed<T>{
  
}
impl<T> Ord for Indexed<T>{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
       self.index.cmp(&other.index)
    }
}
impl<T> PartialOrd for Indexed<T>{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> Indexed<T> {
    pub fn new(index: usize, value: T) -> Self {
        Self { index, value }
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
    pub fn unwrap(self) -> T {
        self.value
    }
    pub fn map_option<U, F>(self, f: F) -> Option<Indexed<U>>
    where
        F: FnOnce(T) -> Option<U>,
    {
        let index = self.index;
        let mapped = f(self.unwrap());
        mapped.map(|value| Indexed{index,value})
    }
    pub fn map_with_index<U, F>(self, f: F) -> Indexed<U>
    where
        F: FnOnce(T) -> U,
    {
        Indexed {
            index: self.index(),
            value: f(self.unwrap()),
        }
    }
}
impl<T> From<Indexed<T>> for (usize, T) {
    fn from(indexed: Indexed<T>) -> Self {
        (indexed.index, indexed.value)
    }
}
impl<T> From<(usize, T)> for Indexed<T> {
    fn from(value: (usize, T)) -> Self {
        Self {
            index: value.0,
            value: value.1,
        }
    }
}
impl<T> Deref for Indexed<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}
impl<T> DerefMut for Indexed<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value_mut()
    }
}
