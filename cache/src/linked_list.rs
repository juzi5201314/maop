use std::fmt::{Debug, Formatter};
use std::iter::FromIterator;

use itertools::Itertools;

type NodePtr<T> = Option<*mut Node<T>>;

pub struct LinkedList<T> {
    head: NodePtr<T>,
    tail: NodePtr<T>,
}

pub struct Node<T> {
    next: NodePtr<T>,
    last: NodePtr<T>,
    val: T,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, val: T) {
        let node = to_ptr(val);

        if let Some(head) = &mut self.head {
            unsafe {
                (*node.unwrap_unchecked()).next = Some(*head);
                (**head).last = node;
            }
        }
        if self.tail.is_none() {
            self.tail = node;
        }
        self.head = node;
    }

    pub fn push_back(&mut self, val: T) {
        let node = to_ptr(val);

        if let Some(tail) = &mut self.tail {
            unsafe {
                (*node.unwrap_unchecked()).last = Some(*tail);
                (**tail).next = node;
            }
        }
        if self.head.is_none() {
            self.head = node;
        }
        self.tail = node;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            let head = Box::from_raw(self.head.take()?);
            self.head = head.next;
            if let Some(head) = self.head {
                (*head).last = None;
            } else {
                self.tail = None;
            }
            Some(head.val)
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        let tail = self.tail.take()?;
        unsafe {
            self.tail = (*tail).last;
            if let Some(tail) = self.tail {
                (*tail).next = None;
            } else {
                self.head = None;
            }
            Some(Box::from_raw(tail).val)
        }
    }

    pub fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| unsafe { &(**node).val })
    }

    pub fn peek_back(&self) -> Option<&T> {
        self.tail.as_ref().map(|node| unsafe { &(**node).val })
    }

    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        self.head.as_ref().map(|node| unsafe { &mut (**node).val })
    }

    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        self.tail.as_ref().map(|node| unsafe { &mut (**node).val })
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        let mut head = self.head;
        for _ in 0..index {
            if let Some(node) = head {
                unsafe {
                    head = (*node).next;
                }
            } else {
                return None;
            }
        }
        head.map(|node| unsafe {
            let last = (*node).last;
            let next = (*node).next;
            if let Some(last) = last {
                (*last).next = next;
            } else {
                self.head = None;
            }
            if let Some(next) = next {
                (*next).last = last;
            } else {
                self.tail = None;
            }
            Box::from_raw(node).val
        })
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        let mut iter = self.iter();
        for _ in 0..index {
            iter.next()?;
        }
        iter.next()
    }

    pub fn len(&self) -> usize {
        let mut l = 0;
        let mut iter = self.iter();
        while iter.next().is_some() {
            l += 1;
        }
        l
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn iter(&self) -> Iter<T> {
        Iter { head: &self.head }
    }
}

fn to_ptr<T>(val: T) -> NodePtr<T> {
    Some(Box::into_raw(box Node {
        next: None,
        last: None,
        val,
    }))
}

pub struct IntoIter<T> {
    head: NodePtr<T>,
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(mut self) -> Self::IntoIter {
        IntoIter {
            head: self.head.take(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.head.take();
        node.map(|node| unsafe {
            self.head = (*node).next;
            Box::from_raw(node).val
        })
    }
}

pub struct Iter<'a, T: 'a> {
    head: &'a NodePtr<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.head.map(|node| unsafe {
            self.head = &(*node).next;
            &(*node).val
        })
    }
}

impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<ITER: IntoIterator<Item = T>>(iter: ITER) -> Self {
        let mut list = LinkedList::new();
        iter.into_iter().for_each(|val| list.push_back(val));
        list
    }
}

impl<T> Extend<T> for LinkedList<T> {
    fn extend<ITER: IntoIterator<Item = T>>(&mut self, iter: ITER) {
        iter.into_iter().for_each(|val| self.push_back(val));
    }
}

impl<T> Clone for LinkedList<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        self.iter().map(Clone::clone).collect()
    }
}

impl<T> Debug for LinkedList<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        write!(
            f,
            "{}",
            self.iter().map(|val| format!("{:?}", val)).join(", ")
        )?;
        write!(f, "]")
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        loop {
            if let Some(head) = self.head {
                unsafe {
                    self.head = (*head).next;
                    if let Some(head) = self.head {
                        (*head).last = None;
                    }
                    Box::from_raw(head);
                }
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::linked_list::LinkedList;

    #[test]
    fn push() {
        let mut list = LinkedList::new();
        list.push_back(5);
        list.push_front(3);
        list.push_back(7);
        list.push_front(1);
        list.push_back(9);
        assert_eq!(
            list.into_iter().collect::<Vec<_>>(),
            vec![1, 3, 5, 7, 9]
        );
    }

    #[test]
    fn pop() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert!(!list.is_empty());
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn peek() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        assert_eq!(list.peek_front(), Some(&1));
        *list.peek_front_mut().unwrap() = 3;
        assert_eq!(list.peek_front(), Some(&3));
        assert_eq!(list.peek_back(), Some(&2));
    }

    #[test]
    fn index() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(list.get(1), Some(&2));
        assert_eq!(list.get(2), Some(&3));
        assert_eq!(list.get(3), None);
        assert_eq!(list.get(4), None);
    }

    #[test]
    fn remove() {
        let mut list = LinkedList::new();
        list.push_back(1);
        list.push_back(2);
        assert_eq!(list.remove(1), Some(2));
        assert_eq!(list.get(1), None);
        assert_eq!(list.remove(0), Some(1));
    }
}
