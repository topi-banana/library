//! キー範囲の区間作用・区間集約ができる遅延セグメント木 [`LazySegmentTree`]。
//!
//! `LazySegmentTree` を使わない提出では `lib.rs` から `mod lazy_segment_tree;` と
//! `pub use lazy_segment_tree::{Action, LazySegmentTree, Monoid};` の 2 行を消すだけで
//! 済むように、別ファイルへ分けています。
//!
//! `LazySegmentTree` を使う提出で 1 ファイルにまとめるときは、`lib.rs` の
//! `mod lazy_segment_tree;` を次の形で置き換えると、先頭の `use super::*;` が
//! そのまま通ります。
//!
//! ```text
//! mod lazy_segment_tree {
//!     // ここに lazy_segment_tree.rs の全文
//! }
//! ```
//!
//! # Examples
//!
//! 区間加算・区間和は [`Monoid`] と [`Action`] を実装するだけで使えます。
//! 集約値が複数要素の和になるため、[`Action::mapping`] で足し込む個数を
//! 知れるように `S` は (和, 要素数) の組にします。
//!
//! ```
//! use avl_tree::{Action, LazySegmentTree, Map, Monoid};
//!
//! /// 区間加算・区間和
//! enum AddSum {}
//!
//! impl Monoid for AddSum {
//!     /// (和, 要素数)
//!     type S = (i64, i64);
//!
//!     fn op(a: &(i64, i64), b: &(i64, i64)) -> (i64, i64) {
//!         (a.0 + b.0, a.1 + b.1)
//!     }
//!
//!     fn identity() -> (i64, i64) {
//!         (0, 0)
//!     }
//! }
//!
//! impl Action for AddSum {
//!     type F = i64;
//!
//!     fn mapping(f: &i64, s: &(i64, i64)) -> (i64, i64) {
//!         (s.0 + f * s.1, s.1)
//!     }
//!
//!     fn composition(f: &i64, g: &i64) -> i64 {
//!         f + g
//!     }
//!
//!     fn id() -> i64 {
//!         0
//!     }
//! }
//!
//! let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
//! seg.put(0, (1, 1));
//! seg.put(1, (2, 1));
//! seg.put(2, (3, 1));
//!
//! // キーが [0, 2) の要素に 10 を足す
//! seg.apply(0..2, 10);
//! assert_eq!(seg.prod(0..2), (23, 2));
//! assert_eq!(seg.get(&2), Some(&(3, 1)));
//! ```
//!
//! index アクセスも使いたい場合は [`Indexed`] を外側に重ねます。
//!
//! ```
//! use avl_tree::{Action, Indexed, LazySegmentTree, Map, Monoid};
//!
//! enum AddSum {}
//!
//! impl Monoid for AddSum {
//!     type S = (i64, i64);
//!
//!     fn op(a: &(i64, i64), b: &(i64, i64)) -> (i64, i64) {
//!         (a.0 + b.0, a.1 + b.1)
//!     }
//!
//!     fn identity() -> (i64, i64) {
//!         (0, 0)
//!     }
//! }
//!
//! impl Action for AddSum {
//!     type F = i64;
//!
//!     fn mapping(f: &i64, s: &(i64, i64)) -> (i64, i64) {
//!         (s.0 + f * s.1, s.1)
//!     }
//!
//!     fn composition(f: &i64, g: &i64) -> i64 {
//!         f + g
//!     }
//!
//!     fn id() -> i64 {
//!         0
//!     }
//! }
//!
//! let mut seg: Map<Indexed<LazySegmentTree<usize, AddSum>>> = Map::new();
//! seg.put(0, (1, 1));
//! seg.put(1, (2, 1));
//!
//! seg.apply(0..2, 10);
//! assert_eq!(seg.prod(0..2), (23, 2));
//! assert_eq!(*seg.get_by_index(1).unwrap().key(), 1);
//! assert_eq!(seg.index_of(seg.slot_by_index(1).unwrap()), 1);
//! ```

use super::*;

use std::ops::Range;

/// 区間の集約に使うモノイド。
pub trait Monoid {
    /// 値と集約値の型。
    type S: Clone;

    /// 2 つの値を結合します。
    fn op(a: &Self::S, b: &Self::S) -> Self::S;

    /// 空区間の値 (単位元) を返します。
    fn identity() -> Self::S;
}

/// モノイドへの作用 (遅延タグ)。
pub trait Action: Monoid {
    /// 作用を表す型。
    type F: Clone;

    /// 値 `s` に作用 `f` を適用した結果を返します。
    fn mapping(f: &Self::F, s: &Self::S) -> Self::S;

    /// `g` を適用したあとに `f` を適用する作用を返します。
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;

    /// 何もしない作用 (恒等作用) を返します。
    fn id() -> Self::F;
}

/// キーの区間に対する作用と集約を載せる要素
///
/// [`Map`] に載せると、キー範囲に対する [`Map::prod`] / [`Map::apply`] が使えます。
/// キーは index に限らず、任意の順序キーで構いません。
/// in-order の index アクセスも使いたい場合は [`Indexed`] を外側に重ねた
/// `Map<Indexed<LazySegmentTree<K, A>>>` を使います。
///
/// 各ノードは自分の値、部分木の集約値、部分木に溜まった作用 (遅延タグ)、
/// 部分木のキー範囲 (`min` / `max`) を持ちます。作用は [`Element::push`] で
/// 子へ流れます。
///
/// # Examples
///
/// ```
/// # use avl_tree::{Action, LazySegmentTree, Map, Monoid};
/// # enum AddSum {}
/// # impl Monoid for AddSum {
/// #     type S = (i64, i64);
/// #     fn op(a: &(i64, i64), b: &(i64, i64)) -> (i64, i64) { (a.0 + b.0, a.1 + b.1) }
/// #     fn identity() -> (i64, i64) { (0, 0) }
/// # }
/// # impl Action for AddSum {
/// #     type F = i64;
/// #     fn mapping(f: &i64, s: &(i64, i64)) -> (i64, i64) { (s.0 + f * s.1, s.1) }
/// #     fn composition(f: &i64, g: &i64) -> i64 { f + g }
/// #     fn id() -> i64 { 0 }
/// # }
/// let mut seg: Map<LazySegmentTree<i32, AddSum>> = Map::new();
/// seg.put(10, (1, 1));
/// seg.put(20, (2, 1));
/// seg.put(30, (3, 1));
/// seg.apply(10..30, 5);
///
/// assert_eq!(seg.prod(10..30), (13, 2));
/// assert_eq!(seg.prod(20..31), (10, 2));
/// assert_eq!(seg.all_prod(), (16, 3));
/// ```
pub struct LazySegmentTree<K, A: Action> {
    key: K,
    value: A::S,
    aggregate: A::S,
    min: K,
    max: K,
    lazy: A::F,
}

impl<K: Clone, A: Action> LazySegmentTree<K, A> {
    /// キーと値から要素を作ります
    ///
    /// 集約値は値 1 個分、部分木のキー範囲は `key..=key` で初期化されます。
    pub fn new(key: K, value: A::S) -> Self {
        LazySegmentTree {
            aggregate: value.clone(),
            min: key.clone(),
            max: key.clone(),
            key,
            value,
            lazy: A::id(),
        }
    }

    /// キーへの参照を返します
    pub fn key(&self) -> &K {
        &self.key
    }

    /// タグ未適用の値への参照を返します
    ///
    /// 木に載せている場合、キーまでの経路の [`Element::push`] が済んでいれば
    /// 現在値になります。[`Map::get`] などで経路を流してから読むと確実です。
    pub fn value(&self) -> &A::S {
        &self.value
    }

    /// 部分木の集約値への参照を返します
    ///
    /// 溜まっている作用は反映済みです。
    pub fn aggregate(&self) -> &A::S {
        &self.aggregate
    }

    /// 部分木の最小キーへの参照を返します
    pub fn min_key(&self) -> &K {
        &self.min
    }

    /// 部分木の最大キーへの参照を返します
    pub fn max_key(&self) -> &K {
        &self.max
    }

    /// 値を取り出します
    pub fn into_value(self) -> A::S {
        self.value
    }

    /// 部分木全体に作用を適用します
    fn apply_subtree(&mut self, f: &A::F) {
        self.aggregate = A::mapping(f, &self.aggregate);
        self.lazy = A::composition(f, &self.lazy);
    }

    /// 自分の値だけに作用を適用します (集約値は `update` で再計算します)
    fn apply_own_value(&mut self, f: &A::F) {
        self.value = A::mapping(f, &self.value);
    }
}

impl<K: Ord + Clone, A: Action> Element for LazySegmentTree<K, A> {
    type Key = K;

    fn key(&self) -> &K {
        &self.key
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        let left_aggregate = left.map_or_else(A::identity, |l| l.aggregate.clone());
        let right_aggregate = right.map_or_else(A::identity, |r| r.aggregate.clone());
        let aggregate = A::op(&self.value, &A::op(&left_aggregate, &right_aggregate));
        // 溜まっている作用はまだ自分の値と子に適用されていないので、
        // 集約値にはここで反映しておく
        self.aggregate = A::mapping(&self.lazy, &aggregate);
        self.min = left.map_or_else(|| self.key.clone(), |l| l.min.clone());
        self.max = right.map_or_else(|| self.key.clone(), |r| r.max.clone());
    }

    fn push(&mut self, left: Option<&mut Self>, right: Option<&mut Self>) {
        self.value = A::mapping(&self.lazy, &self.value);
        for child in [left, right].into_iter().flatten() {
            child.apply_subtree(&self.lazy);
        }
        self.lazy = A::id();
    }
}

/// [`LazyNode::Act`] の集約値の型
type Aggregate<E> = <<E as LazyNode>::Act as Monoid>::S;

/// [`LazyNode::Act`] の作用の型
type Tag<E> = <<E as LazyNode>::Act as Action>::F;

/// [`Map`] のキー範囲操作 (`prod` / `apply`) が使える要素 (内部用)
trait LazyNode: Element {
    /// 作用と集約に使うモノイド
    type Act: Action;

    /// 部分木の集約値 (作用適用済み) への参照を返します
    fn aggregate(&self) -> &Aggregate<Self>;

    /// タグ未適用の値への参照を返します
    fn value(&self) -> &Aggregate<Self>;

    /// 部分木の最小キーへの参照を返します
    fn min_key(&self) -> &Self::Key;

    /// 部分木の最大キーへの参照を返します
    fn max_key(&self) -> &Self::Key;

    /// 部分木全体に作用を適用します
    fn apply_subtree(&mut self, f: &Tag<Self>);

    /// 自分の値だけに作用を適用します
    fn apply_own_value(&mut self, f: &Tag<Self>);

    /// 値を取り出します
    fn into_value(self) -> Aggregate<Self>;
}

impl<K: Ord + Clone, A: Action> LazyNode for LazySegmentTree<K, A> {
    type Act = A;

    fn aggregate(&self) -> &A::S {
        &self.aggregate
    }

    fn value(&self) -> &A::S {
        &self.value
    }

    fn min_key(&self) -> &K {
        &self.min
    }

    fn max_key(&self) -> &K {
        &self.max
    }

    fn apply_subtree(&mut self, f: &A::F) {
        LazySegmentTree::apply_subtree(self, f);
    }

    fn apply_own_value(&mut self, f: &A::F) {
        LazySegmentTree::apply_own_value(self, f);
    }

    fn into_value(self) -> A::S {
        LazySegmentTree::into_value(self)
    }
}

impl<K: Ord + Clone, A: Action> LazyNode for Indexed<LazySegmentTree<K, A>> {
    type Act = A;

    fn aggregate(&self) -> &A::S {
        self.inner().aggregate()
    }

    fn value(&self) -> &A::S {
        self.inner().value()
    }

    fn min_key(&self) -> &K {
        self.inner().min_key()
    }

    fn max_key(&self) -> &K {
        self.inner().max_key()
    }

    fn apply_subtree(&mut self, f: &A::F) {
        self.inner_mut().apply_subtree(f);
    }

    fn apply_own_value(&mut self, f: &A::F) {
        self.inner_mut().apply_own_value(f);
    }

    fn into_value(self) -> A::S {
        self.into_inner().into_value()
    }
}

/// `idx` を根とする部分木のうち、キーが `[lower, upper)` に入る要素の集約値を返します
fn prod_node<E: LazyNode>(
    map: &mut Map<E>,
    idx: usize,
    lower: &E::Key,
    upper: &E::Key,
) -> Aggregate<E> {
    if idx == usize::MAX {
        return <E::Act as Monoid>::identity();
    }

    let node_min = map.nodes[idx].element.min_key();
    let node_max = map.nodes[idx].element.max_key();
    if node_max < lower || upper <= node_min {
        return <E::Act as Monoid>::identity();
    }
    if lower <= node_min && node_max < upper {
        return map.nodes[idx].element.aggregate().clone();
    }

    let in_range = {
        let key = map.nodes[idx].element.key();
        lower <= key && key < upper
    };
    let left = map.nodes[idx].left;
    let right = map.nodes[idx].right;
    map.push_node(idx);

    let mut result = if left == usize::MAX {
        <E::Act as Monoid>::identity()
    } else {
        prod_node(map, left, lower, upper)
    };
    if in_range {
        result = <E::Act as Monoid>::op(&result, map.nodes[idx].element.value());
    }
    if right != usize::MAX {
        result = <E::Act as Monoid>::op(&result, &prod_node(map, right, lower, upper));
    }
    result
}

/// `idx` を根とする部分木のうち、キーが `[lower, upper)` に入る要素に `f` を適用します
///
/// 集約値が変化したならば `true` を返します。
fn apply_node<E: LazyNode>(
    map: &mut Map<E>,
    idx: usize,
    lower: &E::Key,
    upper: &E::Key,
    f: &Tag<E>,
) -> bool {
    if idx == usize::MAX {
        return false;
    }

    let node_min = map.nodes[idx].element.min_key();
    let node_max = map.nodes[idx].element.max_key();
    if node_max < lower || upper <= node_min {
        return false;
    }
    if lower <= node_min && node_max < upper {
        map.nodes[idx].element.apply_subtree(f);
        return true;
    }

    let in_range = {
        let key = map.nodes[idx].element.key();
        lower <= key && key < upper
    };
    let left = map.nodes[idx].left;
    let right = map.nodes[idx].right;
    map.push_node(idx);

    let mut changed = false;
    if left != usize::MAX {
        changed |= apply_node(map, left, lower, upper, f);
    }
    if in_range {
        map.nodes[idx].element.apply_own_value(f);
        changed = true;
    }
    if right != usize::MAX {
        changed |= apply_node(map, right, lower, upper, f);
    }
    if changed {
        map.update_node(idx);
    }
    changed
}

/// キー `key` を持つ要素の、作用適用済みの値への参照を返します
fn get_node<'a, E: LazyNode, Q: ?Sized + Ord>(
    map: &'a mut Map<E>,
    key: &Q,
) -> Option<&'a Aggregate<E>>
where
    E::Key: Borrow<Q>,
{
    map.push_path_to_key(key);
    match map.search(key) {
        Ok(slot) => Some(map.nodes[slot.index].element.value()),
        Err(_) => None,
    }
}

/// キー `key` を持つ要素を削除し、作用適用済みの値を返します
fn remove_node<E: LazyNode, Q: ?Sized + Ord>(map: &mut Map<E>, key: &Q) -> Option<Aggregate<E>>
where
    E::Key: Borrow<Q>,
{
    match map.search(key) {
        Ok(slot) => {
            // SAFETY: slot は直前の search が返した有効な Slot である。
            let element = unsafe { map.slot_remove(slot) };
            Some(element.into_value())
        }
        Err(_) => None,
    }
}

/// キーが `range` に入る要素の集約値を返します
fn prod_range<E: LazyNode>(map: &mut Map<E>, range: Range<E::Key>) -> Aggregate<E> {
    if range.start >= range.end {
        return <E::Act as Monoid>::identity();
    }
    let root = map.root;
    prod_node(map, root, &range.start, &range.end)
}

/// キーが `range` に入る要素に作用 `f` を適用します
fn apply_range<E: LazyNode>(map: &mut Map<E>, range: Range<E::Key>, f: Tag<E>) {
    if range.start >= range.end {
        return;
    }
    let root = map.root;
    apply_node(map, root, &range.start, &range.end, &f);
}

/// すべての要素の集約値を返します
fn all_prod_node<E: LazyNode>(map: &Map<E>) -> Aggregate<E> {
    if map.root == usize::MAX {
        <E::Act as Monoid>::identity()
    } else {
        map.nodes[map.root].element.aggregate().clone()
    }
}

impl<K: Ord + Clone, A: Action> Map<LazySegmentTree<K, A>> {
    /// キーと値を追加し、スロットを返します
    ///
    /// 同じキーがすでにある場合は値を置き換えます。
    /// 新しく入る値に、それまでその位置に溜まっていた作用は適用されません
    /// (先に作用を適用してから [`Map::put`] したものとみなします)。
    pub fn put(&mut self, key: K, value: A::S) -> Slot {
        self.insert(LazySegmentTree::new(key, value))
    }

    /// キーに対応する値への参照を返します
    ///
    /// キーまでの経路の遅延タグを流してから返すため `&mut self` を取ります。
    pub fn get(&mut self, key: &K) -> Option<&A::S> {
        get_node(self, key)
    }

    /// キーに対応する要素を削除し、値を返します
    ///
    /// 返る値には、それまでに適用された作用が反映されています。
    pub fn remove(&mut self, key: &K) -> Option<A::S> {
        remove_node(self, key)
    }

    /// キーが `range` に入る要素の集約値を返します
    ///
    /// 空区間なら [`Monoid::identity`] を返します。
    pub fn prod(&mut self, range: Range<K>) -> A::S {
        prod_range(self, range)
    }

    /// キーが `range` に入る要素に作用 `f` を適用します
    pub fn apply(&mut self, range: Range<K>, f: A::F) {
        apply_range(self, range, f);
    }

    /// すべての要素の集約値を返します
    pub fn all_prod(&self) -> A::S {
        all_prod_node(self)
    }
}

impl<K: Ord + Clone, A: Action> Map<Indexed<LazySegmentTree<K, A>>> {
    /// キーと値を追加し、スロットを返します
    ///
    /// 同じキーがすでにある場合は値を置き換えます。
    /// [`Indexed`] を外側に重ねているため、[`Map::slot_by_index`] /
    /// [`Map::get_by_index`] / [`Map::index_of`] も使えます。
    pub fn put(&mut self, key: K, value: A::S) -> Slot {
        self.insert(Indexed::new(LazySegmentTree::new(key, value)))
    }

    /// キーに対応する値への参照を返します
    ///
    /// キーまでの経路の遅延タグを流してから返すため `&mut self` を取ります。
    pub fn get(&mut self, key: &K) -> Option<&A::S> {
        get_node(self, key)
    }

    /// キーに対応する要素を削除し、値を返します
    ///
    /// 返る値には、それまでに適用された作用が反映されています。
    pub fn remove(&mut self, key: &K) -> Option<A::S> {
        remove_node(self, key)
    }

    /// キーが `range` に入る要素の集約値を返します
    ///
    /// 空区間なら [`Monoid::identity`] を返します。
    pub fn prod(&mut self, range: Range<K>) -> A::S {
        prod_range(self, range)
    }

    /// キーが `range` に入る要素に作用 `f` を適用します
    pub fn apply(&mut self, range: Range<K>, f: A::F) {
        apply_range(self, range, f);
    }

    /// すべての要素の集約値を返します
    pub fn all_prod(&self) -> A::S {
        all_prod_node(self)
    }
}
