//! モノイドに対するセグメント木。
//!
//! 1 点更新と区間積の取得をどちらも `O(log n)` で行う。
//!
//! ```
//! use segtree::{Additive, Segtree};
//!
//! let mut seg: Segtree<Additive<i64>> = vec![1, 2, 3, 4, 5].into();
//! assert_eq!(seg.prod(1..4), 9);
//! seg.set(2, 10);
//! assert_eq!(seg.prod(1..4), 16);
//! assert_eq!(seg.all_prod(), 22);
//! ```

use std::marker::PhantomData;
use std::ops::{Add, Bound, RangeBounds};

/// 単位元と結合的な二項演算を持つ代数構造 (モノイド)。
pub trait Monoid {
    /// 台集合の型。
    type S: Clone;

    /// 単位元を返す。
    fn identity() -> Self::S;

    /// 二項演算。結合律 `op(op(a, b), c) == op(a, op(b, c))` を満たす必要がある。
    fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S;
}

/// 加法についてのモノイド。単位元は [`Default::default`] が返す値。
pub struct Additive<T>(PhantomData<fn() -> T>);

impl<T> Monoid for Additive<T>
where
    T: Copy + Default + Add<Output = T>,
{
    type S = T;

    fn identity() -> Self::S {
        T::default()
    }

    fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S {
        *a + *b
    }
}

/// モノイド `M` の要素列に対するセグメント木。
#[derive(Debug, Clone)]
pub struct Segtree<M: Monoid> {
    n: usize,
    size: usize,
    log: u32,
    data: Vec<M::S>,
}

impl<M: Monoid> Segtree<M> {
    /// 長さ `n` のセグメント木を単位元で初期化して作る。
    pub fn new(n: usize) -> Self {
        vec![M::identity(); n].into()
    }

    /// 列の長さを返す。
    pub fn len(&self) -> usize {
        self.n
    }

    /// 列が空かどうかを返す。
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// `p` 番目の要素を `x` に書き換える。
    ///
    /// # Panics
    ///
    /// `p >= self.len()` のときパニックする。
    pub fn set(&mut self, p: usize, x: M::S) {
        assert!(p < self.n, "添字が範囲外です: {p}");
        let p = p + self.size;
        self.data[p] = x;
        for i in 1..=self.log {
            self.update(p >> i);
        }
    }

    /// `p` 番目の要素を返す。
    ///
    /// # Panics
    ///
    /// `p >= self.len()` のときパニックする。
    pub fn get(&self, p: usize) -> M::S {
        assert!(p < self.n, "添字が範囲外です: {p}");
        self.data[p + self.size].clone()
    }

    /// `range` が表す区間の総積を返す。区間が空なら単位元を返す。
    ///
    /// # Panics
    ///
    /// 区間が `0..self.len()` に収まらない場合、または始点が終点より後ろの場合にパニックする。
    pub fn prod<R: RangeBounds<usize>>(&self, range: R) -> M::S {
        let (mut l, mut r) = self.to_half_open(&range);
        if l == r {
            return M::identity();
        }
        l += self.size;
        r += self.size;
        let mut sml = M::identity();
        let mut smr = M::identity();
        while l < r {
            if l & 1 != 0 {
                sml = M::binary_operation(&sml, &self.data[l]);
                l += 1;
            }
            if r & 1 != 0 {
                r -= 1;
                smr = M::binary_operation(&self.data[r], &smr);
            }
            l >>= 1;
            r >>= 1;
        }
        M::binary_operation(&sml, &smr)
    }

    /// 列全体の総積を返す。
    pub fn all_prod(&self) -> M::S {
        self.data[1].clone()
    }

    /// 節点 `k` の値を 2 つの子から再計算する。
    fn update(&mut self, k: usize) {
        self.data[k] = M::binary_operation(&self.data[2 * k], &self.data[2 * k + 1]);
    }

    /// [`RangeBounds`] を半開区間 `[l, r)` に正規化する。
    fn to_half_open<R: RangeBounds<usize>>(&self, range: &R) -> (usize, usize) {
        let l = match range.start_bound() {
            Bound::Included(&l) => l,
            Bound::Excluded(&l) => l + 1,
            Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            Bound::Included(&r) => r + 1,
            Bound::Excluded(&r) => r,
            Bound::Unbounded => self.n,
        };
        assert!(l <= r && r <= self.n, "区間が範囲外です: {l}..{r}");
        (l, r)
    }
}

impl<M: Monoid> From<Vec<M::S>> for Segtree<M> {
    fn from(v: Vec<M::S>) -> Self {
        let n = v.len();
        let size = n.next_power_of_two();
        let log = size.trailing_zeros();
        let mut data = vec![M::identity(); 2 * size];
        data[size..size + n].clone_from_slice(&v);
        let mut seg = Self { n, size, log, data };
        for k in (1..size).rev() {
            seg.update(k);
        }
        seg
    }
}

impl<M: Monoid> From<&[M::S]> for Segtree<M> {
    fn from(v: &[M::S]) -> Self {
        v.to_vec().into()
    }
}

#[cfg(test)]
mod tests {
    use super::{Additive, Segtree};

    fn naive_sum(v: &[i64], l: usize, r: usize) -> i64 {
        v[l..r].iter().sum()
    }

    #[test]
    fn prod_matches_naive() {
        let v: Vec<i64> = (0..17).map(|i| i * i - 3 * i).collect();
        let seg: Segtree<Additive<i64>> = v.clone().into();
        for l in 0..=v.len() {
            for r in l..=v.len() {
                assert_eq!(seg.prod(l..r), naive_sum(&v, l, r), "prod({l}..{r})");
            }
        }
    }

    #[test]
    fn set_then_prod() {
        let mut seg: Segtree<Additive<i64>> = Segtree::new(8);
        for i in 0..8 {
            seg.set(i, i as i64);
        }
        assert_eq!(seg.all_prod(), 28);
        assert_eq!(seg.prod(2..5), 9);
        seg.set(3, 100);
        assert_eq!(seg.get(3), 100);
        assert_eq!(seg.prod(2..5), 106);
        assert_eq!(seg.all_prod(), 125);
    }

    #[test]
    fn range_bounds_variants() {
        let seg: Segtree<Additive<i64>> = vec![1, 2, 4, 8].into();
        assert_eq!(seg.prod(..), 15);
        assert_eq!(seg.prod(1..), 14);
        assert_eq!(seg.prod(..2), 3);
        assert_eq!(seg.prod(1..=2), 6);
        assert_eq!(seg.prod(2..2), 0);
    }

    #[test]
    fn empty_segtree() {
        let seg: Segtree<Additive<i64>> = Segtree::new(0);
        assert!(seg.is_empty());
        assert_eq!(seg.all_prod(), 0);
        assert_eq!(seg.prod(..), 0);
    }

    #[test]
    #[should_panic(expected = "区間が範囲外です")]
    fn prod_out_of_range_panics() {
        let seg: Segtree<Additive<i64>> = vec![1, 2, 3].into();
        let _ = seg.prod(1..4);
    }
}
