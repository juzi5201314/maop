#![feature(box_syntax)]
#![feature(option_result_unwrap_unchecked)]

mod lru;
pub mod linked_list;
mod fifo_linked_list;

use std::hash::Hash;

/*pub struct CachePool<K, T> {

}

impl<K, T> CachePool<K, T> where K: Hash {
    pub fn new<F>(back_to_source: F) where F: Fn(K) -> T {
        todo!()
    }

    pub fn get(&self, key: K) -> Option<T> {
        todo!()
    }

    pub fn insert(&self, val: T) {

    }
}
*/
