//! Basic hashmap implementation that supports get, set, and pop operations.
//! It uses buckets to avoid key collisions.
//!
//! It tracks the load factor and will rebalance the hashmap if the load factor exceeds a threshold of 0.75

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    mem,
};

const LOAD_FACTOR_UPPER_BOUND: f64 = 0.75;
// const LOAD_FACTOR_UPPER_BOUND: f64 = 100.0;
const LOAD_FACTOR_LOWER_BOUND: f64 = LOAD_FACTOR_UPPER_BOUND / 2.0;
const MIN_BUCKETS: usize = 2;

struct Item<K: Hash + Eq, V> {
    key: K,
    value: V,
    hash: usize,
}

struct HashMap<K: Hash + Eq, V> {
    buckets: Vec<Vec<Item<K, V>>>,
    item_count: usize,
}

impl<K: Hash + Eq, V> HashMap<K, V> {
    pub fn new() -> Self {
        let mut buckets = Vec::new();
        for _ in 0..2 {
            buckets.push(Vec::new());
        }

        HashMap {
            buckets, // Start with just two buckets. We'll resize as needed
            item_count: 0,
        }
    }

    fn calculate_hash(&self, key: &K) -> usize {
        let mut s = DefaultHasher::new();
        key.hash(&mut s);
        s.finish() as usize
    }

    fn get_index(&self, hash: &usize) -> usize {
        hash % self.buckets.len()
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let hash = self.calculate_hash(key);
        let index = self.get_index(&hash);

        let bucket = &self.buckets[index];

        for item in bucket {
            if item.key == *key {
                return Some(&item.value);
            }
        }

        None
    }

    pub fn set(&mut self, key: K, value: V) {
        let hash = self.calculate_hash(&key);
        let index = self.get_index(&hash);

        let bucket = &mut self.buckets[index];

        match bucket.iter_mut().find(|item| item.key == key) {
            Some(item) => item.value = value,
            None => {
                bucket.push(Item { key, value, hash });
                self.item_count += 1;
            }
        }

        self.rebalance();
    }

    fn rebalance(&mut self) {
        let load_factor = self.item_count as f64 / self.buckets.len() as f64;

        let new_size = if load_factor > LOAD_FACTOR_UPPER_BOUND {
            self.buckets.len() * 2
        } else if load_factor < LOAD_FACTOR_LOWER_BOUND {
            std::cmp::max(self.buckets.len() / 2, MIN_BUCKETS)
        } else {
            return;
        };

        let mut new_buckets = Vec::with_capacity(new_size);
        for _ in 0..new_size {
            new_buckets.push(Vec::new());
        }
        let buckets = mem::replace(&mut self.buckets, new_buckets);

        for mut bucket in buckets {
            for item in bucket.drain(..) {
                // let index = self.get_index(&item.key);
                let index = item.hash % new_size;
                self.buckets[index].push(item);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.item_count
    }

    pub fn pop(&mut self, key: &K) -> Option<V> {
        let hash = self.calculate_hash(key);
        let index = self.get_index(&hash);
        let bucket = &mut self.buckets[index];

        let pos = bucket.iter().position(|item| item.key == *key)?;

        let item = bucket.remove(pos);
        self.item_count -= 1;

        self.rebalance();

        Some(item.value)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    #[test]
    fn test_get_set() {
        let mut map = HashMap::new();

        let key = "one";

        map.set(key, 1);
        map.set("two", 2);

        assert_eq!(map.get(&"one"), Some(&1));
        assert_eq!(map.get(&"two"), Some(&2));
        assert_eq!(map.get(&"three"), None);

        map.set("three", 3);
        assert_eq!(map.get(&"three"), Some(&3));
    }

    #[test]
    fn test_pop() {
        let mut map = HashMap::new();

        map.set("one", 1);
        map.set("two", 2);

        assert_eq!(map.get(&"one"), Some(&1));
        assert_eq!(map.get(&"two"), Some(&2));

        let value = map.pop(&"one");
        assert_eq!(value, Some(1));

        assert_eq!(map.get(&"one"), None);
        assert_eq!(map.pop(&"one"), None);
    }

    #[test]
    fn test_ownership() {
        let mut map = HashMap::new();

        let key = "test";

        map.set(key, 1);

        assert_eq!(map.get(&key), Some(&1));
    }

    #[test]
    fn test_rebalance() {
        let mut map = HashMap::new();

        map.set("one", 1);
        map.set("two", 2);
        map.set("three", 3);
        map.set("four", 4);
        map.set("five", 5);
        map.set("six", 6);
        map.set("seven", 7);
        map.set("eight", 8);

        assert_eq!(map.buckets.len(), 16);

        map.pop(&"five");
        map.pop(&"six");
        map.pop(&"seven");
        map.pop(&"eight");

        assert_eq!(map.buckets.len(), 8);
    }

    #[test]
    fn test_downsize_minimum() {
        let mut map = HashMap::new();

        map.set("one", 1);
        map.set("two", 2);
        map.set("three", 3);
        map.set("four", 4);

        assert_eq!(map.buckets.len(), 8);

        map.pop(&"one");
        map.pop(&"two");
        assert_eq!(map.buckets.len(), 4);
        map.pop(&"three");

        assert_eq!(map.buckets.len(), 2);
        map.pop(&"four");

        assert_eq!(map.buckets.len(), 2);
    }

    #[test]
    fn test_insertion_speed() {
        let mut map = HashMap::new();

        let start = Instant::now();

        for i in 0..100000 {
            map.set(format!("key_{}", i), i);
        }

        println!("Insertion took: {:?}", start.elapsed());
    }

    #[test]
    fn test_get_speed() {
        let mut map = HashMap::new();

        for i in 0..100000 {
            map.set(
                String::from(format!(
                    "some_really_long_string_that_will_be_expenseive_to_compare_key_{}",
                    i
                )),
                i,
            );
        }
        let start = Instant::now();
        for i in 0..100000 {
            map.get(&String::from(format!(
                "some_really_long_string_that_will_be_expenseive_to_compare_key_{}",
                i
            )));
        }
        println!("Pop took: {:?}", start.elapsed());
    }
}
