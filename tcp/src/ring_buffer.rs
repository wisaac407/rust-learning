use std::path::Iter;

/// FIFO queue implemented as a ring buffer
use anyhow::{ensure, Result};

/// FIFO queue implemented with as a ring buffer
pub struct RingBuffer<T, const COUNT: usize> {
    start: usize,
    count: usize,
    buffer: [T; COUNT],
}

impl<T: Copy + Default, const COUNT: usize> RingBuffer<T, COUNT> {
    /// Construct a new blank ring buffer
    pub fn new() -> Self {
        assert!(COUNT > 0, "Buffer size must be greater than 0");

        RingBuffer {
            start: 0,
            count: 0,
            buffer: [T::default(); COUNT],
        }
    }

    /// Returns the number of items in the queue
    pub fn get_count(&self) -> usize {
        self.count
    }

    /// Return true if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Return true if the buffer is full
    pub fn is_full(&self) -> bool {
        assert!(
            self.count <= self.buffer.len(),
            "Buffer count is greater than buffer length!"
        );

        self.count == self.buffer.len()
    }

    /// Adds an item to the buffer. Returns an error if the buffer is already full
    pub fn put(&mut self, item: T) -> Result<()> {
        ensure!(!self.is_full(), "Buffer already full!");

        let index = (self.start + self.count) % (self.buffer.len());
        self.count += 1;
        self.buffer[index] = item;

        Ok(())
    }

    /// Returns the first item from the buffer or None if the buffer is empty
    pub fn get(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let index = self.start;
            self.start = (self.start + 1) % self.buffer.len();
            self.count -= 1;

            Some(self.buffer[index])
        }
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: Fn(&T) -> bool,
    {
        let mut offset = 0;

        for virtual_index in 0..self.count {
            let index = (self.start + virtual_index) % self.count;
            if predicate(&self.buffer[index]) {
                if offset > 0 {
                    // Equivilent to (index - offset) % self.count but guarantees no intermediate negatives
                    let new_index = (index + self.count - (offset % self.count)) % self.count;

                    self.buffer[new_index] = self.buffer[index];
                    self.buffer[index] = T::default();
                }
            } else {
                self.buffer[index] = T::default();
                offset += 1;
            }
        }
        self.count -= offset;
    }

    pub fn for_each<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut T),
    {
        for virtual_index in 0..self.count {
            let index = (self.start + virtual_index) % self.count;
            func(&mut self.buffer[index]);
        }
    }
}

pub struct RingBufferIterInto<T, const N: usize> {
    buffer: RingBuffer<T, N>,
    virtual_index: usize,
}

pub struct RingBufferIterRef<'a, T, const N: usize> {
    buffer: &'a RingBuffer<T, N>,
    virtual_index: usize,
}

impl<'a, T, const N: usize> Iterator for RingBufferIterInto<T, N>
where
    T: Copy + Default,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.virtual_index >= self.buffer.count {
            None
        } else {
            let index = (self.virtual_index + self.buffer.start) % self.buffer.count;
            self.virtual_index += 1;

            Some(self.buffer.buffer[index])
        }
    }
}

impl<'a, T, const N: usize> Iterator for RingBufferIterRef<'a, T, N>
where
    T: Copy + Default,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.virtual_index >= self.buffer.count {
            None
        } else {
            let index = (self.virtual_index + self.buffer.start) % self.buffer.count;
            self.virtual_index += 1;

            Some(&self.buffer.buffer[index])
        }
    }
}

impl<T, const N: usize> IntoIterator for RingBuffer<T, N>
where
    T: Copy + Default,
{
    type Item = T;

    type IntoIter = RingBufferIterInto<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            buffer: self,
            virtual_index: 0,
        }
    }
}

impl<'a, T: Copy + Default, const N: usize> RingBuffer<T, N> {
    fn iter(&'a self) -> RingBufferIterRef<'a, T, N> {
        RingBufferIterRef {
            buffer: self,
            virtual_index: 0,
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a RingBuffer<T, N>
where
    T: Copy + Default,
{
    type Item = &'a T;

    type IntoIter = RingBufferIterRef<'a, T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_get() {
        let mut queue: RingBuffer<i32, 10> = RingBuffer::new();
        queue.put(42).unwrap();
        queue.put(57).unwrap();
        assert_eq!(queue.get(), Some(42));
        queue.put(38).unwrap();
        assert_eq!(queue.get(), Some(57));
        assert_eq!(queue.get(), Some(38));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_full_capacity() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();

        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        assert_eq!(queue.get(), Some(1));
        assert_eq!(queue.get(), Some(2));
        assert_eq!(queue.get(), Some(3));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_put_wrap() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(42).unwrap();
        queue.put(57).unwrap();
        assert_eq!(queue.get(), Some(42));
        assert_eq!(queue.get(), Some(57));

        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        assert_eq!(queue.get(), Some(1));
        assert_eq!(queue.get(), Some(2));
        assert_eq!(queue.get(), Some(3));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_buffer_overflow() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        assert!(queue.put(4).is_err());

        assert_eq!(queue.get(), Some(1));
        assert_eq!(queue.get(), Some(2));
        assert_eq!(queue.get(), Some(3));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_is_empty_and_is_full() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        assert!(queue.is_empty());
        queue.put(1).unwrap();
        assert!(!queue.is_empty());
        assert!(!queue.is_full());
        queue.put(2).unwrap();
        queue.put(3).unwrap();
        assert!(queue.is_full());
    }

    #[test]
    fn test_retain() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        queue.retain(|item| *item > 1);

        assert_eq!(queue.get(), Some(2));
        assert_eq!(queue.get(), Some(3));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_retain_wrap() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        assert_eq!(queue.get(), Some(1));

        queue.put(11).unwrap();
        queue.put(22).unwrap();

        queue.retain(|item| *item > 10);

        assert_eq!(queue.get(), Some(11));
        assert_eq!(queue.get(), Some(22));
        assert_eq!(queue.get(), None);
    }

    #[test]
    fn test_iter_into() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        let mut vec = Vec::new();

        for item in queue {
            vec.push(item);
        }

        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter_into_wrapped() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        assert_eq!(queue.get(), Some(1));

        queue.put(3).unwrap();
        queue.put(4).unwrap();

        let mut vec = Vec::new();

        for item in queue {
            vec.push(item);
        }

        assert_eq!(vec, vec![2, 3, 4]);
    }

    #[test]
    fn test_iter_ref() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        let mut iter = queue.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));

        let mut vec = Vec::new();

        for item in &queue {
            vec.push(*item);
        }

        assert_eq!(vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_iter_ref_wrapped() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        assert_eq!(queue.get(), Some(1));

        queue.put(3).unwrap();
        queue.put(4).unwrap();

        let mut vec = Vec::new();

        for item in &queue {
            vec.push(*item);
        }

        assert_eq!(vec, vec![2, 3, 4]);
    }

    #[test]
    fn test_for_each() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        queue.put(3).unwrap();

        queue.for_each(|item| {
            *item += 1;
        });

        assert_eq!(Vec::from_iter(queue), vec![2, 3, 4]);
    }

    #[test]
    fn test_for_each_wrapped() {
        let mut queue: RingBuffer<i32, 3> = RingBuffer::new();
        queue.put(1).unwrap();
        queue.put(2).unwrap();
        assert_eq!(queue.get(), Some(1));

        queue.put(3).unwrap();
        queue.put(4).unwrap();

        queue.for_each(|item| {
            *item += 1;
        });

        assert_eq!(Vec::from_iter(queue), vec![3, 4, 5]);
    }
}
