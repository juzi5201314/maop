#![feature(box_syntax)]
#![feature(option_result_unwrap_unchecked)]
#![allow(dead_code)]

pub mod lru;
mod linked_list;
pub mod fifo;

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
