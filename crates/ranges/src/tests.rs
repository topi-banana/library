use super::*;

fn vec_of(ranges: &Ranges<i32>) -> Vec<(i32, i32)> {
    ranges.iter().map(|(s, e)| (*s, *e)).collect()
}

#[test]
fn merge_on_insert() {
    let mut r = Ranges::new();
    r.insert(5..10);
    r.insert(1..3);
    assert_eq!(vec_of(&r), vec![(1, 3), (5, 10)]);

    r.insert(3..5); // 両隣に接するので 1 本になる
    assert_eq!(vec_of(&r), vec![(1, 10)]);

    r.insert(20..30);
    r.insert(0..25); // 既存を跨いで飲み込む
    assert_eq!(vec_of(&r), vec![(0, 30)]);

    r.insert(7..8); // 完全に内包される
    assert_eq!(vec_of(&r), vec![(0, 30)]);

    r.insert(40..40); // 空区間は無視
    assert_eq!(vec_of(&r), vec![(0, 30)]);
}

#[test]
fn no_merge_for_disjoint_integers() {
    let r: Ranges<i32> = [1..2, 3..4].into_iter().collect();
    assert_eq!(vec_of(&r), vec![(1, 2), (3, 4)]);
}

#[test]
// 1 本だけの Range 配列は「範囲そのものの Vec の書き間違い」と見なされるが、
// ここは意図して 1 区間だけを流し込んでいる。
#[allow(clippy::single_range_in_vec_init)]
fn remove_splits_and_truncates() {
    let mut r: Ranges<i32> = [1..10].into_iter().collect();
    r.remove(3..5);
    assert_eq!(vec_of(&r), vec![(1, 3), (5, 10)]);

    r.remove(2..7); // 左を切り詰め、右の頭を削る
    assert_eq!(vec_of(&r), vec![(1, 2), (7, 10)]);

    r.remove(0..100);
    assert!(r.is_empty());
}

#[test]
fn remove_across_multiple() {
    let mut r: Ranges<i32> = [0..5, 10..15, 20..25].into_iter().collect();
    r.remove(3..22);
    assert_eq!(vec_of(&r), vec![(0, 3), (22, 25)]);
}

#[test]
fn queries() {
    let r: Ranges<i32> = [1..5, 10..20].into_iter().collect();
    assert!(r.contains(&1));
    assert!(r.contains(&4));
    assert!(!r.contains(&5)); // 半開区間なので end は含まない
    assert!(!r.contains(&9));
    assert_eq!(r.covering(&12), Some((&10, &20)));

    assert!(r.contains_range(&(10..20)));
    assert!(!r.contains_range(&(4..11)));
    assert!(r.overlaps(&(4..11)));
    assert!(!r.overlaps(&(5..10)));
}

#[test]
fn immutable_view() {
    let r: Ranges<i32> = [10..20, 1..5, 3..7].into_iter().collect();
    let frozen: ImmutableRanges<_> = r.into();
    assert_eq!(frozen.as_slice(), &[(1, 7), (10, 20)]);

    assert!(!frozen.contains(&0));
    assert!(frozen.contains(&1));
    assert!(frozen.contains(&6));
    assert!(!frozen.contains(&7));
    assert!(frozen.contains(&19));
    assert!(!frozen.contains(&20));
    assert_eq!(frozen.covering(&15), Some(&(10, 20)));
    assert_eq!(frozen.covering_index(&3), Some(0));

    assert!(frozen.overlaps(&(6..11)));
    assert!(!frozen.overlaps(&(7..10)));
    assert!(frozen.contains_range(&(2..7)));
    assert!(!frozen.contains_range(&(2..8)));
}

#[test]
fn immutable_from_iter() {
    // 未ソート/重なり/接触/内包/空区間/同一 start が混ざった入力
    let input = [10..20, 3..7, 1..5, 5..5, 7..10, 3..4, 30..31];
    let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
    assert_eq!(frozen.as_slice(), &[(1, 20), (30, 31)]);

    // Ranges 経由と同じ結果になること
    assert_eq!(frozen, Ranges::from_iter(input).into());

    let (lo, hi) = (3, 1); // 逆転区間。リテラルだと clippy に怒られるので変数経由
    let empty: ImmutableRanges<i32> = [5..5, lo..hi].into_iter().collect();
    assert!(empty.is_empty());
}

#[test]
fn immutable_from_iter_touching_boundary() {
    let touching = [1..3, 3..5];
    let frozen: ImmutableRanges<i32> = touching.clone().into_iter().collect();
    assert_eq!(frozen.as_slice(), &[(1, 5)]);
    assert_eq!(frozen, Ranges::from_iter(touching).into());

    let disjoint = [1..2, 3..4];
    let frozen: ImmutableRanges<i32> = disjoint.clone().into_iter().collect();
    assert_eq!(frozen.as_slice(), &[(1, 2), (3, 4)]);
    assert_eq!(frozen, Ranges::from_iter(disjoint).into());
}

#[test]
fn immutable_from_iter_swallows_chain() {
    // 大区間が後続を次々飲み込む。畳み込みが「残った区間」と比較していないと分裂する
    let input = [0..100, 10..20, 30..40, 50..60];
    let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
    assert_eq!(frozen.as_slice(), &[(0, 100)]);
    assert_eq!(frozen, Ranges::from_iter(input).into());

    // 途中で end が伸びていく鎖
    let input = [0..10, 5..20, 15..30, 100..101];
    let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
    assert_eq!(frozen.as_slice(), &[(0, 30), (100, 101)]);
    assert_eq!(frozen, Ranges::from_iter(input).into());
}

#[test]
fn immutable_from_iter_without_clone() {
    // Clone を要求しないことの確認 (Ranges 経由では通らない)
    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    struct NoClone(i32);

    let r: ImmutableRanges<NoClone> =
        [NoClone(1)..NoClone(3), NoClone(3)..NoClone(5)].into_iter().collect();
    assert_eq!(r.len(), 1);
}

#[test]
fn works_with_non_integer_keys() {
    let mut r = Ranges::new();
    r.insert("a".to_string().."m".to_string());
    r.insert("m".to_string().."z".to_string());
    assert_eq!(r.len(), 1);
    assert!(r.contains(&"q".to_string()));
}
