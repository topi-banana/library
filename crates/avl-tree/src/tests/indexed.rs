use super::*;

/// `Indexed` の部分木サイズが全ノードで整合しているか確認します
fn assert_indexed_sizes<E: Element>(map: &Map<Indexed<E>>, label: impl std::fmt::Debug) {
    fn count<E: Element>(map: &Map<Indexed<E>>, idx: usize) -> usize {
        if idx == usize::MAX {
            return 0;
        }
        let node = &map.nodes[idx];
        1 + count(map, node.left) + count(map, node.right)
    }

    for idx in 0..map.nodes.len() {
        assert_eq!(
            map.nodes[idx].element.size(),
            count(map, idx),
            "indexed size at {idx} ({label:?})"
        );
    }
}

fn assert_indexed_matches_btreemap(
    map: &Map<Indexed<SimpleElement<u32, u64>>>,
    model: &BTreeMap<u32, u64>,
    label: impl std::fmt::Debug,
) {
    assert_eq!(map.len(), model.len(), "{label:?}: len");
    assert_eq!(map.is_empty(), model.is_empty(), "{label:?}: is_empty");

    let got: Vec<(u32, u64)> = map.iter().map(|(&k, &v)| (k, v)).collect();
    let want: Vec<(u32, u64)> = model.iter().map(|(&k, &v)| (k, v)).collect();
    assert_eq!(got, want, "{label:?}: iteration");

    for (index, (&key, &value)) in model.iter().enumerate() {
        let slot = map.slot_by_index(index).unwrap();
        assert_eq!(*unsafe { map.slot_ref(slot) }.inner().key(), key, "{label:?}: key at {index}");
        assert_eq!(map.index_of(slot), index, "{label:?}: index_of {index}");
        assert_eq!(map.get_by_index(index).unwrap().value(), &value, "{label:?}: value at {index}");
        assert_eq!(map.get(&key), Some(&value), "{label:?}: get {key}");
    }
    assert_eq!(map.slot_by_index(model.len()), None, "{label:?}: out of range slot");
    assert!(map.get_by_index(model.len()).is_none(), "{label:?}: out of range value");

    if map.root != usize::MAX {
        assert_eq!(map.nodes[map.root].element.size(), map.len(), "{label:?}: root size");
    }
    assert_indexed_sizes(map, label);
}

#[test]
fn indexed_index_access_matches_btreemap() {
    let mut map: Map<Indexed<SimpleElement<u32, u64>>> = Map::new();
    let mut model: BTreeMap<u32, u64> = BTreeMap::new();
    let mut rng = StdRng::seed_from_u64(0xfeed_face_dead_beef);

    for step in 0..50_000u32 {
        let key = rng.random_range(0..512u32);
        if rng.random_bool(0.6) {
            let value = rng.random();
            map.put(key, value);
            model.insert(key, value);
        } else if model.contains_key(&key) {
            assert_eq!(map.remove(&key), model.remove(&key), "step {step}");
        }
        if step % 251 == 0 {
            assert_indexed_matches_btreemap(&map, &model, step);
        }
    }
    assert_indexed_matches_btreemap(&map, &model, "final");
}

#[test]
fn indexed_multiset_with_tiebreaker() {
    // 重複を許す集合はキーに通し番号を足して一意にする。
    let mut map: Map<Indexed<SimpleElement<(i64, usize), i64>>> = Map::new();
    let mut values: Vec<i64> = Vec::new();
    let mut next_id = 0usize;
    let mut rng = StdRng::seed_from_u64(0x0123_4567_89ab_cdef);

    for step in 0..5_000u32 {
        if rng.random_bool(0.7) || values.is_empty() {
            let value = rng.random_range(0..16i64);
            map.put((value, next_id), value);
            next_id += 1;
            let position = values.partition_point(|&x| x <= value);
            values.insert(position, value);
        } else {
            let index = rng.random_range(0..values.len());
            let slot = map.slot_by_index(index).unwrap();
            assert_eq!(
                *unsafe { map.slot_ref(slot) }.inner().value(),
                values[index],
                "step {step}"
            );
            let removed = unsafe { map.slot_remove(slot) }.into_inner().value;
            assert_eq!(removed, values.remove(index), "step {step}");
        }

        if step % 251 == 0 {
            assert_eq!(map.len(), values.len(), "step {step}: len");
            for (index, &want) in values.iter().enumerate() {
                assert_eq!(
                    *map.get_by_index(index).unwrap().value(),
                    want,
                    "step {step}: index {index}"
                );
                let slot = map.slot_by_index(index).unwrap();
                assert_eq!(map.index_of(slot), index, "step {step}: rank {index}");
            }
            assert!(map.get_by_index(values.len()).is_none(), "step {step}: out of range");
            assert_indexed_sizes(&map, step);
        }
    }
}

#[test]
fn indexed_index_access_edges() {
    let mut map: Map<Indexed<SimpleElement<i32, ()>>> = Map::new();
    assert_eq!(map.slot_by_index(0), None);
    assert!(map.get_by_index(0).is_none());

    let slot = map.put(10, ());
    assert_eq!(map.index_of(slot), 0);
    assert_eq!(map.get_by_index(0).unwrap().key(), &10);
    assert_eq!(map.slot_by_index(1), None);
    assert!(map.get_by_index(1).is_none());

    // 同じキーの置き換えでは要素数は増えない
    let replaced = map.put(10, ());
    assert_eq!(replaced, slot);
    assert_eq!(map.len(), 1);
    assert_eq!(map.index_of(replaced), 0);
    assert_indexed_sizes(&map, "edges");
}

#[test]
fn indexed_value_edits_keep_indices() {
    let mut map: Map<Indexed<SimpleElement<u32, u64>>> = Map::new();
    for key in 0..100u32 {
        map.put(key, 0);
    }

    for key in (0..100u32).step_by(2) {
        let slot = map.search(&key).unwrap();
        *unsafe { map.slot_mut(slot) }.inner_mut().value_mut() = u64::from(key) * 2;
    }

    for (index, key) in (0..100u32).enumerate() {
        let want = if key % 2 == 0 { u64::from(key) * 2 } else { 0 };
        assert_eq!(map.get(&key), Some(&want), "get {key}");
        assert_eq!(map.get_by_index(index).unwrap().key(), &key, "index {index}");
        assert_eq!(map.index_of(map.slot_by_index(index).unwrap()), index, "rank {index}");
    }
    assert_indexed_sizes(&map, "value edits");
}

#[test]
fn indexed_map_get_mut_updates_value() {
    let mut map: Map<Indexed<SimpleElement<u32, u64>>> = Map::new();
    map.put(1, 10);
    map.put(2, 20);

    *map.get_mut(&1).unwrap() = 99;
    assert_eq!(map.get(&1), Some(&99));
    assert_eq!(map.get_by_index(0).unwrap().value(), &99);
    assert_eq!(map.remove(&2), Some(20));
    assert_eq!(map.len(), 1);
    assert_indexed_sizes(&map, "get_mut");
}

#[test]
fn indexed_interval_absorb_delegates() {
    let mut map: Map<Indexed<IntervalElement>> = Map::new();
    let mut model: Vec<(u32, u32, u64)> = Vec::new();

    for &(start, end) in &[(1, 3), (2, 5), (10, 12), (12, 14), (20, 21), (0, 1), (14, 20)] {
        map.insert_element(Indexed::new(IntervalElement { range: (start, end), count: 1 }));
        model_insert_interval(&mut model, (start, end), 1);

        let got: Vec<(u32, u32, u64)> = map
            .iter_elements()
            .map(|e| (e.inner().range.0, e.inner().range.1, e.inner().count))
            .collect();
        assert_eq!(got, model, "after ({start}, {end})");
        assert_indexed_sizes(&map, (start, end));
    }
}

/// `push` で遅延タグを子へ流すテスト用要素 (`Indexed` の委譲の確認用)
#[derive(Debug)]
struct LazyAdd {
    key: i32,
    value: i64,
    lazy: i64,
}

impl Element for LazyAdd {
    type Key = i32;

    fn key(&self) -> &i32 {
        &self.key
    }

    fn push(&mut self, left: Option<&mut Self>, right: Option<&mut Self>) {
        if self.lazy == 0 {
            return;
        }
        for child in [left, right].into_iter().flatten() {
            child.value += self.lazy;
            child.lazy += self.lazy;
        }
        self.lazy = 0;
    }
}

#[test]
fn indexed_push_delegates() {
    let mut root = Indexed::new(LazyAdd { key: 1, value: 0, lazy: 5 });
    let mut left = Indexed::new(LazyAdd { key: 0, value: 1, lazy: 0 });
    let mut right = Indexed::new(LazyAdd { key: 2, value: 2, lazy: 0 });

    root.push(Some(&mut left), Some(&mut right));

    assert_eq!(left.inner().value, 6);
    assert_eq!(left.inner().lazy, 5);
    assert_eq!(right.inner().value, 7);
    assert_eq!(right.inner().lazy, 5);
    assert_eq!(root.inner().lazy, 0);
}
