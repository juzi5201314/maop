#![feature(test)]
#![feature(linked_list_remove)]

extern crate test;

use test::{Bencher, black_box};
use std::collections::LinkedList as StdLinkedList;
use cache::linked_list::LinkedList;

#[bench]
fn push_std(b: &mut Bencher) {
    let mut list = StdLinkedList::new();
    b.iter(|| {
        list.push_back(1);
        list.push_front(2);
    })
}

#[bench]
fn push(b: &mut Bencher) {
    let mut list = LinkedList::new();
    b.iter(|| {
        list.push_back(1);
        list.push_front(2);
    })
}

#[bench]
fn pop_std(b: &mut Bencher) {
    let mut list = StdLinkedList::new();
    list.push_back(1);
    list.push_front(2);
    b.iter(|| {
        black_box(list.pop_front());
        black_box(list.pop_back());
    })
}

#[bench]
fn pop(b: &mut Bencher) {
    let mut list = LinkedList::new();
    list.push_back(1);
    list.push_front(2);
    b.iter(|| {
        black_box(list.pop_front());
        black_box(list.pop_back());
    })
}
