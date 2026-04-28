use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::{Rc, Weak},
};

struct Node<T> {
    value: T,
    next: Option<Rc<RefCell<Node<T>>>>,
    prev: Option<Weak<RefCell<Node<T>>>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Node {
            value,
            next: None,
            prev: None,
        }
    }
}

struct List<T> {
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Weak<RefCell<Node<T>>>>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push(&mut self, value: T) {
        let new_node = Rc::new(RefCell::new(Node::new(value)));

        match &self.tail {
            Some(ref_) => match ref_.upgrade() {
                Some(node) => {
                    let mut orig = node.borrow_mut();

                    orig.next = Some(new_node.clone());
                    new_node.borrow_mut().prev = Some(Rc::downgrade(&node));
                    self.tail = Some(Rc::downgrade(&new_node));
                }
                None => {
                    // What to do here?
                }
            },
            None => {
                // No tail means we also have no head. Should we check?
                self.tail = Some(Rc::downgrade(&new_node));
                self.head = Some(new_node);
            }
        }
    }
    pub fn pop(&mut self) -> Option<T> {
        let tail_rc = self.tail.as_ref()?.upgrade()?;
        let new_tail = tail_rc.borrow().prev.clone();

        new_tail
            .clone()
            .and_then(|tail| tail.upgrade())
            .map(|ref_| {
                let mut node = ref_.borrow_mut();
                node.next = None;
            });

        self.tail = new_tail;

        let result = Rc::try_unwrap(tail_rc).ok().map(RefCell::into_inner);

        match result {
            Some(node) => Some(node.value),
            None => {
                self.head.take();
                None
            }
        }
    }

    pub fn peek(&self) -> Option<&T> {
        None
    }

    pub fn peek_mut(&mut self) -> Option<&mut T> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_push_pop() {
        let mut list = List::new();
        list.push(1);
        list.push(3);

        assert_eq!(list.pop(), Some(3));
        list.push(4);
        assert_eq!(list.pop(), Some(4));

        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
