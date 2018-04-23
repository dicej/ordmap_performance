#![feature(test)]

extern crate im;
extern crate rand;
extern crate test;

use std::iter;
use im::OrdMap;
use im::nodes::btree::{Insert, Node, OrdValue, Remove};
use rand::{Rng, SeedableRng, StdRng};
use test::Bencher;

#[derive(Clone)]
struct Raw<K, V>(K, V);

impl<K: Ord + Clone, V: Eq + Clone> OrdValue for Raw<K, V> {
    type Key = K;

    fn extract_key(&self) -> &K {
        &self.0
    }

    fn ptr_eq(&self, other: &Self) -> bool {
        self.1 == other.1 && self.0 == other.0
    }
}

#[derive(Clone)]
pub struct RawMap<K, V> {
    root: Node<Raw<K, V>>,
}

impl<K: Ord + Clone, V: Eq + Clone> RawMap<K, V> {
    fn new() -> Self {
        RawMap { root: Node::new() }
    }

    fn insert(&self, k: K, v: V) -> Self {
        match self.root.insert(Raw(k, v)) {
            Insert::NoChange => self.clone(),
            Insert::JustInc => unreachable!(),
            Insert::Update(root) => RawMap { root },
            Insert::Split(left, median, right) => RawMap {
                root: Node::from_split(left, median, right),
            },
        }
    }

    fn remove(&self, k: &K) -> Self {
        match self.root.remove(k) {
            Remove::NoChange => self.clone(),
            Remove::Removed(_) => unreachable!(),
            Remove::Update(_, root) => RawMap { root },
        }
    }

    fn insert_mut(&mut self, k: K, v: V) {
        match self.root.insert_mut(Raw(k, v)) {
            Insert::NoChange | Insert::JustInc => {}
            Insert::Update(root) => self.root = root,
            Insert::Split(left, median, right) => self.root = Node::from_split(left, median, right),
        }
    }

    fn remove_mut(&mut self, k: &K) {
        match self.root.remove_mut(k) {
            Remove::NoChange => None,
            Remove::Removed(pair) => Some(pair),
            Remove::Update(pair, root) => {
                self.root = root;
                Some(pair)
            }
        };
    }
}

fn pairs() -> Vec<(u64, u64)> {
    StdRng::from_seed(&[2, 2, 3, 7])
        .gen_iter()
        .zip(iter::repeat(42))
        .take(1000)
        .collect()
}

#[bench]
fn add_and_remove(b: &mut Bencher) {
    let pairs = pairs();
    b.iter(|| {
        test::black_box(
            pairs.iter().cloned().fold(
                pairs
                    .iter()
                    .cloned()
                    .fold(OrdMap::new(), |map, (k, v)| map.insert(k, v)),
                |map, (k, _)| map.remove(&k),
            ),
        );
    });
}

#[bench]
fn add_and_remove_mut(b: &mut Bencher) {
    let pairs = pairs();
    b.iter(|| {
        test::black_box(
            pairs.iter().cloned().fold(
                pairs
                    .iter()
                    .cloned()
                    .fold(OrdMap::new(), |mut map, (k, v)| {
                        map.insert_mut(k, v);
                        map
                    }),
                |mut map, (k, _)| {
                    map.remove_mut(&k);
                    map
                },
            ),
        );
    });
}

#[bench]
fn add_and_remove_raw(b: &mut Bencher) {
    let pairs = pairs();
    b.iter(|| {
        test::black_box(
            pairs.iter().cloned().fold(
                pairs
                    .iter()
                    .cloned()
                    .fold(RawMap::new(), |map, (k, v)| map.insert(k, v)),
                |map, (k, _)| map.remove(&k),
            ),
        );
    });
}

#[bench]
fn add_and_remove_raw_mut(b: &mut Bencher) {
    let pairs = pairs();
    b.iter(|| {
        test::black_box(
            pairs.iter().cloned().fold(
                pairs
                    .iter()
                    .cloned()
                    .fold(RawMap::new(), |mut map, (k, v)| {
                        map.insert_mut(k, v);
                        map
                    }),
                |mut map, (k, _)| {
                    map.remove_mut(&k);
                    map
                },
            ),
        );
    });
}
