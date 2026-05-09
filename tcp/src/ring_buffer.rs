/// FIFO queue implemented as a ring buffer
use anyhow::{ensure, Result};

pub struct RingBuffer<T, const COUNT: usize> {
    start: usize,
    count: usize,
    buffer: [T; COUNT],
}

impl<T: Copy + Default, const COUNT: usize> RingBuffer<T, COUNT> {
    pub fn new() -> Self {
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

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == self.buffer.len()
    }

    pub fn put(&mut self, item: T) -> Result<()> {
        ensure!(self.count < self.buffer.len(), "Buffer already full!");

        let index = (self.start + self.count) % (self.buffer.len());
        self.count += 1;
        self.buffer[index] = item;

        Ok(())
    }

    pub fn get(&mut self) -> Option<T> {
        if self.count == 0 {
            None
        } else {
            let index = self.start;
            self.start = (self.start + 1) % self.buffer.len();
            self.count -= 1;

            Some(self.buffer[index])
        }
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
}
