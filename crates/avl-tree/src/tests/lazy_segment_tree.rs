use super::*;

use std::ops::Range;

/// 区間加算・区間和
///
/// 部分木の集約値は複数要素の和なので、作用を集約値へ写すには要素数が必要になる。
/// そのため `S` は (和, 要素数) の組にする。
enum AddSum {}

/// 要素 1 個分の値
fn add_value(value: i64) -> (i64, i64) {
    (value, 1)
}

impl Monoid for AddSum {
    type S = (i64, i64);

    fn op(a: &(i64, i64), b: &(i64, i64)) -> (i64, i64) {
        (a.0 + b.0, a.1 + b.1)
    }

    fn identity() -> (i64, i64) {
        (0, 0)
    }
}

impl Action for AddSum {
    type F = i64;

    fn mapping(f: &i64, s: &(i64, i64)) -> (i64, i64) {
        (s.0 + f * s.1, s.1)
    }

    fn composition(f: &i64, g: &i64) -> i64 {
        f + g
    }

    fn id() -> i64 {
        0
    }
}

/// 素朴な区間加算・区間和モデル
struct NaiveAddSum {
    entries: BTreeMap<i64, i64>,
}

impl NaiveAddSum {
    fn new() -> Self {
        NaiveAddSum { entries: BTreeMap::new() }
    }

    fn put(&mut self, key: i64, value: i64) {
        self.entries.insert(key, value);
    }

    fn remove(&mut self, key: i64) -> Option<i64> {
        self.entries.remove(&key)
    }

    fn get(&self, key: i64) -> Option<i64> {
        self.entries.get(&key).copied()
    }

    fn apply(&mut self, range: Range<i64>, f: i64) {
        for value in self.entries.range_mut(range).map(|(_, v)| v) {
            *value += f;
        }
    }

    fn prod(&self, range: Range<i64>) -> i64 {
        self.entries.range(range).map(|(_, &v)| v).sum()
    }

    fn all_prod(&self) -> i64 {
        self.entries.values().sum()
    }
}

fn assert_add_sum_matches(
    seg: &mut Map<LazySegmentTree<i64, AddSum>>,
    model: &NaiveAddSum,
    label: impl std::fmt::Debug,
) {
    assert_eq!(seg.len(), model.entries.len(), "{label:?}: len");
    for (&key, &value) in &model.entries {
        assert_eq!(seg.get(&key).copied(), Some(add_value(value)), "{label:?}: get {key}");
    }
    for l in 0..16i64 {
        for r in l..=16 {
            assert_eq!(
                seg.prod(l * 4..r * 4),
                (model.prod(l * 4..r * 4), count_in(model, l * 4..r * 4)),
                "{label:?}: prod {}..{}",
                l * 4,
                r * 4
            );
        }
    }
    assert_eq!(seg.all_prod(), (model.all_prod(), model.entries.len() as i64), "{label:?}: all");
    assert_invariants(seg, &format!("{label:?}"));
}

fn count_in(model: &NaiveAddSum, range: Range<i64>) -> i64 {
    model.entries.range(range).count() as i64
}

#[test]
fn range_add_sum_matches_naive() {
    let mut seg: Map<LazySegmentTree<i64, AddSum>> = Map::new();
    let mut model = NaiveAddSum::new();
    let mut rng = StdRng::seed_from_u64(0x5eed_0000_dead_beef);

    for step in 0..50_000u32 {
        let key = rng.random_range(0..64i64);
        match rng.random_range(0..8u32) {
            0 | 1 => {
                let value = rng.random_range(-100..100i64);
                seg.put(key, add_value(value));
                model.put(key, value);
            }
            2 => {
                assert_eq!(
                    seg.remove(&key).map(|s| s.0),
                    model.remove(key),
                    "step {step}: remove {key}"
                );
            }
            3 | 4 => {
                let l = rng.random_range(0..64i64);
                let r = rng.random_range(l..=64);
                let f = rng.random_range(-10..10i64);
                seg.apply(l..r, f);
                model.apply(l..r, f);
            }
            5 | 6 => {
                let l = rng.random_range(0..64i64);
                let r = rng.random_range(l..=64);
                assert_eq!(
                    seg.prod(l..r),
                    (model.prod(l..r), count_in(&model, l..r)),
                    "step {step}: prod {l}..{r}"
                );
            }
            _ => {
                assert_eq!(
                    seg.get(&key).copied(),
                    model.get(key).map(add_value),
                    "step {step}: get {key}"
                );
            }
        }

        if step % 251 == 0 {
            assert_add_sum_matches(&mut seg, &model, step);
        }
    }
    assert_add_sum_matches(&mut seg, &model, "final");
}

/// 区間アフィン・区間和 (mod 998244353)
enum AffineSum {}

const MOD: u64 = 998_244_353;

impl Monoid for AffineSum {
    /// (和, 要素数)
    type S = (u64, u64);

    fn op(a: &(u64, u64), b: &(u64, u64)) -> (u64, u64) {
        ((a.0 + b.0) % MOD, a.1 + b.1)
    }

    fn identity() -> (u64, u64) {
        (0, 0)
    }
}

impl Action for AffineSum {
    /// x -> a * x + b
    type F = (u64, u64);

    fn mapping(f: &(u64, u64), s: &(u64, u64)) -> (u64, u64) {
        ((f.0 * s.0 + f.1 * s.1) % MOD, s.1)
    }

    fn composition(f: &(u64, u64), g: &(u64, u64)) -> (u64, u64) {
        // g を適用したあとに f を適用する
        ((f.0 * g.0) % MOD, (f.0 * g.1 + f.1) % MOD)
    }

    fn id() -> (u64, u64) {
        (1, 0)
    }
}

/// 素朴な区間アフィン・区間和モデル
struct NaiveAffineSum {
    entries: BTreeMap<i64, u64>,
}

impl NaiveAffineSum {
    fn new() -> Self {
        NaiveAffineSum { entries: BTreeMap::new() }
    }

    fn put(&mut self, key: i64, value: u64) {
        self.entries.insert(key, value % MOD);
    }

    fn remove(&mut self, key: i64) -> Option<u64> {
        self.entries.remove(&key)
    }

    fn get(&self, key: i64) -> Option<u64> {
        self.entries.get(&key).copied()
    }

    fn apply(&mut self, range: Range<i64>, f: (u64, u64)) {
        for value in self.entries.range_mut(range).map(|(_, v)| v) {
            *value = (f.0 * *value + f.1) % MOD;
        }
    }

    fn prod(&self, range: Range<i64>) -> (u64, u64) {
        let mut sum = 0;
        let mut count = 0;
        for &value in self.entries.range(range).map(|(_, v)| v) {
            sum = (sum + value) % MOD;
            count += 1;
        }
        (sum, count)
    }
}

fn assert_affine_matches(
    seg: &mut Map<LazySegmentTree<i64, AffineSum>>,
    model: &NaiveAffineSum,
    label: impl std::fmt::Debug,
) {
    assert_eq!(seg.len(), model.entries.len(), "{label:?}: len");
    for (&key, &value) in &model.entries {
        assert_eq!(seg.get(&key).copied(), Some((value, 1)), "{label:?}: get {key}");
    }
    for l in 0..8i64 {
        for r in l..=8 {
            assert_eq!(
                seg.prod(l * 4..r * 4),
                model.prod(l * 4..r * 4),
                "{label:?}: prod {}..{}",
                l * 4,
                r * 4
            );
        }
    }
    assert_eq!(seg.all_prod(), model.prod(i64::MIN..i64::MAX), "{label:?}: all_prod");
    assert_invariants(seg, &format!("{label:?}"));
}

#[test]
fn range_affine_sum_matches_naive() {
    let mut seg: Map<LazySegmentTree<i64, AffineSum>> = Map::new();
    let mut model = NaiveAffineSum::new();
    let mut rng = StdRng::seed_from_u64(0xaff1_0000_1234_5678);

    for step in 0..30_000u32 {
        let key = rng.random_range(0..32i64);
        match rng.random_range(0..8u32) {
            0 | 1 => {
                let value = rng.random_range(0..MOD);
                seg.put(key, (value, 1));
                model.put(key, value);
            }
            2 => {
                assert_eq!(
                    seg.remove(&key),
                    model.remove(key).map(|value| (value, 1)),
                    "step {step}: remove {key}"
                );
            }
            3 | 4 => {
                let l = rng.random_range(0..32i64);
                let r = rng.random_range(l..=32);
                let a = rng.random_range(0..MOD);
                let b = rng.random_range(0..MOD);
                seg.apply(l..r, (a, b));
                model.apply(l..r, (a, b));
            }
            5 | 6 => {
                let l = rng.random_range(0..32i64);
                let r = rng.random_range(l..=32);
                assert_eq!(seg.prod(l..r), model.prod(l..r), "step {step}: prod {l}..{r}");
            }
            _ => {
                assert_eq!(
                    seg.get(&key).copied(),
                    model.get(key).map(|value| (value, 1)),
                    "step {step}: get {key}"
                );
            }
        }

        if step % 251 == 0 {
            assert_affine_matches(&mut seg, &model, step);
        }
    }
    assert_affine_matches(&mut seg, &model, "final");
}

#[test]
fn rotations_preserve_pending_actions() {
    let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
    seg.put(0, add_value(0));
    seg.apply(0..100, 7);

    // 全体に作用が溜まった状態で、回転が起きるように昇順に挿入する
    for key in 1..64 {
        seg.put(key, add_value(0));
    }
    for key in 0..64 {
        let want = if key == 0 { 7 } else { 0 };
        assert_eq!(seg.get(&key).copied(), Some(add_value(want)), "key {key}");
    }
    assert_eq!(seg.prod(0..64), (7, 64));
    assert_invariants(&seg, "rotations after insert");

    // さらに作用を重ねてから、回転が起きるように削除する
    seg.apply(0..64, 1);
    for key in 0..32 {
        let want = if key == 0 { 8 } else { 1 };
        assert_eq!(seg.remove(&key).map(|s| s.0), Some(want), "remove key {key}");
    }
    assert_eq!(seg.prod(0..64), (32, 32));
    for key in 32..64 {
        assert_eq!(seg.get(&key).copied(), Some(add_value(1)), "remaining key {key}");
    }
    assert_invariants(&seg, "rotations after delete");
}

#[test]
fn insert_inside_pending_range_is_not_affected() {
    let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
    seg.put(0, add_value(1));
    seg.put(2, add_value(2));
    seg.apply(0..3, 10);

    seg.put(1, add_value(5));

    assert_eq!(seg.get(&0).copied(), Some(add_value(11)));
    assert_eq!(seg.get(&1).copied(), Some(add_value(5)));
    assert_eq!(seg.get(&2).copied(), Some(add_value(12)));
    assert_eq!(seg.prod(0..3), (28, 3));
    assert_invariants(&seg, "insert inside pending range");
}

#[test]
fn put_replaces_without_inheriting_pending_action() {
    let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
    seg.put(1, add_value(10));
    seg.apply(0..10, 100);
    assert_eq!(seg.get(&1).copied(), Some(add_value(110)));

    seg.put(1, add_value(1));
    assert_eq!(seg.get(&1).copied(), Some(add_value(1)));
    assert_eq!(seg.prod(0..10), (1, 1));
    assert_invariants(&seg, "replace inside pending range");
}

#[test]
fn remove_with_pending_actions_returns_applied_value() {
    let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
    for key in 0..15 {
        seg.put(key, add_value(i64::from(key)));
    }
    seg.apply(0..15, 100);

    // 子を2つ持つノードを含めて削除する
    assert_eq!(seg.remove(&7).map(|s| s.0), Some(107));
    assert_eq!(seg.remove(&3).map(|s| s.0), Some(103));
    assert_eq!(seg.remove(&11).map(|s| s.0), Some(111));
    assert_eq!(seg.remove(&1).map(|s| s.0), Some(101));

    let removed = [1, 3, 7, 11];
    let want: i64 =
        (0..15).filter(|key| !removed.contains(key)).map(|key| i64::from(key) + 100).sum();
    assert_eq!(seg.prod(0..15), (want, 11));
    for key in 0..15 {
        if removed.contains(&key) {
            assert_eq!(seg.get(&key), None, "removed key {key}");
        } else {
            let want = i64::from(key) + 100;
            assert_eq!(seg.get(&key).copied(), Some(add_value(want)), "remaining key {key}");
        }
    }
    assert_invariants(&seg, "remove with pending actions");
}

#[test]
fn key_ranges_with_gaps() {
    let mut seg: Map<LazySegmentTree<i64, AddSum>> = Map::new();
    for key in [10, 20, 30, 40] {
        seg.put(key, add_value(1));
    }

    seg.apply(15..35, 10);

    assert_eq!(seg.get(&10).copied(), Some(add_value(1)));
    assert_eq!(seg.get(&20).copied(), Some(add_value(11)));
    assert_eq!(seg.get(&30).copied(), Some(add_value(11)));
    assert_eq!(seg.get(&40).copied(), Some(add_value(1)));
    assert_eq!(seg.prod(0..15), (1, 1));
    assert_eq!(seg.prod(15..35), (22, 2));
    assert_eq!(seg.prod(15..25), (11, 1));
    assert_eq!(seg.prod(35..i64::MAX), (1, 1));
    assert_eq!(seg.prod(20..20), (0, 0));
    assert_invariants(&seg, "sparse keys");
}

#[test]
fn indexed_lazy_segment_tree_combines_index_and_range() {
    let mut seg: Map<Indexed<LazySegmentTree<usize, AddSum>>> = Map::new();
    for key in 0..16usize {
        seg.put(key, add_value(key as i64));
    }
    seg.apply(4..12, 100);
    seg.apply(8..10, 1);

    let want = |key: usize| {
        key as i64
            + if (4..12).contains(&key) { 100 } else { 0 }
            + if (8..10).contains(&key) { 1 } else { 0 }
    };

    for key in 0..16usize {
        assert_eq!(seg.get(&key).copied(), Some(add_value(want(key))), "key {key}");
    }

    // index アクセス (順序統計) も使える
    for index in 0..16usize {
        let slot = seg.slot_by_index(index).unwrap();
        assert_eq!(seg.index_of(slot), index, "rank {index}");
        assert_eq!(*unsafe { seg.slot_ref(slot) }.inner().key(), index, "key at {index}");
    }

    let total: i64 = (0..16).map(want).sum();
    assert_eq!(seg.prod(0..16), (total, 16));
    assert_eq!(seg.all_prod(), (total, 16));
    assert_invariants(&seg, "indexed lazy segment tree");

    // index アクセスと削除の併用
    assert_eq!(seg.remove(&5).map(|s| s.0), Some(105));
    let slot = seg.slot_by_index(5).unwrap();
    assert_eq!(*unsafe { seg.slot_ref(slot) }.inner().key(), 6);
    assert_eq!(seg.index_of(slot), 5);
    assert_eq!(seg.prod(0..16), (total - 105, 15));
    assert_invariants(&seg, "indexed lazy after remove");
}

#[test]
fn indexed_lazy_segment_tree_matches_naive() {
    let mut seg: Map<Indexed<LazySegmentTree<i64, AddSum>>> = Map::new();
    let mut model = NaiveAddSum::new();
    let mut rng = StdRng::seed_from_u64(0x1ded_0000_cafe_babe);

    for step in 0..20_000u32 {
        let key = rng.random_range(0..32i64);
        match rng.random_range(0..8u32) {
            0 | 1 => {
                let value = rng.random_range(-100..100i64);
                seg.put(key, add_value(value));
                model.put(key, value);
            }
            2 => {
                assert_eq!(
                    seg.remove(&key).map(|s| s.0),
                    model.remove(key),
                    "step {step}: remove {key}"
                );
            }
            3 | 4 => {
                let l = rng.random_range(0..32i64);
                let r = rng.random_range(l..=32);
                let f = rng.random_range(-10..10i64);
                seg.apply(l..r, f);
                model.apply(l..r, f);
            }
            5 | 6 => {
                let l = rng.random_range(0..32i64);
                let r = rng.random_range(l..=32);
                assert_eq!(
                    seg.prod(l..r),
                    (model.prod(l..r), count_in(&model, l..r)),
                    "step {step}: prod {l}..{r}"
                );
            }
            _ => {
                assert_eq!(
                    seg.get(&key).copied(),
                    model.get(key).map(add_value),
                    "step {step}: get {key}"
                );
            }
        }

        if step % 251 == 0 {
            assert_eq!(seg.len(), model.entries.len(), "step {step}: len");
            assert_invariants(&seg, &format!("step {step}"));
            for (index, (&key, &value)) in model.entries.iter().enumerate() {
                let slot = seg.slot_by_index(index).unwrap();
                assert_eq!(*unsafe { seg.slot_ref(slot) }.inner().key(), key, "step {step}");
                assert_eq!(seg.index_of(slot), index, "step {step}");
                assert_eq!(
                    seg.get(&key).copied(),
                    Some(add_value(value)),
                    "step {step}: get {key}"
                );
            }
        }
    }
}

#[test]
fn lazy_segment_tree_edges() {
    let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
    assert_eq!(seg.prod(-10..10), (0, 0));
    assert_eq!(seg.all_prod(), (0, 0));
    assert_eq!(seg.get(&0), None);
    assert_eq!(seg.remove(&0), None);
    seg.apply(-10..10, 5);

    seg.put(5, add_value(7));
    assert_eq!(seg.prod(-10..5), (0, 0));
    assert_eq!(seg.prod(-10..6), (7, 1));
    assert_eq!(seg.prod(5..5), (0, 0));
    assert_eq!(seg.prod(6..10), (0, 0));
    seg.apply(-10..5, 3);
    seg.apply(6..10, 3);
    assert_eq!(seg.prod(-10..6), (7, 1));
    assert_eq!(seg.remove(&5).map(|s| s.0), Some(7));

    // 空区間に対する操作は何もしない
    seg.put(1, add_value(1));
    seg.apply(5..5, 100);
    assert_eq!(seg.get(&1).copied(), Some(add_value(1)));

    assert_eq!(seg.remove(&1).map(|s| s.0), Some(1));
    assert!(seg.is_empty());
    assert_eq!(seg.all_prod(), (0, 0));
    assert_invariants(&seg, "edges");
}
