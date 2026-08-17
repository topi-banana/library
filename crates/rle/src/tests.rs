use super::{Rle, rle};

fn collect(s: &str) -> Vec<(char, usize)> {
    s.chars().rle().collect()
}

#[test]
fn empty() {
    assert!(rle(&mut std::iter::empty::<char>()).is_empty());
    assert!(collect("").is_empty());
}

#[test]
fn single_element() {
    assert_eq!(rle(&mut "a".chars()), vec![('a', 1)]);
    assert_eq!(collect("a"), vec![('a', 1)]);
}

#[test]
fn all_same() {
    assert_eq!(rle(&mut "aaaa".chars()), vec![('a', 4)]);
    assert_eq!(collect("aaaa"), vec![('a', 4)]);
}

#[test]
fn value_can_appear_in_multiple_runs() {
    let expected = vec![('a', 1), ('b', 2), ('a', 3)];
    assert_eq!(rle(&mut "abbaaa".chars()), expected);
    assert_eq!(collect("abbaaa"), expected);
}

#[test]
fn no_run_is_merged_with_its_neighbor() {
    let expected = vec![('a', 1), ('b', 1), ('a', 1), ('b', 1)];
    assert_eq!(rle(&mut "abab".chars()), expected);
    assert_eq!(collect("abab"), expected);
}

#[test]
fn counts_sum_to_the_original_length() {
    let s = "aabbbcaadddde";
    let runs = collect(s);
    assert_eq!(runs.iter().map(|&(_, cnt)| cnt).sum::<usize>(), s.len());
    // 隣り合う区間の値は必ず異なる。
    assert!(runs.windows(2).all(|w| w[0].0 != w[1].0));
}

#[test]
fn iterator_is_lazy() {
    // 無限列でも、取り出した分の区間を確定させるところまでしか進まない。
    let runs: Vec<_> = (0..).map(|i| i / 2).rle().take(3).collect();
    assert_eq!(runs, vec![(0, 2), (1, 2), (2, 2)]);
}

#[test]
fn iterator_stays_exhausted() {
    let mut iter = "ab".chars().rle();
    assert_eq!(iter.next(), Some(('a', 1)));
    assert_eq!(iter.next(), Some(('b', 1)));
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
}

#[test]
fn function_and_adapter_agree() {
    let xs = [3, 3, 1, 1, 1, 4, 1, 5, 5, 5, 5, 9, 2, 2];
    assert_eq!(rle(&mut xs.into_iter()), xs.into_iter().rle().collect::<Vec<_>>());
}

#[test]
fn function_leaves_the_source_exhausted() {
    let mut iter = "aab".chars();
    assert_eq!(rle(&mut iter), vec![('a', 2), ('b', 1)]);
    assert_eq!(iter.next(), None);
}
