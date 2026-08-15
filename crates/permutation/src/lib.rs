//! 順列 (permutation)。
//!
//! 辞書順で次/前の順列へ遷移する [`next_permutation`] / [`prev_permutation`] と、
//! 多重集合の順列を全列挙する [`permutation`] がある。
//!
//! ```
//! let mut a = [1, 2, 3];
//! assert!(permutation::next_permutation(&mut a));
//! assert_eq!(a, [1, 3, 2]);
//! ```

/// 辞書順で次に大きい順列へ遷移し、`true` を返す。
///
/// `a` が既に最大の順列 (降順) のときは `false` を返し、`a` は変更しない。
/// C++ の `std::next_permutation` と違い、最小の順列へ折り返さない。
///
/// ```
/// let mut a = [1, 2, 3];
/// assert!(permutation::next_permutation(&mut a));
/// assert_eq!(a, [1, 3, 2]);
/// assert!(permutation::next_permutation(&mut a));
/// assert_eq!(a, [2, 1, 3]);
///
/// let mut last = [3, 2, 1];
/// assert!(!permutation::next_permutation(&mut last));
/// assert_eq!(last, [3, 2, 1]);
/// ```
pub fn next_permutation<T: Ord>(a: &mut [T]) -> bool {
    let Some(i) = a.windows(2).rposition(|w| unsafe { w.get_unchecked(0) < w.get_unchecked(1) })
    else {
        return false;
    };
    let j = a.iter().rposition(|x| x > unsafe { a.get_unchecked(i) }).unwrap();
    a.swap(i, j);
    unsafe { a.get_unchecked_mut(i + 1..) }.reverse();
    true
}

/// 辞書順で次に小さい順列へ遷移し、`true` を返す。
///
/// `a` が既に最小の順列 (昇順) のときは `false` を返し、`a` は変更しない。
/// C++ の `std::prev_permutation` と違い、最大の順列へ折り返さない。
///
/// ```
/// let mut a = [2, 1, 3];
/// assert!(permutation::prev_permutation(&mut a));
/// assert_eq!(a, [1, 3, 2]);
///
/// let mut first = [1, 2, 3];
/// assert!(!permutation::prev_permutation(&mut first));
/// assert_eq!(first, [1, 2, 3]);
/// ```
pub fn prev_permutation<T: Ord>(a: &mut [T]) -> bool {
    let Some(i) = a.windows(2).rposition(|w| unsafe { w.get_unchecked(0) > w.get_unchecked(1) })
    else {
        return false;
    };
    let j = a.iter().rposition(|x| x < unsafe { a.get_unchecked(i) }).unwrap();
    a.swap(i, j);
    unsafe { a.get_unchecked_mut(i + 1..) }.reverse();
    true
}

/// 多重集合 `counts` (要素と残り個数の組) から長さ `len` の順列を全て列挙する。
///
/// 各順列は `counts` の要素を `(値, 個数)` の個数だけ重複して並べたもので、
/// 個数は順列の個数分だけ消費される。`counts` の個数は呼び出し後に元へ戻る。
/// `counts` の合計個数が `len` 未満なら空の [`Vec`] を返し、`len == 0` なら
/// 空の順列 1 つ `[[]]` を返す。
///
/// ```
/// let mut counts = [(1, 2), (2, 1)];
/// let mut ps = permutation::permutation(&mut counts, 3);
/// ps.sort();
/// assert_eq!(
///     ps,
///     vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]]
/// );
///
/// let mut empty = [(1, 1)];
/// assert_eq!(permutation::permutation(&mut empty, 0), vec![vec![]]);
/// assert_eq!(permutation::permutation(&mut empty, 2), Vec::<Vec<i32>>::new());
/// ```
pub fn permutation<T: Ord + Clone>(counts: &mut [(T, usize)], len: usize) -> Vec<Vec<T>> {
    fn dfs<T: Ord + Clone>(
        counts: &mut [(T, usize)],
        indices: &mut Vec<usize>,
        result: &mut Vec<Vec<T>>,
        len: usize,
    ) {
        if indices.len() == len {
            let mut c = Vec::with_capacity(len);
            for &i in &*indices {
                c.push(unsafe { counts.get_unchecked(i) }.0.clone());
            }
            result.push(c);
            return;
        }

        for i in 0..counts.len() {
            if unsafe { counts.get_unchecked(i) }.1 > 0 {
                indices.push(i);
                unsafe { counts.get_unchecked_mut(i) }.1 -= 1;
                dfs(counts, indices, result, len);
                unsafe { counts.get_unchecked_mut(i) }.1 += 1;
                indices.pop();
            }
        }
    }

    let mut result = vec![];
    dfs(counts, &mut Vec::with_capacity(len), &mut result, len);
    result
}

#[cfg(test)]
mod tests {
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
}
