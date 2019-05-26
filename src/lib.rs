use std::borrow::Borrow;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;

const INITIAL_NBUCKETS: usize = 1;

#[derive(Debug)]
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];

        self.items += 1;

        for &mut (ref existing_key, ref mut existing_value) in bucket.iter_mut() {
            if existing_key == &key {
                return Some(mem::replace(existing_value, value));
            }
        }

        bucket.push((key, value));
        None
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    pub fn bucket<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        return (hasher.finish() % self.buckets.len() as u64) as usize;
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket
            .iter()
            .position(|&(ref existing_key, _)| existing_key.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key);
        self.buckets[bucket]
            .iter() // Iterator<Item = &(K, V)>
            .find(|&(ref existing_key, _)| existing_key.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_NBUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some(&(ref key, ref value)) => {
                        self.at += 1;
                        break Some((key, value));
                    }

                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },

                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut hashmap = HashMap::new();
        assert_eq!(hashmap.is_empty(), true);
        hashmap.insert("foo", 42);
        assert_eq!(hashmap.is_empty(), false);
        assert_eq!(hashmap.len(), 1);
        assert_eq!(hashmap.get("foo"), Some(&42));
        assert_eq!(hashmap.contains_key(&"foo"), true);
        assert_eq!(hashmap.remove(&"foo"), Some(42));
        assert_eq!(hashmap.len(), 0);
        assert_eq!(hashmap.get(&"foo"), None);

        hashmap.insert("foo", 42);
        hashmap.insert("bar", 24);

        for (key, value) in &hashmap {
            println!("{}: {}", key, value)
        }
    }
}
