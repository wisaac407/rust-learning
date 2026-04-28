use std::{
    cell::RefCell,
    hash::{DefaultHasher, Hash, Hasher},
    mem,
    rc::{Rc, Weak},
};

const LOAD_FACTOR_UPPER_BOUND: f64 = 0.75;
const LOAD_FACTOR_LOWER_BOUND: f64 = LOAD_FACTOR_UPPER_BOUND / 2.0;
const MIN_BUCKETS: usize = 2;

struct Node<K, V> {
    value: Rc<V>,
    key: K,
    next: Option<Rc<RefCell<Node<K, V>>>>,
    prev: Option<Weak<RefCell<Node<K, V>>>>,
}

struct List<K, V> {
    head: Option<Rc<RefCell<Node<K, V>>>>,
    tail: Option<Weak<RefCell<Node<K, V>>>>,
}

impl<K, V> List<K, V> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn enqueue(&mut self, key: K, value: V) -> Weak<RefCell<Node<K, V>>> {
        let new_node = Rc::new(RefCell::new(Node {
            next: self.head.take(),
            prev: None,
            key,
            value: Rc::new(value),
        }));

        let weak_ref = Rc::downgrade(&new_node);

        // Matching against old head
        match &new_node.borrow_mut().next {
            Some(node) => {
                node.borrow_mut().prev = Some(weak_ref.clone());
            }
            None => {
                self.tail = Some(weak_ref.clone());
            }
        };
        self.head = Some(new_node);
        return weak_ref;
    }

    pub fn dequeue(&mut self) -> Option<Node<K, V>> {
        let tail_rc = self.tail.clone()?.upgrade()?;
        let new_tail = tail_rc.borrow().prev.clone();

        new_tail
            .clone()
            .and_then(|tail| tail.upgrade())
            .map(|ref_| {
                let mut node = ref_.borrow_mut();
                node.next = None;
            });

        self.tail = new_tail;

        if let Some(ptr) = &self.head {
            if Rc::ptr_eq(ptr, &tail_rc) {
                self.head.take();
            }
        }

        Rc::try_unwrap(tail_rc).ok().map(RefCell::into_inner)
    }

    /// Moves a given node to the front of the queue
    pub fn touch_node(&mut self, node: Rc<RefCell<Node<K, V>>>) {
        if let Some(head) = &self.head {
            if Rc::ptr_eq(head, &node) {
                return;
            }
        }

        let tail_ref = self.tail.clone().and_then(|tail| tail.upgrade());

        if let Some(tail) = tail_ref {
            if Rc::ptr_eq(&tail, &node) {
                // Note: We will update the tail next pointer below when we set node.prev.next = node.next.
                // If node is already the tail, node.next will be None
                self.tail = node.borrow().prev.clone();
            }
        }

        // At this point, the head isn't empty, and the node isn't the head. We can move the node
        let mut node_ = node.borrow_mut();

        if let Some(next) = node_.next.clone() {
            next.borrow_mut().prev = node_.prev.clone();
        }

        let prev_ref = node_.prev.clone().and_then(|node| node.upgrade());
        if let Some(prev) = prev_ref {
            prev.borrow_mut().next = node_.next.clone();
        }

        node_.next = self.head.take().map(|head| {
            head.borrow_mut().prev = Some(Rc::downgrade(&node));
            head
        });
        node_.prev = None;
        self.head = Some(node.clone());
    }

    pub fn remove_node(
        &mut self,
        node: Rc<RefCell<Node<K, V>>>,
    ) -> Result<Node<K, V>, &'static str> {
        {
            if let Some(head) = &self.head {
                if Rc::ptr_eq(head, &node) {
                    self.head = node.borrow().next.clone();
                }
            }

            let tail_ref = self.tail.clone().and_then(|tail| tail.upgrade());

            if let Some(tail) = tail_ref {
                if Rc::ptr_eq(&tail, &node) {
                    // Note: We will update the tail next pointer below when we set node.prev.next = node.next.
                    // If node is already the tail, node.next will be None
                    self.tail = node.borrow().prev.clone();
                }
            }

            // At this point, the head isn't empty, and the node isn't the head. We can remove the node
            let node_ = node.borrow_mut();

            if let Some(next) = node_.next.clone() {
                next.borrow_mut().prev = node_.prev.clone();
            }

            let prev_ref = node_.prev.clone().and_then(|node| node.upgrade());
            if let Some(prev) = prev_ref {
                prev.borrow_mut().next = node_.next.clone();
            }
        }

        // The node will either be owned by 1. the head, or 2. the previous node (through node.next).
        // At this point, both Rc's should be cleared up, so the only remaining strong reference should be from the arguments
        match Rc::try_unwrap(node) {
            Err(_) => Err("Too many strong references to node"),
            Ok(result) => Ok(RefCell::into_inner(result)),
        }
    }
}

struct LRUCache<K: Hash + Eq, V> {
    buckets: Vec<Vec<Weak<RefCell<Node<K, V>>>>>,
    list: List<K, V>,
    item_count: usize,
    max_size: usize,
}

impl<K: Hash + Eq, V> LRUCache<K, V> {
    pub fn new(max_size: usize) -> Self {
        let mut buckets = Vec::new();
        for _ in 0..2 {
            buckets.push(Vec::new());
        }

        LRUCache {
            buckets, // Start with just two buckets. We'll resize as needed
            list: List::new(),
            item_count: 0,
            max_size,
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

    pub fn get(&mut self, key: &K) -> Option<Rc<V>> {
        let hash = self.calculate_hash(key);
        let index = self.get_index(&hash);

        let bucket = &self.buckets[index];

        for item_ref in bucket {
            if let Some(node) = item_ref.upgrade() {
                if node.borrow().key == *key {
                    self.list.touch_node(node.clone());
                    return Some(node.borrow().value.clone());
                }
            }
        }

        None
    }

    pub fn set(&mut self, key: K, value: V) {
        let hash = self.calculate_hash(&key);
        let index = self.get_index(&hash);

        let bucket = &mut self.buckets[index];

        match bucket.iter().find(|node_ref| {
            match node_ref.upgrade().map(|node| node.borrow().key == key) {
                Some(val) => val,
                None => false,
            }
        }) {
            Some(node) => {
                if let Some(node_ref) = node.upgrade() {
                    node_ref.borrow_mut().value = Rc::new(value);
                    self.list.touch_node(node_ref);
                }
            }
            None => {
                bucket.push(self.list.enqueue(key, value));
                self.item_count += 1;
            }
        }

        while self.len() > self.max_size {
            if let Some(node) = self.list.dequeue() {
                let hash = self.calculate_hash(&node.key);
                let index = self.get_index(&hash);
                self.buckets[index].retain(|ref_| ref_.upgrade().is_some());

                // The list owns the items. Dequeuing will remove the items
                self.item_count -= 1;
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
                match item.upgrade() {
                    Some(node_ref) => {
                        let node = node_ref.borrow();
                        let index = self.get_index(&self.calculate_hash(&node.key));
                        self.buckets[index].push(item);
                    }
                    None => {
                        self.item_count -= 1;
                    }
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.item_count
    }

    pub fn pop(&mut self, key: &K) -> Option<Rc<V>> {
        let hash = self.calculate_hash(key);
        let index = self.get_index(&hash);
        let bucket = &mut self.buckets[index];

        let pos = bucket
            .iter()
            .map(|node_ref| node_ref.upgrade())
            .position(|node_ref| {
                if let Some(node) = node_ref {
                    node.borrow().key == *key
                } else {
                    false
                }
            })?;

        let node = bucket.remove(pos);
        if let Some(node) = node.upgrade() {
            self.item_count -= 1;

            self.rebalance();
            self.list.remove_node(node).ok().map(|node| node.value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut list = List::new();
        list.enqueue(1, 1);
        list.enqueue(2, 2);
        list.enqueue(3, 3);

        assert_eq!(list.dequeue().unwrap().value, Rc::new(1));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(2));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(3));
    }

    #[test]
    fn test_touch() {
        let mut list = List::new();
        list.enqueue(1, 1);
        list.enqueue(2, 2);

        let node = list.head.clone();

        list.enqueue(3, 3);

        assert!(node.is_some());

        list.touch_node(node.unwrap());

        assert_eq!(list.dequeue().unwrap().value, Rc::new(1));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(3));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(2));
        assert!(list.dequeue().is_none());
    }

    #[test]
    fn test_touch_single_node() {
        let mut list = List::new();
        list.enqueue(1, 1);
        list.touch_node(list.head.clone().unwrap());

        assert_eq!(
            list.tail.clone().unwrap().upgrade().unwrap().borrow().value,
            Rc::new(1)
        );
        assert_eq!(list.dequeue().unwrap().value, Rc::new(1));
    }

    #[test]
    fn test_touch_tail_node() {
        let mut list = List::new();
        list.enqueue(1, 1);
        list.enqueue(2, 2);
        list.enqueue(3, 3);

        list.touch_node(list.tail.clone().unwrap().upgrade().unwrap());

        assert_eq!(list.dequeue().unwrap().value, Rc::new(2));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(3));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(1));
        assert!(list.dequeue().is_none());
    }

    #[test]
    fn test_remove_node() {
        let mut list = List::new();
        list.enqueue(1, 1);
        list.enqueue(2, 2);
        list.enqueue(3, 3);

        match list.remove_node(list.head.clone().unwrap()) {
            Ok(_) => {}
            Err(err) => panic!("{}", err),
        }

        assert_eq!(list.dequeue().unwrap().value, Rc::new(1));
        assert_eq!(list.dequeue().unwrap().value, Rc::new(2));
        assert!(list.dequeue().is_none());
    }

    #[test]
    fn test_cache_get_set() {
        let mut cache = LRUCache::new(5);

        cache.set("one", 1);
        cache.set("two", 2);
        cache.set("three", 3);

        assert_eq!(cache.get(&"one"), Some(Rc::new(1)));
        assert_eq!(cache.get(&"two"), Some(Rc::new(2)));
        assert_eq!(cache.get(&"three"), Some(Rc::new(3)));
        assert_eq!(cache.get(&"does not exist"), None);
    }

    #[test]
    fn test_cache_has_upper_limit() {
        let mut cache = LRUCache::new(2);

        cache.set("one", 1);
        cache.set("two", 2);
        cache.set("three", 3);

        assert_eq!(cache.get(&"two"), Some(Rc::new(2)));
        assert_eq!(cache.get(&"three"), Some(Rc::new(3)));

        assert_eq!(cache.get(&"one"), None);
    }

    #[test]
    fn test_get_touches_nodes() {
        let mut cache = LRUCache::new(2);

        cache.set("one", 1);
        cache.set("two", 2);

        assert_eq!(cache.get(&"one"), Some(Rc::new(1)));

        cache.set("three", 3);

        assert_eq!(cache.get(&"one"), Some(Rc::new(1)));
        assert_eq!(cache.get(&"three"), Some(Rc::new(3)));

        assert_eq!(cache.get(&"two"), None);
    }

    #[test]
    fn test_set_touches_nodes() {
        let mut cache = LRUCache::new(2);

        cache.set("one", 1);
        cache.set("two", 2);

        cache.set("one", 10);

        cache.set("three", 3);

        assert_eq!(cache.get(&"one"), Some(Rc::new(10)));
        assert_eq!(cache.get(&"three"), Some(Rc::new(3)));

        assert_eq!(cache.get(&"two"), None);
    }
}
