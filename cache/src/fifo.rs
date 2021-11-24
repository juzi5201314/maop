use std::collections::VecDeque;

use crate::linked_list::LinkedList;

/*
           +--------------------------+
  back     |                          |    front
---------> |    LinkedList/VecDeque   +----------->
           |                          |
           +--------------------------+
 */
#[derive(Clone, Debug)]
pub struct FifoCache<T> {
    container: Container<T>,
    capacity: usize,
    size: usize,
}

pub enum ContainerImpl {
    LinkedList,
    VecDeque,
}

#[derive(Clone, Debug)]
enum Container<T> {
    LinkedList(LinkedList<T>),
    VecDeque(VecDeque<T>),
}

impl<T> Container<T> {
    pub fn push(&mut self, val: T) {
        match self {
            Container::LinkedList(list) => list.push_back(val),
            Container::VecDeque(deq) => deq.push_back(val),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match self {
            Container::LinkedList(list) => list.pop_front(),
            Container::VecDeque(deq) => deq.pop_front(),
        }
    }

    pub fn peek(&self) -> Option<&T> {
        match self {
            Container::LinkedList(list) => list.peek_front(),
            Container::VecDeque(deq) => deq.front(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Container::LinkedList(list) => list.is_empty(),
            Container::VecDeque(deq) => deq.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Container::LinkedList(list) => list.len(),
            Container::VecDeque(deq) => deq.len(),
        }
    }
}

impl<T> FifoCache<T> {
    pub fn new(container: ContainerImpl, capacity: usize) -> Self {
        FifoCache {
            container: match container {
                ContainerImpl::LinkedList => {
                    Container::LinkedList(LinkedList::new())
                }
                ContainerImpl::VecDeque => Container::VecDeque(
                    VecDeque::with_capacity(capacity),
                ),
            },
            capacity,
            size: 0,
        }
    }

    pub fn push(&mut self, val: T) -> Option<T> {
        let front = if self.size >= self.capacity {
            self.container.pop()
        } else {
            None
        };

        self.container.push(val);
        self.size += 1;
        front
    }

    pub fn pop(&mut self) -> Option<T> {
        self.size -= 1;
        self.container.pop()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.container.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        debug_assert!(self.size == self.container.len());
        self.size
    }
}
