use super::{next_permutation, permutation, prev_permutation};

#[test]
fn next_permutation_enumeration() {
    let mut a = [1, 2, 3];
    let mut ps = vec![a.to_vec()];
    while next_permutation(&mut a) {
        ps.push(a.to_vec());
    }
    assert_eq!(
        ps,
        vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1]
        ]
    );
}

#[test]
fn next_permutation_last_returns_false_and_keeps_input() {
    let mut a = [3, 2, 1];
    assert!(!next_permutation(&mut a));
    assert_eq!(a, [3, 2, 1]);
}

#[test]
fn next_permutation_short_slices() {
    let mut empty: [i32; 0] = [];
    assert!(!next_permutation(&mut empty));
    let mut one = [1];
    assert!(!next_permutation(&mut one));
    assert_eq!(one, [1]);
}

#[test]
fn prev_permutation_enumeration() {
    let mut a = [3, 2, 1];
    let mut ps = vec![a.to_vec()];
    while prev_permutation(&mut a) {
        ps.push(a.to_vec());
    }
    assert_eq!(
        ps,
        vec![
            vec![3, 2, 1],
            vec![3, 1, 2],
            vec![2, 3, 1],
            vec![2, 1, 3],
            vec![1, 3, 2],
            vec![1, 2, 3]
        ]
    );
}

#[test]
fn prev_permutation_first_returns_false_and_keeps_input() {
    let mut a = [1, 2, 3];
    assert!(!prev_permutation(&mut a));
    assert_eq!(a, [1, 2, 3]);
}

#[test]
fn next_and_prev_are_inverse() {
    let a = [2, 1, 4, 3];
    let mut forward = a;
    assert!(next_permutation(&mut forward));
    let mut backward = forward;
    assert!(prev_permutation(&mut backward));
    assert_eq!(backward, a);
}

#[test]
fn permutation_distinct_elements() {
    let mut counts = [(1, 1), (2, 1), (3, 1)];
    let mut ps = permutation(&mut counts, 3);
    ps.sort();
    assert_eq!(
        ps,
        vec![
            vec![1, 2, 3],
            vec![1, 3, 2],
            vec![2, 1, 3],
            vec![2, 3, 1],
            vec![3, 1, 2],
            vec![3, 2, 1]
        ]
    );
    // 呼び出し後は元の個数へ戻っている。
    assert_eq!(counts, [(1, 1), (2, 1), (3, 1)]);
}

#[test]
fn permutation_with_duplicates() {
    let mut counts = [(1, 2), (2, 1)];
    let mut ps = permutation(&mut counts, 3);
    ps.sort();
    assert_eq!(ps, vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]]);
}

#[test]
fn permutation_empty_and_too_long() {
    let mut counts = [(1, 1)];
    assert_eq!(permutation(&mut counts, 0), vec![vec![]]);
    assert_eq!(permutation(&mut counts, 2), Vec::<Vec<i32>>::new());
}
