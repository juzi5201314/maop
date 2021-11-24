use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::iter::FromIterator;
use std::rc::Rc;

use crate::linked_list::LinkedList;

pub struct LruCache<K, T> {
    capacity: usize,
    size: usize,
    map: HashMap<
        Rc<K>,
        *mut crate::linked_list::Node<Entry<Rc<K>, T>>,
    >,
    list: LinkedList<Entry<Rc<K>, T>>,
}

pub struct Entry<K, T> {
    pub key: K,
    pub val: T,
}

impl<K, T> LruCache<K, T>
where
    K: Hash + Eq,
{
    pub fn new(capacity: usize) -> Self {
        LruCache {
            capacity,
            size: 0,
            map: HashMap::with_capacity(capacity),
            list: LinkedList::new(),
        }
    }

    fn update(
        &mut self,
        node: *mut crate::linked_list::Node<Entry<Rc<K>, T>>,
    ) {
        unsafe {
            self.list.unlink(node);
            self.list.push_back_node(Some(node));
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<T>
    where
        Rc<K>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map
            .remove(key)
            .map(|node| {
                self.size -= 1;
                self.list.remove_node(Some(node)).map(|node| node.val)
            })
            .flatten()
    }

    pub fn insert(&mut self, key: K, val: T) -> Option<T> {
        let mut ret = None;
        #[allow(mutable_borrow_reservation_conflict)]
        if let Some(node) = self.map.get(&key) {
            self.update(*node);
            return ret;
        } else if self.size >= self.capacity {
            let ret_node = self.list.pop_front().unwrap();
            self.map.remove(&ret_node.key);
            self.size -= 1;
            ret = Some(ret_node.val)
        }

        let key = Rc::new(key);

        let node = crate::linked_list::to_ptr(Entry {
            key: Rc::clone(&key),
            val,
        });

        self.list.push_back_node(node);
        self.map.insert(key, unsafe { node.unwrap_unchecked() });
        self.size += 1;
        ret
    }

    pub fn get<Q>(&mut self, key: &Q) -> Option<&T>
    where
        Rc<K>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.get_mut(key).map(|v| &*v)
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut T>
    where
        Rc<K>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.get(key).map(|node| *node).map(|node| unsafe {
            self.update(node);
            &mut (*node).val.val
        })
    }

    pub fn peek_mut<Q>(&mut self, key: &Q) -> Option<&mut T>
    where
        Rc<K>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map
            .get(key)
            .map(|node| *node)
            .map(|node| unsafe { &mut (*node).val.val })
    }

    pub fn peek<Q>(&mut self, key: &Q) -> Option<&T>
    where
        Rc<K>: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.peek_mut(key).map(|v| &*v)
    }

    #[inline]
    pub fn peek_pop(&self) -> Option<&T> {
        self.peek_pop_entry().map(|entry| entry.val)
    }

    pub fn peek_pop_entry(&self) -> Option<Entry<&K, &T>> {
        self.list.peek_front().map(|node| Entry {
            key: &*node.key,
            val: &node.val,
        })
    }

    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        self.pop_entry().map(|entry| entry.val)
    }

    pub fn pop_entry(&mut self) -> Option<Entry<K, T>> {
        self.list.pop_front().map(|entry| {
            self.map.remove(&entry.key);
            self.size -= 1;
            Entry {
                key: unsafe {
                    Rc::try_unwrap(entry.key).unwrap_unchecked()
                },
                val: entry.val,
            }
        })
    }

    pub fn iter(&self) -> Iter<K, T> {
        Iter {
            iter: self.list.iter(),
        }
    }
}

impl<K, T> Extend<(K, T)> for LruCache<K, T>
where
    K: Hash + Eq,
{
    fn extend<I: IntoIterator<Item = (K, T)>>(&mut self, iter: I) {
        iter.into_iter().for_each(|(k, v)| {
            self.insert(k, v);
        })
    }
}

impl<K, T> Clone for LruCache<K, T>
where
    K: Clone + Hash + Eq,
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut lru = LruCache::new(self.capacity);
        lru.extend(self.iter().map(|(k, v)| (k.clone(), v.clone())));
        lru
    }
}

impl<K, T> Clone for Entry<Rc<K>, T>
where
    K: Clone + Hash + Eq,
    T: Clone,
{
    fn clone(&self) -> Self {
        Entry {
            key: Rc::new(K::clone(&self.key)),
            val: self.val.clone(),
        }
    }
}

impl<K, T> Debug for Entry<K, T>
where
    K: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entry(&self.key, &self.val).finish()
    }
}

impl<K, T> Debug for LruCache<K, T>
where
    K: Debug,
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LruCache")
            .field("capacity", &self.capacity)
            .field("size", &self.size)
            .field("content", &self.list)
            .finish()
    }
}

pub struct IntoIter<K, T> {
    lru: LruCache<K, T>,
}

impl<K, T> IntoIterator for LruCache<K, T>
where
    K: Hash + Eq,
{
    type Item = (K, T);
    type IntoIter = IntoIter<K, T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { lru: self }
    }
}

impl<K, T> Iterator for IntoIter<K, T>
where
    K: Hash + Eq,
{
    type Item = (K, T);

    fn next(&mut self) -> Option<Self::Item> {
        self.lru.pop_entry().map(Into::into)
    }
}

pub struct Iter<'a, K: 'a, T: 'a> {
    iter: crate::linked_list::Iter<'a, Entry<Rc<K>, T>>,
}

impl<'a, K, T> Iterator for Iter<'a, K, T>
where
    K: Hash + Eq,
{
    type Item = (&'a K, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|entry| (&*entry.key, &entry.val))
            .map(Into::into)
    }
}

impl<K, T> FromIterator<(K, T)> for LruCache<K, T>
where
    K: Hash + Eq,
{
    fn from_iter<I: IntoIterator<Item = (K, T)>>(iter: I) -> Self {
        let v = iter.into_iter().collect::<Vec<(K, T)>>();
        let mut lru = LruCache::new(v.len());
        v.into_iter().for_each(|(k, v)| {
            lru.insert(k, v);
        });
        lru
    }
}

impl<K, T> From<Entry<K, T>> for (K, T) {
    fn from(entry: Entry<K, T>) -> Self {
        (entry.key, entry.val)
    }
}

impl<'a, K, T> From<&'a Entry<K, T>> for (&'a K, &'a T) {
    fn from(entry: &'a Entry<K, T>) -> Self {
        (&entry.key, &entry.val)
    }
}

#[cfg(test)]
mod tests {
    use crate::lru::LruCache;

    #[test]
    fn insert_get() {
        let mut lru = LruCache::new(3);
        lru.insert(0, "a");
        lru.insert(1, "b");
        lru.insert(2, "c");
        assert_eq!(lru.peek_pop(), Some(&"a"));
        lru.get(&0);
        assert_eq!(lru.peek_pop(), Some(&"b"));
        lru.insert(1, "d");
        lru.insert(3, "e");
        assert_eq!(lru.peek_pop(), Some(&"a"));
        lru.remove(&2);
        assert_eq!(lru.peek_pop(), Some(&"a"));
    }

    #[test]
    fn peek() {
        let mut lru = LruCache::new(3);
        lru.insert(0, "a");
        lru.insert(1, "b");
        lru.insert(2, "c");
        assert_eq!(lru.peek(&0), Some(&"a"));
        assert_eq!(lru.peek_pop(), Some(&"a"));
    }
}
