use super::*;
use rand::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

/// AVL 木の構造不変条件を確認します
fn assert_invariants<E: Element>(map: &Map<E>, label: &str)
where
    E::Key: Clone + std::fmt::Debug,
{
    if map.root == usize::MAX {
        assert_eq!(map.len, 0, "{label}: len of empty map");
        assert!(map.nodes.is_empty(), "{label}: nodes of empty map");
        return;
    }

    assert_eq!(map.nodes[map.root].parent, usize::MAX, "{label}: root parent");

    // 親リンクと到達ノード数
    let mut reachable = vec![map.root];
    let mut i = 0;
    while i < reachable.len() {
        let idx = reachable[i];
        i += 1;
        let node = &map.nodes[idx];
        if node.left != usize::MAX {
            assert_eq!(map.nodes[node.left].parent, idx, "{label}: left parent link at {idx}");
            reachable.push(node.left);
        }
        if node.right != usize::MAX {
            assert_eq!(map.nodes[node.right].parent, idx, "{label}: right parent link at {idx}");
            reachable.push(node.right);
        }
    }
    assert_eq!(reachable.len(), map.len, "{label}: reachable node count");

    // in-order のキーが狭義単調増加
    let mut keys = Vec::with_capacity(map.len);
    let mut current = map.root;
    while map.nodes[current].left != usize::MAX {
        current = map.nodes[current].left;
    }
    while current != usize::MAX {
        keys.push(map.nodes[current].element.key().clone());
        current = map.next_node(current);
    }
    assert_eq!(keys.len(), map.len, "{label}: in-order count");
    for pair in keys.windows(2) {
        assert!(pair[0] < pair[1], "{label}: BST order violated");
    }

    // 高さと平衡
    for &idx in &reachable {
        let node = &map.nodes[idx];
        let lh = if node.left != usize::MAX { map.nodes[node.left].height } else { 0 };
        let rh = if node.right != usize::MAX { map.nodes[node.right].height } else { 0 };
        assert_eq!(node.height, lh.max(rh) + 1, "{label}: height at {idx}");
        assert!(lh.abs_diff(rh) <= 1, "{label}: balance at {idx} ({lh} vs {rh})");
    }
}

fn assert_matches_btreemap<V: Copy + std::fmt::Debug + PartialEq>(
    map: &Map<SimpleElement<u32, V>>,
    model: &BTreeMap<u32, V>,
    label: impl std::fmt::Debug,
) {
    assert_eq!(map.len(), model.len(), "{label:?}: len");
    assert_eq!(map.is_empty(), model.is_empty(), "{label:?}: is_empty");

    let got: Vec<(u32, V)> = map.iter().map(|e| (*e.key(), *e.value())).collect();
    let want: Vec<(u32, V)> = model.iter().map(|(&k, &v)| (k, v)).collect();
    assert_eq!(got, want, "{label:?}: iteration");

    for (&key, &value) in model {
        assert_eq!(map.get(&key), Some(&value), "{label:?}: get {key}");
        assert!(map.contains(&key), "{label:?}: contains {key}");
    }

    assert_invariants(map, &format!("{label:?}"));
}

#[test]
fn put_remove_matches_btreemap() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    let mut rng = StdRng::seed_from_u64(0x1234_5678_9abc_def0);

    for step in 0..100_000u32 {
        let key = rng.random_range(0..512u32);
        if rng.random_bool(0.6) {
            let value = rng.random();
            map.put(key, value);
            model.insert(key, value);
        } else {
            assert_eq!(map.remove(&key), model.remove(&key), "step {step}");
        }
        if step % 997 == 0 {
            assert_matches_btreemap(&map, &model, step);
        }
    }
    assert_matches_btreemap(&map, &model, "final");
}

#[test]
fn put_replaces_existing_value() {
    let mut map: Map<SimpleElement<i32, i32>> = Map::new();
    let slot = map.put(1, 10);
    map.put(1, 20);
    assert_eq!(map.len(), 1);
    assert_eq!(map.get(&1), Some(&20));
    assert_eq!(unsafe { map.slot_ref(slot) }.value(), &20);
}

#[test]
fn slot_api_works() {
    let mut map: Map<SimpleElement<i32, i32>> = Map::new();

    let vacant = map.search(&10).unwrap_err();
    let slot = map.slot_insert(vacant, SimpleElement::new(10, 1));

    assert_eq!(map.search(&10), Ok(slot));
    assert_eq!(map.search(&5), Err(VacantSlot::Left(slot.index)));
    assert_eq!(map.search(&15), Err(VacantSlot::Right(slot.index)));

    unsafe { map.slot_mut(slot).value = 2 };
    assert_eq!(map.get(&10), Some(&2));

    let removed = unsafe { map.slot_remove(slot) };
    assert_eq!(*removed.key(), 10);
    assert_eq!(*removed.value(), 2);
    assert!(map.is_empty());
    assert_invariants(&map, "after slot_remove");
}

#[test]
fn int_set_works() {
    let mut set: Map<i32> = Map::new();
    for key in [5, 1, 3, 1] {
        set.insert(key);
    }
    assert_eq!(set.len(), 3);
    assert!(set.contains(&1));
    assert!(!set.contains(&2));

    let keys: Vec<i32> = set.iter().copied().collect();
    assert_eq!(keys, vec![1, 3, 5]);

    let slot = set.search(&3).unwrap();
    assert_eq!(unsafe { set.slot_remove(slot) }, 3);
    assert_eq!(set.iter().copied().collect::<Vec<_>>(), vec![1, 5]);
    assert_invariants(&set, "int set");
}

#[test]
fn slot_traversal() {
    let mut map: Map<SimpleElement<u32, ()>> = Map::new();
    for key in [5u32, 1, 9, 3, 7] {
        map.put(key, ());
    }

    // next で昇順に辿れる
    let mut keys = Vec::new();
    let mut current = map.first();
    while let Some(slot) = current {
        keys.push(*unsafe { map.slot_ref(slot) }.key());
        current = map.next(slot);
    }
    assert_eq!(keys, vec![1, 3, 5, 7, 9]);

    // prev で降順に辿れる
    let mut keys = Vec::new();
    let mut current = map.last();
    while let Some(slot) = current {
        keys.push(*unsafe { map.slot_ref(slot) }.key());
        current = map.prev(slot);
    }
    assert_eq!(keys, vec![9, 7, 5, 3, 1]);

    // predecessor / successor
    let pred = |key: u32| map.predecessor(&key).map(|s| *unsafe { map.slot_ref(s) }.key());
    let succ = |key: u32| map.successor(&key).map(|s| *unsafe { map.slot_ref(s) }.key());
    assert_eq!(pred(5), Some(3));
    assert_eq!(succ(5), Some(7));
    assert_eq!(pred(0), None);
    assert_eq!(succ(10), None);
    assert_eq!(pred(1), None);
    assert_eq!(succ(9), None);
}

#[test]
fn iter_mut_and_into_iter() {
    let mut map: Map<SimpleElement<u32, u32>> = Map::new();
    for key in 0..100u32 {
        map.put(key, key);
    }

    for element in map.iter_mut() {
        *element.value_mut() *= 2;
    }
    let got: Vec<(u32, u32)> = map.iter().map(|e| (*e.key(), *e.value())).collect();
    let want: Vec<(u32, u32)> = (0..100u32).map(|k| (k, k * 2)).collect();
    assert_eq!(got, want);
    assert_invariants(&map, "after iter_mut");

    let got: Vec<(u32, u32)> = map.into_iter().map(|e| (e.key, e.value)).collect();
    assert_eq!(got, want);
}

fn extract_if_case(n: u32, seed: u64, name: &str, predicate: impl Fn(u32) -> bool) {
    let mut map: Map<SimpleElement<u32, u32>> = Map::new();
    let mut model = BTreeMap::new();
    let mut keys: Vec<u32> = (0..n).collect();
    keys.shuffle(&mut StdRng::seed_from_u64(seed));
    for key in keys {
        map.put(key, key * 7 + 1);
        model.insert(key, key * 7 + 1);
    }

    let removed: Vec<(u32, u32)> = map
        .extract_if(|element| predicate(*element.key()))
        .map(|element| (element.key, element.value))
        .collect();

    // 昇順 (in-order) で重複なく返る
    let mut prev: Option<u32> = None;
    for &(key, value) in &removed {
        assert_eq!(value, key * 7 + 1, "[{name}] value mismatch for {key}");
        if let Some(p) = prev {
            assert!(p < key, "[{name}] not in ascending order: {p} then {key}");
        }
        prev = Some(key);
    }

    let want: Vec<u32> = model.keys().copied().filter(|&key| predicate(key)).collect();
    assert_eq!(removed.iter().map(|&(key, _)| key).collect::<Vec<_>>(), want, "[{name}] keys");

    for &key in &want {
        model.remove(&key);
    }
    assert_matches_btreemap(&map, &model, name);
}

#[test]
fn extract_if_matches_model() {
    for seed in [12345u64, 777, 0xdead_beef] {
        for n in [1u32, 2, 3, 5, 40, 200] {
            extract_if_case(n, seed, &format!("even n={n} seed={seed}"), |key| key % 2 == 0);
            extract_if_case(n, seed, &format!("odd n={n} seed={seed}"), |key| key % 2 == 1);
            extract_if_case(n, seed, &format!("lt n={n} seed={seed}"), |key| key < n / 3);
            extract_if_case(n, seed, &format!("all n={n} seed={seed}"), |_| true);
            extract_if_case(n, seed, &format!("none n={n} seed={seed}"), |_| false);
            extract_if_case(n, seed, &format!("mod3 n={n} seed={seed}"), |key| key % 3 == 0);
        }
    }
    for seed in [1u64, 42, 0xfeed_face, 0x0123_4567_89ab_cdef] {
        let n = 2000;
        let modulus = StdRng::seed_from_u64(seed).random_range(2..=5u32);
        extract_if_case(n, seed, &format!("random n={n} seed={seed}"), move |key| {
            key % modulus != 0
        });
    }
}

/// `update` で部分木サイズを保持するテスト用要素
#[derive(Debug)]
struct SizedElement {
    key: i32,
    size: usize,
}

impl Element for SizedElement {
    type Key = i32;

    fn key(&self) -> &i32 {
        &self.key
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.size = 1 + left.map_or(0, |l| l.size) + right.map_or(0, |r| r.size);
    }
}

fn assert_subtree_sizes(map: &Map<SizedElement>, label: impl std::fmt::Debug) {
    fn count(map: &Map<SizedElement>, idx: usize) -> usize {
        if idx == usize::MAX {
            return 0;
        }
        let node = &map.nodes[idx];
        1 + count(map, node.left) + count(map, node.right)
    }

    for idx in 0..map.nodes.len() {
        assert_eq!(
            map.nodes[idx].element.size,
            count(map, idx),
            "subtree size at {idx} ({label:?})"
        );
    }
}

#[test]
fn update_hook_maintains_subtree_sizes() {
    let mut map: Map<SizedElement> = Map::new();
    let mut model = BTreeSet::new();
    let mut rng = StdRng::seed_from_u64(0xdead_beef);

    for step in 0..50_000u32 {
        let key = rng.random_range(0..256i32);
        if rng.random_bool(0.6) {
            map.insert(SizedElement { key, size: 0 });
            model.insert(key);
        } else if model.contains(&key) {
            let slot = map.search(&key).unwrap();
            unsafe { map.slot_remove(slot) };
            model.remove(&key);
        }
        assert_eq!(map.len(), model.len(), "step {step}");
        if step % 251 == 0 {
            assert_invariants(&map, &format!("sizes step {step}"));
            assert_subtree_sizes(&map, step);
        }
    }

    assert_subtree_sizes(&map, "final");
    let keys: Vec<i32> = map.iter().map(|e| e.key).collect();
    assert_eq!(keys, model.iter().copied().collect::<Vec<_>>());
}

/// `update` で部分木和を保持するテスト用要素
#[derive(Debug)]
struct SumElement {
    key: i32,
    value: u64,
    sum: u64,
}

impl Element for SumElement {
    type Key = i32;

    fn key(&self) -> &i32 {
        &self.key
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.sum = self.value + left.map_or(0, |l| l.sum) + right.map_or(0, |r| r.sum);
    }
}

fn assert_sum_consistency(map: &Map<SumElement>) {
    fn check(map: &Map<SumElement>, idx: usize) -> u64 {
        if idx == usize::MAX {
            return 0;
        }
        let node = &map.nodes[idx];
        let sum = check(map, node.left) + check(map, node.right) + node.element.value;
        assert_eq!(node.element.sum, sum, "sum at {idx}");
        sum
    }
    if map.root != usize::MAX {
        check(map, map.root);
    }
}

#[test]
fn slot_refresh_recomputes_ancestors() {
    let mut map: Map<SumElement> = Map::new();
    let mut model = BTreeMap::new();
    let mut rng = StdRng::seed_from_u64(42);

    for key in 0..100i32 {
        map.insert(SumElement { key, value: 1, sum: 0 });
        model.insert(key, 1u64);
    }
    assert_eq!(map.nodes[map.root].element.sum, 100);

    for step in 0..500 {
        let key = rng.random_range(0..100i32);
        let value = rng.random_range(0..1000u64);
        let slot = map.search(&key).unwrap();
        unsafe {
            map.slot_mut(slot).value = value;
            map.slot_refresh(slot);
        }
        model.insert(key, value);

        let total: u64 = model.values().sum();
        assert_eq!(map.nodes[map.root].element.sum, total, "step {step}");
        assert_sum_consistency(&map);
    }
}

/// `can_absorb` / `absorb` で重複・隣接区間を統合するテスト用要素
///
/// 区間は `[start, end)` で表します。
#[derive(Debug)]
struct IntervalElement {
    range: (u32, u32),
    count: u64,
}

impl Element for IntervalElement {
    type Key = (u32, u32);

    fn key(&self) -> &Self::Key {
        &self.range
    }

    fn can_absorb(&self, other: &Self) -> bool {
        self.range.0 <= other.range.1 && other.range.0 <= self.range.1
    }

    fn absorb(&mut self, other: Self) {
        self.range.0 = self.range.0.min(other.range.0);
        self.range.1 = self.range.1.max(other.range.1);
        self.count += other.count;
    }
}

fn model_insert_interval(model: &mut Vec<(u32, u32, u64)>, range: (u32, u32), count: u64) {
    let (mut start, mut end) = range;
    let mut total = count;
    model.retain(|&(s, e, c)| {
        if s <= end && start <= e {
            start = start.min(s);
            end = end.max(e);
            total += c;
            false
        } else {
            true
        }
    });
    model.push((start, end, total));
    model.sort_unstable();
}

#[test]
fn absorb_merges_intervals() {
    let mut map: Map<IntervalElement> = Map::new();
    let mut model: Vec<(u32, u32, u64)> = Vec::new();
    let mut rng = StdRng::seed_from_u64(0x0123_4567_89ab_cdef);

    for step in 0..20_000u32 {
        let start = rng.random_range(0..64u32);
        let len = rng.random_range(0..8u32);
        if len == 0 {
            continue;
        }
        let range = (start, start + len);

        map.insert(IntervalElement { range, count: 1 });
        model_insert_interval(&mut model, range, 1);

        if step % 251 == 0 {
            assert_invariants(&map, &format!("intervals step {step}"));
            let got: Vec<(u32, u32, u64)> =
                map.iter().map(|e| (e.range.0, e.range.1, e.count)).collect();
            assert_eq!(got, model, "step {step}");
        }
    }

    let got: Vec<(u32, u32, u64)> = map.iter().map(|e| (e.range.0, e.range.1, e.count)).collect();
    assert_eq!(got, model);
}

mod indexed;
