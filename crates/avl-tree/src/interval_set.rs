//! 重なる・接する区間を 1 本にまとめて持つ区間集合 [`IntervalSet`]。
//!
//! `IntervalSet` を使わない提出では `lib.rs` から `mod interval_set;` と
//! `pub use interval_set::{Interval, IntervalSet, Merge};` の 2 行を消すだけで
//! 済むように、別ファイルへ分けています。
//!
//! `IntervalSet` を使う提出で 1 ファイルにまとめるときは、`lib.rs` の
//! `mod interval_set;` を次の形で置き換えると、先頭の `use super::*;` が
//! そのまま通ります。
//!
//! ```text
//! mod interval_set {
//!     // ここに interval_set.rs の全文
//! }
//! ```
//!
//! # Examples
//!
//! 重なる・接する区間は 1 本にまとめられ、値は [`Merge`] で合成されます。
//! 和で合成する区間集合は次のように書きます。
//!
//! ```
//! use avl_tree::{IntervalSet, Map, Merge};
//!
//! /// 重なる区間の値を足し合わせる
//! enum Sum {}
//!
//! impl Merge for Sum {
//!     type S = i64;
//!
//!     fn merge(a: &i64, b: &i64) -> i64 {
//!         a + b
//!     }
//!
//!     fn identity() -> i64 {
//!         0
//!     }
//! }
//!
//! let mut set: Map<IntervalSet<i64, Sum>> = Map::new();
//! set.insert_range(0..10, 3);
//! set.insert_range(5..15, 5); // [0, 10) と重なるので [0, 15) 値 8 にまとまる
//! set.insert_range(20..30, 7);
//!
//! assert_eq!(set.len(), 2);
//! assert_eq!(set.get_point(&7), Some(&8));
//! assert_eq!(set.get_point(&18), None);
//!
//! // 削除は被覆部分の切り取り。はみ出した断片は元の値を引き継ぐ
//! set.remove_range(10..25);
//! let got: Vec<(i64, i64, i64)> = set
//!     .iter_intervals()
//!     .map(|(interval, &value)| (*interval.start(), *interval.end(), value))
//!     .collect();
//! assert_eq!(got, vec![(0, 10, 8), (25, 30, 7)]);
//! ```

use super::*;

use std::ops::Range;

/// 重なる区間の値を合成するモノイド。
///
/// `Map<IntervalSet<T, M>>` に区間を追加したとき、重なる・接する区間は
/// 1 本にまとめられ、値が [`merge`](Merge::merge) で合成されます。
/// 値を足し合わせたいときは `merge` を `a + b`、`identity` を `0` にします。
///
/// [`LazySegmentTree`] の [`Monoid`] とは別のトレイトで、
/// `interval_set.rs` だけを切り出しても使えます。
///
/// # Examples
///
/// ```
/// use avl_tree::Merge;
///
/// /// 和で合成する
/// enum Sum {}
///
/// impl Merge for Sum {
///     type S = i64;
///
///     fn merge(a: &i64, b: &i64) -> i64 {
///         a + b
///     }
///
///     fn identity() -> i64 {
///         0
///     }
/// }
///
/// assert_eq!(Sum::merge(&3, &5), 8);
/// assert_eq!(Sum::identity(), 0);
/// ```
pub trait Merge {
    /// 値の型。
    type S: Clone;

    /// 2 つの値を合成します。
    fn merge(a: &Self::S, b: &Self::S) -> Self::S;

    /// 値が無い点の値 (単位元) を返します。
    ///
    /// `Map::get_point_or_identity` が、どの区間にも覆われていない点に対して返します。
    fn identity() -> Self::S;
}

/// 半開区間 `[start, end)`。
///
/// [`IntervalSet`] のキーです。順序は `start` → `end` の辞書式で、
/// 区間は `start` の昇順に並びます。
///
/// # Examples
///
/// ```
/// use avl_tree::Interval;
///
/// let interval = Interval::new(1, 4);
/// assert_eq!(*interval.start(), 1);
/// assert_eq!(*interval.end(), 4);
/// assert_eq!(interval.into_range(), 1..4);
///
/// assert!(Interval::new(1, 4) < Interval::new(3, 4));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval<T> {
    start: T,
    end: T,
}

impl<T> Interval<T> {
    /// 区間 `[start, end)` を作ります。
    pub fn new(start: T, end: T) -> Self {
        Interval { start, end }
    }

    /// 始点 `start` への参照を返します。
    pub fn start(&self) -> &T {
        &self.start
    }

    /// 終点 `end` への参照を返します。
    pub fn end(&self) -> &T {
        &self.end
    }

    /// 標準ライブラリの [`Range`] に変換します。
    pub fn into_range(self) -> Range<T> {
        self.start..self.end
    }
}

impl<T> From<Range<T>> for Interval<T> {
    fn from(range: Range<T>) -> Self {
        Interval { start: range.start, end: range.end }
    }
}

/// 重なる・接する区間を 1 本にまとめて持つ区間集合の要素。
///
/// [`Map`] に載せて使います。`Map<IntervalSet<T, M>>` には区間を扱う
/// [`insert_range`](Map::insert_range) / [`remove_range`](Map::remove_range) /
/// [`get_point`](Map::get_point) / [`covers`](Map::covers) /
/// [`covering`](Map::covering) などが生えます。
///
/// 区間の追加では、重なる・接する区間を吸収して 1 本にまとめ、
/// 値を [`Merge::merge`] で合成します。区間の削除では、被覆部分を切り取り、
/// はみ出した断片を残します。断片の値は元の値を引き継ぎます
/// (モノイドでは値を分割できないため)。
///
/// # Examples
///
/// ```
/// use avl_tree::{IntervalSet, Map, Merge};
/// # enum Sum {}
/// # impl Merge for Sum {
/// #     type S = i64;
/// #     fn merge(a: &i64, b: &i64) -> i64 { a + b }
/// #     fn identity() -> i64 { 0 }
/// # }
/// let mut set: Map<IntervalSet<i64, Sum>> = Map::new();
/// set.insert_range(0..5, 1);
/// set.insert_range(5..8, 2); // 端点で接するので [0, 8) 値 3 にまとまる
///
/// assert_eq!(set.len(), 1);
/// let (interval, &value) = set.covering(&3).unwrap();
/// assert_eq!((*interval.start(), *interval.end(), value), (0, 8, 3));
/// ```
pub struct IntervalSet<T, M: Merge> {
    interval: Interval<T>,
    value: M::S,
}

impl<T, M: Merge> IntervalSet<T, M> {
    /// 区間と値から要素を作ります。
    pub fn new(range: Range<T>, value: M::S) -> Self {
        IntervalSet { interval: Interval::from(range), value }
    }

    /// 区間への参照を返します。
    pub fn interval(&self) -> &Interval<T> {
        &self.interval
    }

    /// 始点 `start` への参照を返します。
    pub fn start(&self) -> &T {
        self.interval.start()
    }

    /// 終点 `end` への参照を返します。
    pub fn end(&self) -> &T {
        self.interval.end()
    }

    /// 値への参照を返します。
    pub fn value(&self) -> &M::S {
        &self.value
    }

    /// 値を取り出します。
    pub fn into_value(self) -> M::S {
        self.value
    }
}

impl<T: Clone, M: Merge> Clone for IntervalSet<T, M> {
    fn clone(&self) -> Self {
        IntervalSet { interval: self.interval.clone(), value: self.value.clone() }
    }
}

impl<T: fmt::Debug, M: Merge> fmt::Debug for IntervalSet<T, M>
where
    M::S: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IntervalSet")
            .field("interval", &self.interval)
            .field("value", &self.value)
            .finish()
    }
}

impl<T: Ord, M: Merge> Element for IntervalSet<T, M> {
    type Key = Interval<T>;

    fn key(&self) -> &Interval<T> {
        &self.interval
    }

    fn can_absorb(&self, other: &Self) -> bool {
        // 端点で接する (`self.end == other.start`) 場合もまとめる。
        self.interval.start() <= other.interval.end()
            && other.interval.start() <= self.interval.end()
    }

    fn absorb(&mut self, other: Self) {
        let IntervalSet { interval: Interval { start: s, end: e }, value } = other;
        if s < self.interval.start {
            self.interval.start = s;
        }
        if e > self.interval.end {
            self.interval.end = e;
        }
        self.value = M::merge(&self.value, &value);
    }
}

impl<T: Ord, M: Merge> Map<IntervalSet<T, M>> {
    /// 点 `point` を含む区間と値への参照を返します。
    ///
    /// 計算量は `O(log n)` です。
    pub fn covering(&self, point: &T) -> Option<(&Interval<T>, &M::S)> {
        let idx = self.covering_node(point)?;
        let element = &self.nodes[idx].element;
        Some((&element.interval, &element.value))
    }

    /// 点 `point` を含む区間の値への参照を返します。
    ///
    /// 計算量は `O(log n)` です。
    pub fn get_point(&self, point: &T) -> Option<&M::S> {
        let idx = self.covering_node(point)?;
        Some(&self.nodes[idx].element.value)
    }

    /// 点 `point` を含む区間の値、無ければ [`Merge::identity`] を返します。
    ///
    /// 計算量は `O(log n)` です。
    pub fn get_point_or_identity(&self, point: &T) -> M::S {
        match self.get_point(point) {
            Some(value) => value.clone(),
            None => M::identity(),
        }
    }

    /// 点 `point` がいずれかの区間に含まれるかを返します。
    ///
    /// 計算量は `O(log n)` です。
    pub fn covers(&self, point: &T) -> bool {
        self.covering_node(point).is_some()
    }

    /// 区間 `range` の全体が 1 本の区間に含まれるかを返します。
    ///
    /// 空区間 (`start >= end`) は `true` を返します。計算量は `O(log n)` です。
    pub fn contains_range(&self, range: &Range<T>) -> bool {
        if range.start >= range.end {
            return true;
        }
        match self.covering_node(&range.start) {
            Some(idx) => self.nodes[idx].element.interval.end() >= &range.end,
            None => false,
        }
    }

    /// 区間 `range` がいずれかの区間と少しでも重なるかを返します。
    ///
    /// 端点で接するだけの区間は重なったとはみなしません。
    /// 空区間 (`start >= end`) は `false` を返します。計算量は `O(log n)` です。
    pub fn overlaps(&self, range: &Range<T>) -> bool {
        if range.start >= range.end {
            return false;
        }
        if self.covering_node(&range.start).is_some() {
            return true;
        }
        let idx = self.first_start_ge(&range.start);
        idx != usize::MAX && self.nodes[idx].element.interval.start() < &range.end
    }

    /// 区間と値への不変イテレータを返します。
    ///
    /// 区間は `start` の昇順に並び、後ろからも進められます。
    pub fn iter_intervals(&self) -> IterIntervals<'_, T, M> {
        IterIntervals { inner: self.iter_elements() }
    }

    /// 点 `point` を含む区間のノードを返します。
    fn covering_node(&self, point: &T) -> Option<usize> {
        let idx = self.last_start_le(point);
        if idx != usize::MAX && self.nodes[idx].element.interval.end() > point {
            Some(idx)
        } else {
            None
        }
    }

    /// `point` 以下で最大の `start` を持つノードを返します (無ければ `usize::MAX`)。
    fn last_start_le(&self, point: &T) -> usize {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.interval.start() <= point {
                best = current;
                current = node.right;
            } else {
                current = node.left;
            }
        }
        best
    }

    /// `point` より小さい `start` を持つ最大のノードを返します (無ければ `usize::MAX`)。
    fn last_start_lt(&self, point: &T) -> usize {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.interval.start() < point {
                best = current;
                current = node.right;
            } else {
                current = node.left;
            }
        }
        best
    }

    /// `point` 以上で最小の `start` を持つノードを返します (無ければ `usize::MAX`)。
    fn first_start_ge(&self, point: &T) -> usize {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.interval.start() < point {
                current = node.right;
            } else {
                best = current;
                current = node.left;
            }
        }
        best
    }

    /// `[start, end)` と交差する区間のうち、in-order で最初のノードを返します。
    fn first_overlapping(&self, start: &T, end: &T) -> Option<usize> {
        // `start` より小さい始点を持つ最後の区間が `start` を覆っていればそれ。
        let pred = self.last_start_lt(start);
        if pred != usize::MAX && self.nodes[pred].element.interval.end() > start {
            return Some(pred);
        }
        // それ以外は `start` 以上の始点を持つ最初の区間が `end` より左から始まるか。
        let next = self.first_start_ge(start);
        if next != usize::MAX && self.nodes[next].element.interval.start() < end {
            Some(next)
        } else {
            None
        }
    }
}

impl<T: Ord + Clone, M: Merge> Map<IntervalSet<T, M>> {
    /// 区間 `[start, end)` に値 `value` を追加します。
    ///
    /// 追加する区間と重なる・接する区間はすべて吸収され、1 本にまとまります。
    /// 値は、追加する値を左端として、吸収されるたびに
    /// `merge(現在の値, 吸収される値)` の順で合成されます
    /// (可換な `merge` なら順序は気にしなくて構いません)。
    ///
    /// `start >= end` の空区間は何もしません。ならし計算量は `O(log n)` です。
    pub fn insert_range(&mut self, range: Range<T>, value: M::S) {
        if range.start >= range.end {
            return;
        }
        self.insert_element(IntervalSet::new(range, value));
    }

    /// 区間 `[start, end)` の被覆を削除します。
    ///
    /// 交差する区間は、`range` の外側の部分だけを残して切り取られます。
    /// はみ出した断片は元の区間の値を受け継ぎます。
    /// モノイドでは値を分割できないため、`range` が区間の内側に収まるときは
    /// 同じ値が左右の断片の両方に残ります (値の総和は削除で保存されません)。
    ///
    /// `start >= end` の空区間は何もしません。
    /// 交差した区間が `k` 本のとき、計算量は `O(k log n)` です。
    pub fn remove_range(&mut self, range: Range<T>) {
        let Range { start, end } = range;
        if start >= end {
            return;
        }

        let mut removed = Vec::new();
        while let Some(idx) = self.first_overlapping(&start, &end) {
            // SAFETY: idx は直前の探索が返したこのマップの有効なノードの添字。
            removed.push(unsafe { self.slot_remove(Slot { index: idx }) });
        }

        for element in removed {
            let IntervalSet { interval: Interval { start: s, end: e }, value } = element;
            // 交差しているので `s < end` かつ `start < e`。
            if s < start {
                let left = s..start.clone();
                if e > end {
                    // 区間が削除範囲をまたぐので 2 本に割れる。
                    // 値は合成せず、両方の断片に同じ値を持たせる。
                    self.insert_element(IntervalSet::new(left, value.clone()));
                    self.insert_element(IntervalSet::new(end.clone()..e, value));
                } else {
                    self.insert_element(IntervalSet::new(left, value));
                }
            } else if e > end {
                self.insert_element(IntervalSet::new(end.clone()..e, value));
            }
        }
    }
}

/// [`Map::iter_intervals`] が返す、区間と値のイテレータ。
pub struct IterIntervals<'a, T: Ord, M: Merge> {
    inner: IterElements<'a, IntervalSet<T, M>>,
}

impl<'a, T: Ord, M: Merge> Iterator for IterIntervals<'a, T, M> {
    type Item = (&'a Interval<T>, &'a M::S);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|element| (&element.interval, &element.value))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, T: Ord, M: Merge> DoubleEndedIterator for IterIntervals<'a, T, M> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|element| (&element.interval, &element.value))
    }
}

impl<T: Ord, M: Merge> ExactSizeIterator for IterIntervals<'_, T, M> {}

impl<T: Ord, M: Merge> std::iter::FusedIterator for IterIntervals<'_, T, M> {}
