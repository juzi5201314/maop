use crate::linked_list::LinkedList;

/*
           +--------------------------+
  back     |                          |    front
---------> |        LinkedList        +----------->
           |                          |
           +--------------------------+
 */
pub struct FifoLinkedList<T> {
    list: LinkedList<T>,
    capacity: usize,
    size: usize
}

impl<T> FifoLinkedList<T> {
    pub fn new(capacity: usize) -> Self {
        FifoLinkedList {
            list: LinkedList::new(),
            capacity,
            size: 0
        }
    }

    pub fn push(&mut self, val: T) -> Option<T> {
        let front = if self.size >= self.capacity {
            self.list.pop_front()
        } else {
            None
        };

        self.list.push_back(val);
        self.size += 1;
        front
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.size -= 1;
        self.list.pop_front()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        debug_assert!(self.size == self.list.len());
        self.size
    }

}
