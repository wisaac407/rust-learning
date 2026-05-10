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
}
