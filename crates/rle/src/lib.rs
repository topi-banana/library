//! 連長圧縮 (Run-Length Encoding)。
//!
//! 列を「同じ値が連続する区間」ごとに `(値, 連続する個数)` へまとめる。
//!
//! 一括で [`Vec`] を作る [`rle`] と、遅延評価のイテレータアダプタ [`Rle::rle`] がある。
//!
//! ```
//! use rle::Rle;
//!
//! assert_eq!(
//!     rle::rle(&mut "aaabbc".chars()),
//!     vec![('a', 3), ('b', 2), ('c', 1)]
//! );
//!
//! let mut iter = "aaabbc".chars().rle();
//! assert_eq!(iter.next(), Some(('a', 3)));
//! assert_eq!(iter.next(), Some(('b', 2)));
//! assert_eq!(iter.next(), Some(('c', 1)));
//! assert_eq!(iter.next(), None);
//! ```

/// 列を連長圧縮し、`(値, 連続する個数)` を並べた [`Vec`] を返す。
///
/// 元の列の長さを `n` として `O(n)` 時間。返る [`Vec`] の長さは連続する区間の個数で、
/// 高々 `n` になる。空の列に対しては空の [`Vec`] を返す。
///
/// # Examples
///
/// ```
/// assert_eq!(
///     rle::rle(&mut [1, 1, 2, 1].into_iter()),
///     vec![(1, 2), (2, 1), (1, 1)]
/// );
/// assert!(rle::rle(&mut std::iter::empty::<u8>()).is_empty());
/// ```
pub fn rle<T: Iterator<Item = I>, I: PartialEq>(iter: &mut T) -> Vec<(I, usize)> {
    let mut res = Vec::new();
    let Some(mut pre) = iter.next() else {
        return res;
    };
    let mut cnt = 1usize;
    for now in iter {
        if now != pre {
            // 区間が切り替わるので、直前の区間を確定させて cnt を 0 に戻す。
            res.push((pre, std::mem::take(&mut cnt)));
        }
        pre = now;
        cnt += 1;
    }
    res.push((pre, cnt));
    res
}

/// [`Rle::rle`] が返すイテレータ。
#[derive(Debug, Clone)]
pub struct RleIter<T, I> {
    iter: T,
    /// 出力待ちの区間の値。元のイテレータが尽きたあとは `None`。
    pre: Option<I>,
    /// `pre` と同じ値がここまでに何個続いたか。
    cnt: usize,
}

impl<T: Iterator<Item = I>, I: PartialEq> Iterator for RleIter<T, I> {
    type Item = (I, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let pre = self.pre.take()?;
        for now in self.iter.by_ref() {
            if now != pre {
                let res = (pre, self.cnt);
                self.pre = Some(now);
                self.cnt = 1;
                return Some(res);
            }
            self.cnt += 1;
        }
        // 元のイテレータが尽きた。`pre` は take 済みなので次回以降は None を返す。
        Some((pre, std::mem::take(&mut self.cnt)))
    }
}

/// イテレータに連長圧縮のアダプタを生やす拡張トレイト。
///
/// [`Iterator`] を実装するすべての型に対して実装されている。
pub trait Rle: Iterator + Sized {
    /// 連長圧縮した [`RleIter`] を返す。
    ///
    /// 各要素は `(値, 連続する個数)` で、個数は必ず 1 以上になる。
    /// 元の列が空なら、返るイテレータも空になる。
    ///
    /// 1 要素を返すのに必要な分だけ元のイテレータを進めるため、
    /// 無限列に対しても使える。ただし呼び出した時点で先頭の 1 要素を読むので、
    /// 副作用のあるイテレータに対しては最初の [`Iterator::next`] より前に 1 要素進む。
    ///
    /// # Examples
    ///
    /// ```
    /// use rle::Rle;
    ///
    /// let runs: Vec<_> = [1, 1, 2, 1].into_iter().rle().collect();
    /// assert_eq!(runs, vec![(1, 2), (2, 1), (1, 1)]);
    ///
    /// // 無限列でも先頭から順に取り出せる。
    /// let heads: Vec<_> = (0..).map(|i| i / 3).rle().take(2).collect();
    /// assert_eq!(heads, vec![(0, 3), (1, 3)]);
    /// ```
    fn rle(mut self) -> RleIter<Self, Self::Item>
    where
        Self::Item: PartialEq,
    {
        let pre = self.next();
        let cnt = usize::from(pre.is_some());
        RleIter {
            iter: self,
            pre,
            cnt,
        }
    }
}

impl<T: Iterator> Rle for T {}

#[cfg(test)]
mod tests {
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
        assert_eq!(
            rle(&mut xs.into_iter()),
            xs.into_iter().rle().collect::<Vec<_>>()
        );
    }

    #[test]
    fn function_leaves_the_source_exhausted() {
        let mut iter = "aab".chars();
        assert_eq!(rle(&mut iter), vec![('a', 2), ('b', 1)]);
        assert_eq!(iter.next(), None);
    }
}
