use super::*;
use rand::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::ops::Bound;

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

    let got: Vec<(u32, V)> = map.iter().map(|(&k, &v)| (k, v)).collect();
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
fn insert_returns_previous_value() {
    let mut map: Map<SimpleElement<i32, i32>> = Map::new();
    assert_eq!(map.insert(1, 10), None);
    assert_eq!(map.insert(1, 20), Some(10));
    assert_eq!(map.insert(2, 30), None);
    assert_eq!(map.iter().map(|(&k, &v)| (k, v)).collect::<Vec<_>>(), vec![(1, 20), (2, 30)]);
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
        set.insert_element(key);
    }
    assert_eq!(set.len(), 3);
    assert!(set.contains(&1));
    assert!(!set.contains(&2));

    let keys: Vec<i32> = set.iter_elements().copied().collect();
    assert_eq!(keys, vec![1, 3, 5]);

    let slot = set.search(&3).unwrap();
    assert_eq!(unsafe { set.slot_remove(slot) }, 3);
    assert_eq!(set.iter_elements().copied().collect::<Vec<_>>(), vec![1, 5]);
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

    for (_, value) in map.iter_mut() {
        *value *= 2;
    }
    let got: Vec<(u32, u32)> = map.iter().map(|(&k, &v)| (k, v)).collect();
    let want: Vec<(u32, u32)> = (0..100u32).map(|k| (k, k * 2)).collect();
    assert_eq!(got, want);
    assert_invariants(&map, "after iter_mut");

    let got: Vec<(u32, u32)> = map.into_iter().collect();
    assert_eq!(got, want);
}

#[test]
fn element_level_iterators_work() {
    let mut map: Map<SizedElement> = Map::new();
    for key in [3, 1, 2] {
        map.insert_element(SizedElement { key, size: 0 });
    }

    assert_eq!(map.iter_elements().map(|e| e.key).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(map.iter_elements().rev().map(|e| e.key).collect::<Vec<_>>(), vec![3, 2, 1]);

    // 可変イテレータで集約値を編集したら slot_refresh で直す
    for element in map.iter_mut_elements() {
        element.size = 0;
    }
    // 各ノードを refresh すると、根まで伝播して集約値が復元される
    let keys: Vec<i32> = map.iter_elements().map(|e| e.key).collect();
    for key in keys {
        let slot = map.search(&key).unwrap();
        unsafe { map.slot_refresh(slot) };
    }
    assert_subtree_sizes(&map, "after iter_mut_elements");

    let removed: Vec<SizedElement> = map.extract_elements_if(|e| e.key % 2 == 1).collect();
    assert_eq!(removed.len(), 2);
    assert_eq!(map.len(), 1);

    let owned: Vec<i32> = map.into_elements().map(|e| e.key).collect();
    assert_eq!(owned, vec![2]);
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

    let removed: Vec<(u32, u32)> = map.extract_if(.., |key, _| predicate(*key)).collect();

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

#[test]
fn extract_if_range_removes_only_in_range() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    for key in 0..100u32 {
        let value = u64::from(key) * 3 + 1;
        map.put(key, value);
        model.insert(key, value);
    }

    let removed: Vec<(u32, u64)> = map.extract_if(20..80, |key, _| key % 2 == 0).collect();
    let want: Vec<(u32, u64)> =
        (20..80u32).filter(|k| k % 2 == 0).map(|k| (k, u64::from(k) * 3 + 1)).collect();
    assert_eq!(removed, want);
    for &(key, _) in &want {
        model.remove(&key);
    }
    assert_matches_btreemap(&map, &model, "extract_if range");

    // 範囲外の要素は残る
    for key in (0..20u32).chain(80..100) {
        assert!(map.contains_key(&key), "key {key} should remain");
    }
}

#[test]
fn entry_api_basics() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();

    *map.entry(1).or_insert(10) += 1;
    map.entry(1).and_modify(|value| *value *= 2).or_insert(0);
    assert_eq!(map.get(&1), Some(&22));

    map.entry(2).or_insert_with_key(|key| u64::from(*key) * 5);
    assert_eq!(map.get(&2), Some(&10));

    map.entry(3).or_default();
    assert_eq!(map.get(&3), Some(&0));

    map.entry(4).or_insert_with(|| 40);
    assert_eq!(map.get(&4), Some(&40));

    assert_eq!(map.entry(1).key(), &1);

    let mut occupied = map.entry(4).insert_entry(44);
    assert_eq!(*occupied.key(), 4);
    assert_eq!(*occupied.get(), 44);
    *occupied.get_mut() += 1;
    assert_eq!(occupied.insert(100), 45);
    assert_eq!(occupied.remove_entry(), (4, 100));
    assert_eq!(map.len(), 3);

    match map.entry(5) {
        Entry::Vacant(entry) => assert_eq!(entry.into_key(), 5),
        Entry::Occupied(_) => panic!("expected vacant"),
    }
    match map.entry(1) {
        Entry::Occupied(entry) => assert_eq!(*entry.into_mut(), 22),
        Entry::Vacant(_) => panic!("expected occupied"),
    }
}

#[test]
fn entry_random_matches_btreemap() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    let mut rng = StdRng::seed_from_u64(0x0bad_cafe_f00d_1234);

    for step in 0..20_000u32 {
        let key = rng.random_range(0..256u32);
        match rng.random_range(0..5u32) {
            0 => {
                let value = rng.random();
                assert_eq!(map.insert(key, value), model.insert(key, value), "step {step}");
            }
            1 => {
                *map.entry(key).or_insert(0) += 1;
                *model.entry(key).or_insert(0) += 1;
            }
            2 => {
                map.entry(key).and_modify(|v| *v = v.wrapping_add(3)).or_insert(5);
                model.entry(key).and_modify(|v| *v = v.wrapping_add(3)).or_insert(5);
            }
            3 => {
                assert_eq!(map.remove_entry(&key), model.remove_entry(&key), "step {step}");
            }
            _ => {
                assert_eq!(
                    map.get_key_value(&key).map(|(&k, &v)| (k, v)),
                    model.get_key_value(&key).map(|(&k, &v)| (k, v)),
                    "step {step}"
                );
            }
        }
        if step % 251 == 0 {
            assert_matches_btreemap(&map, &model, step);
        }
    }
    assert_matches_btreemap(&map, &model, "final");
}

#[test]
fn entry_debug_format() {
    let mut map: Map<SimpleElement<i32, i32>> = Map::new();
    map.insert(1, 2);
    assert_eq!(format!("{:?}", map.entry(1)), "Entry(OccupiedEntry { key: 1, value: 2 })");
    assert_eq!(format!("{:?}", map.entry(3)), "Entry(VacantEntry(3))");
}

#[test]
fn range_api_matches_btreemap() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    let mut rng = StdRng::seed_from_u64(0x1111_2222_3333_4444);
    for key in 0..200u32 {
        let value = rng.random();
        map.put(key, value);
        model.insert(key, value);
    }
    for key in (0..200u32).step_by(3) {
        map.remove(&key);
        model.remove(&key);
    }

    let ranges: Vec<(Bound<u32>, Bound<u32>)> = vec![
        (Bound::Unbounded, Bound::Unbounded),
        (Bound::Included(50), Bound::Excluded(120)),
        (Bound::Included(50), Bound::Included(120)),
        (Bound::Excluded(50), Bound::Included(120)),
        (Bound::Excluded(50), Bound::Excluded(120)),
        (Bound::Unbounded, Bound::Excluded(10)),
        (Bound::Included(190), Bound::Unbounded),
        (Bound::Excluded(100), Bound::Included(100)),
        (Bound::Included(51), Bound::Excluded(51)),
    ];
    for &(lower, upper) in &ranges {
        let got: Vec<(u32, u64)> = map.range((lower, upper)).map(|(&k, &v)| (k, v)).collect();
        let want: Vec<(u32, u64)> = model.range((lower, upper)).map(|(&k, &v)| (k, v)).collect();
        assert_eq!(got, want, "range {lower:?}..{upper:?}");

        let rev: Vec<(u32, u64)> = map.range((lower, upper)).rev().map(|(&k, &v)| (k, v)).collect();
        let mut want_rev = want;
        want_rev.reverse();
        assert_eq!(rev, want_rev, "rev range {lower:?}..{upper:?}");
    }

    // next と next_back を混ぜても重複なく返る
    let mut iter = map.range(40..140);
    let mut got = Vec::new();
    while let Some((&key, _)) = iter.next() {
        got.push(key);
        if let Some((&key, _)) = iter.next_back() {
            got.push(key);
        } else {
            break;
        }
    }
    got.sort_unstable();
    let want: Vec<u32> = model.range(40..140).map(|(&k, _)| k).collect();
    assert_eq!(got, want);
}

#[test]
#[should_panic(expected = "range start is greater than range end in BTreeMap")]
#[allow(clippy::reversed_empty_ranges)]
fn range_reversed_bounds_panics() {
    let map: Map<SimpleElement<u32, u64>> = Map::new();
    let _ = map.range(120..50).count();
}

#[test]
#[should_panic(expected = "range start and end are equal and excluded in BTreeMap")]
fn range_excluded_equal_bounds_panics() {
    let map: Map<SimpleElement<u32, u64>> = Map::new();
    let _ = map.range((Bound::Excluded(5), Bound::Excluded(5))).count();
}

#[test]
#[should_panic(expected = "range start is greater than range end in BTreeMap")]
#[allow(clippy::reversed_empty_ranges)]
fn range_mut_reversed_bounds_panics() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let _ = map.range_mut(120..50).count();
}

#[test]
#[should_panic(expected = "range start is greater than range end in BTreeMap")]
#[allow(clippy::reversed_empty_ranges)]
fn extract_if_reversed_bounds_panics() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let _ = map.extract_if(120..50, |_, _| true).count();
}

#[test]
fn range_mut_updates_values() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    for key in 0..100u32 {
        map.put(key, u64::from(key));
        model.insert(key, u64::from(key));
    }

    for (_, value) in map.range_mut(20..80) {
        *value += 1000;
    }
    for (_, value) in model.range_mut(20..80) {
        *value += 1000;
    }
    assert_matches_btreemap(&map, &model, "range_mut");

    let keys: Vec<u32> = map.range_mut(20..80).rev().map(|(&k, _)| k).collect();
    assert_eq!(keys, (20..80u32).rev().collect::<Vec<_>>());
}

#[test]
fn keys_values_iterators() {
    let map: Map<SimpleElement<u32, u64>> =
        (0..10u32).map(|key| (key, u64::from(key) * 2)).collect();

    let keys: Vec<u32> = map.keys().copied().collect();
    assert_eq!(keys, (0..10u32).collect::<Vec<_>>());
    let keys_rev: Vec<u32> = map.keys().rev().copied().collect();
    assert_eq!(keys_rev, (0..10u32).rev().collect::<Vec<_>>());

    let values: Vec<u64> = map.values().copied().collect();
    assert_eq!(values, (0..10u32).map(|key| u64::from(key) * 2).collect::<Vec<_>>());
    let values_rev: Vec<u64> = map.values().rev().copied().collect();
    assert_eq!(values_rev, values.iter().copied().rev().collect::<Vec<_>>());

    let mut map = map;
    for value in map.values_mut() {
        *value += 1;
    }
    assert_eq!(
        map.values().copied().collect::<Vec<_>>(),
        (0..10u32).map(|key| u64::from(key) * 2 + 1).collect::<Vec<_>>()
    );

    let by_ref: Vec<(u32, u64)> = (&map).into_iter().map(|(&k, &v)| (k, v)).collect();
    assert_eq!(by_ref.len(), 10);

    for (_, value) in &mut map {
        *value += 1;
    }
    let by_mut: Vec<(u32, u64)> = map.iter().map(|(&k, &v)| (k, v)).collect();
    assert_eq!(by_mut[0], (0, 2));

    let keys: Vec<u32> = map.clone().into_keys().collect();
    assert_eq!(keys, (0..10u32).collect::<Vec<_>>());
    let values: Vec<u64> = map.clone().into_values().collect();
    assert_eq!(values, (0..10u32).map(|key| u64::from(key) * 2 + 2).collect::<Vec<_>>());
}

#[test]
fn retain_matches_btreemap() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    for key in 0..500u32 {
        map.put(key, u64::from(key));
        model.insert(key, u64::from(key));
    }

    map.retain(|key, value| {
        *value += 1;
        key % 3 != 0
    });
    model.retain(|key, value| {
        *value += 1;
        key % 3 != 0
    });
    assert_matches_btreemap(&map, &model, "retain");
}

#[test]
fn drain_and_clear() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    for key in (0..100u32).rev() {
        map.put(key, u64::from(key));
    }
    let drained: Vec<(u32, u64)> = map.drain().collect();
    assert_eq!(drained, (0..100u32).map(|key| (key, u64::from(key))).collect::<Vec<_>>());
    assert!(map.is_empty());
    assert_invariants(&map, "after drain");

    // 途中で drop しても空になる
    for key in 0..10u32 {
        map.put(key, u64::from(key));
    }
    {
        let mut drain = map.drain();
        assert_eq!(drain.next(), Some((0, 0)));
    }
    assert!(map.is_empty());

    for key in 0..10u32 {
        map.put(key, u64::from(key));
    }
    let capacity = map.capacity();
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.capacity(), capacity);
    assert_invariants(&map, "after clear");
}

#[test]
fn capacity_methods() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::with_capacity(100);
    assert!(map.capacity() >= 100);

    map.reserve(200);
    assert!(map.capacity() >= 200);
    map.reserve_exact(400);
    assert!(map.capacity() >= 400);
    map.try_reserve(1).unwrap();
    map.try_reserve_exact(1).unwrap();

    // shrink_to は min_capacity を下回らない範囲で縮める
    map.shrink_to(50);
    assert!(map.capacity() >= 50);
    map.shrink_to_fit();
    assert!(map.capacity() >= map.len());

    for key in 0..100u32 {
        map.put(key, u64::from(key));
    }
    assert_eq!(map.len(), 100);

    // 要素を入れた状態でも len を下回らない
    map.shrink_to_fit();
    assert!(map.capacity() >= 100);
    map.shrink_to(10);
    assert!(map.capacity() >= 100);
}

#[test]
fn first_last_and_pop() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    for key in 0..10u32 {
        map.put(key, u64::from(key) * 10);
    }

    assert_eq!(map.first_key_value().map(|(&k, &v)| (k, v)), Some((0, 0)));
    assert_eq!(map.last_key_value().map(|(&k, &v)| (k, v)), Some((9, 90)));

    let entry = map.first_entry().unwrap();
    assert_eq!(*entry.key(), 0);
    assert_eq!(entry.remove(), 0);
    assert_eq!(map.len(), 9);

    let entry = map.last_entry().unwrap();
    assert_eq!(*entry.key(), 9);
    assert_eq!(*entry.get(), 90);
    *entry.into_mut() += 5;
    assert_eq!(map.get(&9), Some(&95));

    assert_eq!(map.pop_first(), Some((1, 10)));
    assert_eq!(map.pop_last(), Some((9, 95)));
    assert_eq!(map.len(), 7);

    let mut empty: Map<SimpleElement<u32, u64>> = Map::new();
    assert_eq!(empty.first_key_value(), None);
    assert_eq!(empty.last_key_value(), None);
    assert!(empty.first_entry().is_none());
    assert!(empty.last_entry().is_none());
    assert_eq!(empty.pop_first(), None);
    assert_eq!(empty.pop_last(), None);
}

#[test]
fn append_and_split_off_match_btreemap() {
    let mut map: Map<SimpleElement<u32, u64>> =
        (0..100u32).map(|key| (key, u64::from(key))).collect();
    let mut other: Map<SimpleElement<u32, u64>> =
        (50..150u32).map(|key| (key, u64::from(key) + 1000)).collect();
    let mut model: BTreeMap<u32, u64> = (0..100u32).map(|key| (key, u64::from(key))).collect();
    let mut model_other: BTreeMap<u32, u64> =
        (50..150u32).map(|key| (key, u64::from(key) + 1000)).collect();

    map.append(&mut other);
    model.append(&mut model_other);
    assert!(other.is_empty());
    assert!(model_other.is_empty());
    assert_matches_btreemap(&map, &model, "append");

    let split = map.split_off(&80);
    let split_model = model.split_off(&80);
    assert_matches_btreemap(&map, &model, "split left");
    assert_matches_btreemap(&split, &split_model, "split right");

    // 存在しないキーでも動く
    let mut map2: Map<SimpleElement<u32, u64>> =
        (0..10u32).map(|key| (key, u64::from(key))).collect();
    let right = map2.split_off(&5);
    assert_eq!(map2.keys().copied().collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
    assert_eq!(right.keys().copied().collect::<Vec<_>>(), vec![5, 6, 7, 8, 9]);

    let right = map2.split_off(&100);
    assert!(right.is_empty());
    assert_eq!(map2.len(), 5);
}

#[test]
fn get_disjoint_mut_works() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    for key in 0..10u32 {
        map.put(key, u64::from(key));
    }

    match map.get_disjoint_mut([&1, &9, &100]) {
        [Some(a), Some(b), None] => {
            *a += 100;
            *b += 1000;
        }
        _ => panic!("unexpected result"),
    }
    assert_eq!(map.get(&1), Some(&101));
    assert_eq!(map.get(&9), Some(&1009));
    assert_eq!(map.get(&0), Some(&0));

    // 空配列も受け付ける
    let empty: [Option<&mut u64>; 0] = map.get_disjoint_mut([]);
    assert_eq!(empty.len(), 0);
}

#[test]
#[should_panic(expected = "duplicate keys found")]
fn get_disjoint_mut_panics_on_duplicates() {
    let mut map: Map<SimpleElement<u32, u64>> = Map::new();
    map.put(1, 10);
    let _ = map.get_disjoint_mut([&1, &1]);
}

#[test]
fn index_operator_reads_value() {
    let map: Map<SimpleElement<u32, u64>> =
        (0..5u32).map(|key| (key, u64::from(key) * 2)).collect();
    assert_eq!(map[&3], 6);
    assert_eq!(map[&0], 0);
}

#[test]
#[should_panic(expected = "no entry found for key")]
fn index_operator_panics_for_missing_key() {
    let map: Map<SimpleElement<u32, u64>> = Map::new();
    let _ = map[&0];
}

#[test]
fn from_iter_extend_and_comparison_traits() {
    let mut map: Map<SimpleElement<i32, i32>> = [(1, 10), (2, 20)].into_iter().collect();
    map.extend([(3, 30)]);
    let key = 4;
    let value = 40;
    map.extend([(&key, &value)]);
    assert_eq!(format!("{map:?}"), "{1: 10, 2: 20, 3: 30, 4: 40}");

    let same: Map<SimpleElement<i32, i32>> =
        [(1, 10), (2, 20), (3, 30), (4, 40)].into_iter().collect();
    assert_eq!(map, same);

    let mut less: Map<SimpleElement<i32, i32>> = [(1, 10)].into_iter().collect();
    assert!(less < map);
    assert!(less != map);
    less.extend([(2, 20), (3, 30), (4, 40)]);
    assert_eq!(less, map);
    assert!(less <= map);
    assert!(!(less > map));

    let mut hasher = DefaultHasher::new();
    let mut same_hasher = DefaultHasher::new();
    map.hash(&mut hasher);
    same.hash(&mut same_hasher);
    assert_eq!(hasher.finish(), same_hasher.finish());

    let bigger: Map<SimpleElement<i32, i32>> =
        [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)].into_iter().collect();
    assert!(map < bigger);
}

#[test]
fn clone_is_independent() {
    let mut map: Map<SimpleElement<u32, u64>> =
        (0..50u32).map(|key| (key, u64::from(key))).collect();
    let clone = map.clone();
    assert_eq!(map, clone);

    for (_, value) in map.iter_mut() {
        *value += 1;
    }
    assert_ne!(map, clone);
    assert_eq!(clone.get(&0), Some(&0));
    assert_eq!(map.get(&0), Some(&1));
    assert_invariants(&clone, "clone");
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
            map.insert_element(SizedElement { key, size: 0 });
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
    let keys: Vec<i32> = map.iter_elements().map(|e| e.key).collect();
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
        map.insert_element(SumElement { key, value: 1, sum: 0 });
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

        map.insert_element(IntervalElement { range, count: 1 });
        model_insert_interval(&mut model, range, 1);

        if step % 251 == 0 {
            assert_invariants(&map, &format!("intervals step {step}"));
            let got: Vec<(u32, u32, u64)> =
                map.iter_elements().map(|e| (e.range.0, e.range.1, e.count)).collect();
            assert_eq!(got, model, "step {step}");
        }
    }

    let got: Vec<(u32, u32, u64)> =
        map.iter_elements().map(|e| (e.range.0, e.range.1, e.count)).collect();
    assert_eq!(got, model);
}

mod indexed;
mod interval_set;
mod lazy_segment_tree;
