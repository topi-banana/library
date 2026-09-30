use super::*;

use std::ops::Range;

/// 重なる区間の値を足し合わせる
///
/// 削除で区間が割れると値が両断片に残り、再びまとまると重複して足される。
/// 値を保存しないので、モデルと突き合わせるために wrapping で足す。
enum Sum {}

impl Merge for Sum {
    type S = i64;

    fn merge(a: &i64, b: &i64) -> i64 {
        a.wrapping_add(*b)
    }

    fn identity() -> i64 {
        0
    }
}

type Set = Map<IntervalSet<i64, Sum>>;

/// 素朴な区間集合モデル。重なりも接触もなく、値は sum 済み
fn model_insert(model: &mut Vec<(i64, i64, i64)>, range: Range<i64>, value: i64) {
    let (mut start, mut end) = (range.start, range.end);
    if start >= end {
        return;
    }
    let mut total = value;
    model.retain(|&(s, e, v)| {
        if s <= end && start <= e {
            start = start.min(s);
            end = end.max(e);
            total = total.wrapping_add(v);
            false
        } else {
            true
        }
    });
    model.push((start, end, total));
    model.sort_unstable();
}

fn model_remove(model: &mut Vec<(i64, i64, i64)>, range: Range<i64>) {
    let (start, end) = (range.start, range.end);
    if start >= end {
        return;
    }
    let mut next = Vec::with_capacity(model.len() + 1);
    for &(s, e, v) in model.iter() {
        if e <= start || end <= s {
            next.push((s, e, v));
            continue;
        }
        // はみ出した断片は元の値を引き継ぐ
        if s < start {
            next.push((s, start, v));
        }
        if e > end {
            next.push((end, e, v));
        }
    }
    next.sort_unstable();
    *model = next;
}

fn assert_set_matches(set: &Set, model: &[(i64, i64, i64)], label: impl std::fmt::Debug) {
    assert_eq!(set.len(), model.len(), "{label:?}: len");
    assert_eq!(set.is_empty(), model.is_empty(), "{label:?}: is_empty");
    let got = entries(set);
    assert_eq!(got, model, "{label:?}: intervals");
    assert_invariants(set, &format!("{label:?}"));

    for point in -2..70i64 {
        let covering = model.iter().find(|&&(s, e, _)| s <= point && point < e);
        assert_eq!(
            set.get_point(&point).copied(),
            covering.map(|&(_, _, v)| v),
            "{label:?}: get {point}"
        );
        assert_eq!(set.covers(&point), covering.is_some(), "{label:?}: covers {point}");
        assert_eq!(
            set.get_point_or_identity(&point),
            covering.map_or(0, |&(_, _, v)| v),
            "{label:?}: get_or_identity {point}"
        );
        match covering {
            Some(&(s, e, v)) => {
                let (interval, &value) = set.covering(&point).unwrap();
                assert_eq!(
                    (*interval.start(), *interval.end(), value),
                    (s, e, v),
                    "{label:?}: covering {point}"
                );
            }
            None => assert!(set.covering(&point).is_none(), "{label:?}: covering {point}"),
        }
    }

    for l in -2..70i64 {
        for r in -2..70i64 {
            if l >= r {
                assert!(set.contains_range(&(l..r)), "{label:?}: contains_range {l}..{r}");
                assert!(!set.overlaps(&(l..r)), "{label:?}: overlaps {l}..{r}");
                continue;
            }
            let covered = model.iter().any(|&(s, e, _)| s <= l && r <= e);
            assert_eq!(set.contains_range(&(l..r)), covered, "{label:?}: contains_range {l}..{r}");
            let overlaps = model.iter().any(|&(s, e, _)| s < r && l < e);
            assert_eq!(set.overlaps(&(l..r)), overlaps, "{label:?}: overlaps {l}..{r}");
        }
    }
}

/// `set` の区間を `(start, end, value)` の列にする
fn entries(set: &Set) -> Vec<(i64, i64, i64)> {
    set.iter_intervals()
        .map(|(interval, &value)| (*interval.start(), *interval.end(), value))
        .collect()
}

#[test]
fn insert_remove_matches_model() {
    let mut set: Set = Map::new();
    let mut model: Vec<(i64, i64, i64)> = Vec::new();
    let mut rng = StdRng::seed_from_u64(0x1357_9bdf_2468_ace0);

    for step in 0..20_000 {
        let start = rng.random_range(0..64i64);
        let len = rng.random_range(0..8i64);
        let range = start..start + len;
        if rng.random_bool(0.55) {
            let value = rng.random_range(-10..10i64);
            set.insert_range(range.clone(), value);
            model_insert(&mut model, range, value);
        } else {
            set.remove_range(range.clone());
            model_remove(&mut model, range);
        }
        if step % 251 == 0 {
            assert_set_matches(&set, &model, step);
        }
    }
    assert_set_matches(&set, &model, "final");
}

#[test]
fn insert_merges_touching_and_overlapping() {
    let mut set: Set = Map::new();
    set.insert_range(0..3, 1);
    set.insert_range(3..5, 2); // 接する
    set.insert_range(4..7, 3); // 重なる
    set.insert_range(10..12, 4); // 離れている
    set.insert_range(12..14, 5); // 接する

    let got = entries(&set);
    assert_eq!(got, vec![(0, 7, 6), (10, 14, 9)]);
}

#[test]
fn insert_keeps_gap_separate() {
    let mut set: Set = Map::new();
    set.insert_range(0..2, 1);
    set.insert_range(3..5, 2); // 1 つ分空いているのでまとまらない
    assert_eq!(set.len(), 2);
    assert_eq!(set.get_point(&2), None);
}

#[test]
fn insert_same_range_merges_values() {
    let mut set: Set = Map::new();
    set.insert_range(0..5, 1);
    set.insert_range(0..5, 2);
    assert_eq!(set.len(), 1);
    assert_eq!(set.get_point(&3), Some(&3));
}

#[test]
fn remove_trims_and_splits_keeping_value() {
    let mut set: Set = Map::new();
    set.insert_range(0..10, 5);

    set.remove_range(3..7); // 中央をくり抜くと 2 本に割れる
    let got = entries(&set);
    assert_eq!(got, vec![(0, 3, 5), (7, 10, 5)]);

    set.remove_range(8..12); // 右端を切り詰める
    let got = entries(&set);
    assert_eq!(got, vec![(0, 3, 5), (7, 8, 5)]);

    set.remove_range(0..2); // 左端を切り詰める
    let got = entries(&set);
    assert_eq!(got, vec![(2, 3, 5), (7, 8, 5)]);
}

#[test]
fn remove_drops_covered_intervals() {
    let mut set: Set = Map::new();
    set.insert_range(0..3, 1);
    set.insert_range(5..8, 2);
    set.insert_range(10..15, 3);

    set.remove_range(4..9); // 真ん中を丸ごと消す
    let got = entries(&set);
    assert_eq!(got, vec![(0, 3, 1), (10, 15, 3)]);

    set.remove_range(0..100); // 全部消す
    assert!(set.is_empty());
}

#[test]
#[allow(clippy::reversed_empty_ranges)] // 空区間が無視されることを確かめる
fn empty_ranges_are_ignored() {
    let mut set: Set = Map::new();
    set.insert_range(5..5, 1);
    set.insert_range(7..3, 2);
    assert!(set.is_empty());

    set.insert_range(0..10, 3);
    set.remove_range(4..4);
    set.remove_range(9..2);
    assert_eq!(set.len(), 1);
    assert_eq!(set.get_point(&5), Some(&3));
}

/// 値の合成が新しい値側から始まることを確かめるための連結
enum Concat {}

impl Merge for Concat {
    type S = String;

    fn merge(a: &String, b: &String) -> String {
        format!("{a}{b}")
    }

    fn identity() -> String {
        String::new()
    }
}

#[test]
fn merge_combines_new_value_first() {
    let mut set: Map<IntervalSet<i64, Concat>> = Map::new();
    set.insert_range(0..5, String::from("one"));
    // 追加する "two" が左、吸収される "one" が右
    set.insert_range(5..10, String::from("two"));

    assert_eq!(set.get_point(&3), Some(&String::from("twoone")));

    set.insert_range(10..15, String::from("three"));
    assert_eq!(set.get_point(&12), Some(&String::from("threetwoone")));
}

#[test]
fn works_with_non_copy_keys() {
    let mut set: Map<IntervalSet<String, Sum>> = Map::new();
    set.insert_range(String::from("a")..String::from("c"), 1);
    set.insert_range(String::from("c")..String::from("e"), 2); // 端点が接する

    assert_eq!(set.len(), 1);
    assert_eq!(set.get_point(&String::from("b")), Some(&3));
    let (interval, &value) = set.covering(&String::from("d")).unwrap();
    assert_eq!(
        (interval.start().clone(), interval.end().clone()),
        (String::from("a"), String::from("e"))
    );
    assert_eq!(value, 3);

    set.remove_range(String::from("b")..String::from("d"));
    let got: Vec<(String, String, i64)> = set
        .iter_intervals()
        .map(|(interval, &value)| (interval.start().clone(), interval.end().clone(), value))
        .collect();
    assert_eq!(
        got,
        vec![(String::from("a"), String::from("b"), 3), (String::from("d"), String::from("e"), 3)]
    );
}

#[test]
fn iter_is_double_ended() {
    let mut set: Set = Map::new();
    set.insert_range(0..2, 1);
    set.insert_range(5..7, 2);
    set.insert_range(10..12, 3);

    let starts: Vec<i64> = set.iter_intervals().map(|(interval, _)| *interval.start()).collect();
    assert_eq!(starts, vec![0, 5, 10]);
    assert_eq!(set.iter_intervals().len(), 3);

    let reversed: Vec<i64> =
        set.iter_intervals().rev().map(|(interval, _)| *interval.start()).collect();
    assert_eq!(reversed, vec![10, 5, 0]);
}
