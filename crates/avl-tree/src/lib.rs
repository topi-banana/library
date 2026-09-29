//! [`Element`] トレイトで集約値を差し替えられる AVL 木マップです。
//!
//! キーの比較と平衡化は木側が行い、部分木の集約値 (サイズ、和、マージ結果など) は
//! 要素側の [`Element::update`] が子から計算します。素のキーと値の組には
//! [`SimpleElement`] を使ってください。
//!
//! `Map<SimpleElement<K, V>>` や `Map<Indexed<SimpleElement<K, V>>>` では
//! [`MapElement`] 経由で `BTreeMap` / `HashMap` に近い API
//! ([`insert`](Map::insert) / [`entry`](Map::entry) / [`range`](Map::range) /
//! [`retain`](Map::retain) など) が使えます。
//!
//! 重なる・接する区間を 1 本にまとめて持つ区間集合には [`IntervalSet`] を
//! 使ってください。
//!
//! ```
//! use avl_tree::{Map, SimpleElement};
//!
//! let mut map: Map<SimpleElement<i32, &str>> = Map::new();
//! map.put(2, "two");
//! map.put(1, "one");
//!
//! assert_eq!(map.get(&1), Some(&"one"));
//! ```

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::TryReserveError;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Bound, Index, RangeBounds};

mod indexed;
mod interval_set;
mod lazy_segment_tree;

pub use indexed::Indexed;
pub use interval_set::{Interval, IntervalSet, Merge};
pub use lazy_segment_tree::{Action, LazySegmentTree, Monoid};

/// AVL 木のノードに載せる値。
///
/// 木は比較・検索に [`Element::key`] が返すキーだけを使います。
/// 部分木の集約値 (部分木サイズやモノイドの総和など) を持つ要素は
/// [`Element::update`] で子から再計算します。
pub trait Element: Sized {
    /// 順序付けに使うキーの型。
    type Key: Ord;

    /// キーへの参照を返します。
    fn key(&self) -> &Self::Key;

    /// 子が確定したあと、部分木の集約値を再計算します。
    ///
    /// 挿入・削除・回転の再平衡パスで、子ノードを渡して呼ばれます。
    /// 既定では何もしません。
    fn update(&mut self, _left: Option<&Self>, _right: Option<&Self>) {}

    /// 溜めた遅延タグを子へ流します。
    ///
    /// 遅延セグメント木のように作用を溜める要素のための拡張点です。
    /// 木は回転の前や、挿入・削除・検索の経路、区間操作の途中で呼びます。
    /// 既定では何もしません。
    fn push(&mut self, _left: Option<&mut Self>, _right: Option<&mut Self>) {}

    /// 追加時に `other` を吸収できるかを返します。
    ///
    /// 重複・隣接する区間のように、同じキーで複数の要素を持てない要素のための判定です。
    /// 既定では常に `false` を返します。
    fn can_absorb(&self, _other: &Self) -> bool {
        false
    }

    /// [`Element::can_absorb`] が `true` を返したときに `other` を吸収します。
    ///
    /// 吸収後もキーが他の要素と重複しないように実装する必要があります。
    /// 既定では何もせず `other` を捨てます。
    fn absorb(&mut self, _other: Self) {}
}

/// キーと値を分離できる要素。
///
/// [`Map`] に `BTreeMap` / `HashMap` 風のキー・値 API
/// ([`Map::insert`] / [`Map::get`] / [`Map::entry`] など) を生やすための
/// トレイトです。`SimpleElement<K, V>` と、その `Indexed` によるラップに
/// 実装しています。
///
/// 集約値が値に依存しないように実装してください。値の変更は
/// [`Map::get_mut`] などで直接行えるため、値に依存する集約値は
/// 更新されません。
pub trait MapElement: Element {
    /// 値の型。
    type Value;

    /// キーと値から要素を生成します。
    fn new(key: Self::Key, value: Self::Value) -> Self;

    /// 値への参照を返します。
    fn value(&self) -> &Self::Value;

    /// 値への可変参照を返します。
    fn value_mut(&mut self) -> &mut Self::Value;

    /// キーと値への可変参照を同時に返します。
    fn key_value_mut(&mut self) -> (&Self::Key, &mut Self::Value);

    /// キーと値を取り出します。
    fn into_kv(self) -> (Self::Key, Self::Value);
}

/// ノード構造体 (アリーナ形式の要素)
#[derive(Clone)]
struct Node<E> {
    element: E,
    parent: usize,
    left: usize,
    right: usize,
    height: usize,
}

/// スロット (存在する要素へのハンドル)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Slot {
    index: usize,
}

/// 空きスロット (挿入位置の情報を保持)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VacantSlot {
    None,
    Left(usize),
    Right(usize),
}

/// AVL木マップ。
///
/// 要素の型 `E` が [`Element`] を実装することで、順序統計や区間のマージなど、
/// 用途ごとの振る舞いを差し替えられます。`E` が [`MapElement`] を実装していれば
/// `BTreeMap` / `HashMap` に近いキー・値 API も使えます。
pub struct Map<E: Element> {
    nodes: Vec<Node<E>>,
    root: usize,
    len: usize,
}

/// `key` が上限 `bound` を満たすかを返します。
fn within_upper<Q, K>(key: &K, bound: Bound<&Q>) -> bool
where
    Q: ?Sized + Ord,
    K: ?Sized + Borrow<Q>,
{
    match bound {
        Bound::Included(end) => key.borrow() <= end,
        Bound::Excluded(end) => key.borrow() < end,
        Bound::Unbounded => true,
    }
}

/// 範囲の両端の組み合わせが不正なら、`BTreeMap` と同じ条件・メッセージで panic します。
fn check_range<Q>(start: Bound<&Q>, end: Bound<&Q>)
where
    Q: ?Sized + Ord,
{
    match (start, end) {
        (Bound::Excluded(s), Bound::Excluded(e)) if s == e => {
            panic!("range start and end are equal and excluded in BTreeMap")
        }
        (Bound::Included(s) | Bound::Excluded(s), Bound::Included(e) | Bound::Excluded(e))
            if s > e =>
        {
            panic!("range start is greater than range end in BTreeMap")
        }
        _ => {}
    }
}

impl<E: Element> Map<E> {
    /// 新しい空のマップを生成します
    pub fn new() -> Self {
        Map { nodes: Vec::new(), root: usize::MAX, len: 0 }
    }

    /// `capacity` 個以上の要素を格納できる容量を確保した空のマップを生成します
    pub fn with_capacity(capacity: usize) -> Self {
        Map { nodes: Vec::with_capacity(capacity), root: usize::MAX, len: 0 }
    }

    /// 要素数を返します
    pub fn len(&self) -> usize {
        self.len
    }

    /// 要素が空かどうかを返します
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 容量を返します
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// `additional` 個以上の要素を追加で格納できるように容量を確保します
    ///
    /// 容量がすでに足りている場合は何もしません。
    pub fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
    }

    /// `additional` 個以上の要素を追加で格納できるように最小限の容量を確保します
    pub fn reserve_exact(&mut self, additional: usize) {
        self.nodes.reserve_exact(additional);
    }

    /// `additional` 個以上の要素を追加で格納できるように容量を確保します
    ///
    /// 確保に失敗した場合は [`TryReserveError`] を返します。
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.nodes.try_reserve(additional)
    }

    /// `additional` 個以上の要素を追加で格納できるように最小限の容量を確保します
    ///
    /// 確保に失敗した場合は [`TryReserveError`] を返します。
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.nodes.try_reserve_exact(additional)
    }

    /// 余分な容量を解放します
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// 容量を `min_capacity` 以上に保ちつつ、余分な容量を解放します
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.nodes.shrink_to(min_capacity);
    }

    /// すべての要素を削除します
    ///
    /// 確保した容量は保持されます。
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = usize::MAX;
        self.len = 0;
    }

    /// キーで検索し、存在すれば Slot、なければ VacantSlot を返します
    pub fn search<Q: ?Sized + Ord>(&self, key: &Q) -> Result<Slot, VacantSlot>
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut vacant_slot = VacantSlot::None;

        while current != usize::MAX {
            let node = &self.nodes[current];
            match key.cmp(node.element.key().borrow()) {
                Ordering::Equal => return Ok(Slot { index: current }),
                Ordering::Less => {
                    vacant_slot = VacantSlot::Left(current);
                    current = node.left;
                }
                Ordering::Greater => {
                    vacant_slot = VacantSlot::Right(current);
                    current = node.right;
                }
            }
        }
        Err(vacant_slot)
    }

    /// キーを持つ要素が存在するかを返します
    pub fn contains<Q: ?Sized + Ord>(&self, key: &Q) -> bool
    where
        E::Key: Borrow<Q>,
    {
        self.search(key).is_ok()
    }

    /// `key` より小さい最大の要素を返します
    pub fn predecessor<Q: ?Sized + Ord>(&self, key: &Q) -> Option<Slot>
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.key().borrow() < key {
                best = current;
                current = node.right;
            } else {
                current = node.left;
            }
        }
        if best == usize::MAX { None } else { Some(Slot { index: best }) }
    }

    /// `key` より大きい最小の要素を返します
    pub fn successor<Q: ?Sized + Ord>(&self, key: &Q) -> Option<Slot>
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if key < node.element.key().borrow() {
                best = current;
                current = node.left;
            } else {
                current = node.right;
            }
        }
        if best == usize::MAX { None } else { Some(Slot { index: best }) }
    }

    /// in-order で最初の要素を返します
    pub fn first(&self) -> Option<Slot> {
        if self.root == usize::MAX {
            return None;
        }
        let mut current = self.root;
        while self.nodes[current].left != usize::MAX {
            current = self.nodes[current].left;
        }
        Some(Slot { index: current })
    }

    /// in-order で最後の要素を返します
    pub fn last(&self) -> Option<Slot> {
        if self.root == usize::MAX {
            return None;
        }
        let mut current = self.root;
        while self.nodes[current].right != usize::MAX {
            current = self.nodes[current].right;
        }
        Some(Slot { index: current })
    }

    /// スロットの in-order 後継を返します
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub fn next(&self, s: Slot) -> Option<Slot> {
        let index = self.next_node(s.index);
        if index == usize::MAX { None } else { Some(Slot { index }) }
    }

    /// スロットの in-order 先行を返します
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub fn prev(&self, s: Slot) -> Option<Slot> {
        let index = self.prev_node(s.index);
        if index == usize::MAX { None } else { Some(Slot { index }) }
    }

    /// Slot から要素の不変参照を取得します
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_ref(&self, s: Slot) -> &E {
        &self.nodes[s.index].element
    }

    /// Slot から要素の可変参照を取得します
    ///
    /// キーを書き換えると木の順序が壊れるため、値の編集にのみ使ってください。
    /// 集約値を編集した場合は [`Map::slot_refresh`] で根まで再計算する必要があります。
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_mut(&mut self, s: Slot) -> &mut E {
        &mut self.nodes[s.index].element
    }

    /// VacantSlot に要素を挿入し、Slot を返します
    ///
    /// [`Map::insert_element`] と違い、同一キーの置き換えや吸収は行いません。
    /// 挿入する要素は葉として正規化するため、先に [`Element::update`] を呼びます。
    pub fn slot_insert(&mut self, v: VacantSlot, mut element: E) -> Slot {
        // 挿入位置までの遅延タグを先に流し、新しく入る要素に
        // 過去に適用された作用が乗らないようにする。
        self.push_path_to_key(element.key());
        element.update(None, None);

        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            element,
            parent: usize::MAX,
            left: usize::MAX,
            right: usize::MAX,
            height: 1,
        });

        match v {
            VacantSlot::None => self.root = new_idx,
            VacantSlot::Left(parent) => {
                unsafe { self.nodes.get_unchecked_mut(new_idx) }.parent = parent;
                self.nodes[parent].left = new_idx;
                self.rebalance_from(parent);
            }
            VacantSlot::Right(parent) => {
                unsafe { self.nodes.get_unchecked_mut(new_idx) }.parent = parent;
                self.nodes[parent].right = new_idx;
                self.rebalance_from(parent);
            }
        }

        self.len += 1;
        Slot { index: new_idx }
    }

    /// 要素を追加し、配置されたスロットを返します
    ///
    /// 同じキーを持つ要素がすでにある場合、[`Element::can_absorb`] が `true` なら
    /// 吸収し、そうでなければそのスロットを上書きします。
    /// さらに、前後の隣接要素に対して [`Element::can_absorb`] が `true` を返す限り
    /// 繰り返し吸収します。
    pub fn insert_element(&mut self, element: E) -> Slot {
        let mut element = element;
        element.update(None, None);

        // 同一キーの既存要素を吸収するか、そのスロットを上書きする
        if let Ok(slot) = self.search(element.key()) {
            if element.can_absorb(unsafe { self.slot_ref(slot) }) {
                let other = unsafe { self.slot_remove(slot) };
                element.absorb(other);
            } else {
                // 置き換える位置までの遅延タグを先に流し、新しい値に
                // 過去の作用が乗らないようにする (子は作用を受け取る)。
                self.push_path_to(slot.index);
                self.nodes[slot.index].element = element;
                unsafe { self.slot_refresh(slot) };
                return slot;
            }
        }

        // 前後の隣接要素を吸収できる限り取り込む
        loop {
            if let Some(prev) = self.predecessor(element.key())
                && element.can_absorb(unsafe { self.slot_ref(prev) })
            {
                let other = unsafe { self.slot_remove(prev) };
                element.absorb(other);
                continue;
            }
            if let Some(next) = self.successor(element.key())
                && element.can_absorb(unsafe { self.slot_ref(next) })
            {
                let other = unsafe { self.slot_remove(next) };
                element.absorb(other);
                continue;
            }
            break;
        }

        // 吸収の前後でキーの一意性が保たれているため、ここは必ず空きスロットになる
        let vacant = self.search(element.key()).unwrap_err();
        self.slot_insert(vacant, element)
    }

    /// Slot の要素を削除し、要素を返します
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_remove(&mut self, s: Slot) -> E {
        let mut del_idx = s.index;

        // 削除対象までの遅延タグを先に流す。削除対象の子は作用を受け取り、
        // 返す要素の値は作用適用済みになる。
        self.push_path_to(del_idx);

        let has_left = self.nodes[del_idx].left != usize::MAX;
        let has_right = self.nodes[del_idx].right != usize::MAX;

        // 子が2つある場合、後継とデータを交換して削除対象を移動
        if has_left && has_right {
            let mut s_idx = self.nodes[del_idx].right;
            // 交換でキー集合が入れ替わるため、後継までの経路の遅延タグも
            // 先に流しておく。
            loop {
                self.push_node(s_idx);
                let left = self.nodes[s_idx].left;
                if left == usize::MAX {
                    break;
                }
                s_idx = left;
            }
            self.swap_contents(del_idx, s_idx);
            del_idx = s_idx;
        }

        // del_idx は高々1つの子しか持たない
        let child = if self.nodes[del_idx].left != usize::MAX {
            self.nodes[del_idx].left
        } else {
            self.nodes[del_idx].right
        };

        let parent = self.nodes[del_idx].parent;
        let mut parent_to_balance = parent;

        if parent == usize::MAX {
            self.root = child;
        } else if self.nodes[parent].left == del_idx {
            self.nodes[parent].left = child;
        } else {
            self.nodes[parent].right = child;
        }

        if child != usize::MAX {
            self.nodes[child].parent = parent;
        }

        // 削除する要素を取り出す。
        // このスロットは直後の上書き (ptr::write) か set_len で消えるため、
        // 取り出した値が二重に Drop されることはない。
        // SAFETY: del_idx は直前まで有効なノードを指している。
        let element = unsafe { std::ptr::read(&self.nodes[del_idx].element) };

        // スワップ削除 (O(1)削除)
        let last_idx = self.nodes.len() - 1;

        if del_idx != last_idx {
            // SAFETY: last_idx も有効なノードを指しており、read の直後に
            // del_idx へ write して元のスロットは set_len で切り詰める。
            let last_node = unsafe { std::ptr::read(&self.nodes[last_idx]) };
            unsafe { std::ptr::write(&mut self.nodes[del_idx], last_node) };

            // 移動したノードのポインタを修正
            let moved_parent = self.nodes[del_idx].parent;
            let moved_left = self.nodes[del_idx].left;
            let moved_right = self.nodes[del_idx].right;

            if moved_parent != usize::MAX {
                if self.nodes[moved_parent].left == last_idx {
                    self.nodes[moved_parent].left = del_idx;
                } else if self.nodes[moved_parent].right == last_idx {
                    self.nodes[moved_parent].right = del_idx;
                }
            } else {
                self.root = del_idx;
            }
            if moved_left != usize::MAX {
                self.nodes[moved_left].parent = del_idx;
            }
            if moved_right != usize::MAX {
                self.nodes[moved_right].parent = del_idx;
            }

            // 再平衡化の起点が最後のノードだった場合は、そのノードは
            // スワップ削除で del_idx に移動しているためそちらを使う
            if parent_to_balance == last_idx {
                parent_to_balance = del_idx;
            }
        }
        // SAFETY: 最後の要素を読み出し済みなので、長さを1つ縮めるだけでよい。
        unsafe { self.nodes.set_len(last_idx) };

        self.rebalance_from(parent_to_balance);

        self.len -= 1;
        element
    }

    /// Slot の要素とその祖先の集約値を再計算します
    ///
    /// [`Map::slot_mut`] で値や集約値に関わるフィールドを編集したあとに呼びます。
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_refresh(&mut self, s: Slot) {
        let mut current = s.index;
        while current != usize::MAX {
            self.update_node(current);
            current = self.nodes[current].parent;
        }
    }

    /// `idx` を削除したあとに次に処理すべきノードのインデックスを返します
    ///
    /// `next_idx` は削除前の in-order 後継、`last_idx` は削除前のノード配列の
    /// 末尾です。スワップ削除で要素が移動する分を補正します。
    fn next_after_removal(
        &self,
        idx: usize,
        next_idx: usize,
        last_idx: usize,
        has_two_children: bool,
    ) -> usize {
        if has_two_children {
            if last_idx == idx { next_idx } else { idx }
        } else if next_idx == last_idx {
            idx
        } else {
            next_idx
        }
    }

    /// `key` 以上で最小のノードのインデックスを返します (無ければ `usize::MAX`)
    fn first_ge<Q: ?Sized + Ord>(&self, key: &Q) -> usize
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.key().borrow() < key {
                current = node.right;
            } else {
                best = current;
                current = node.left;
            }
        }
        best
    }

    /// `key` より大きい最小のノードのインデックスを返します (無ければ `usize::MAX`)
    fn first_gt<Q: ?Sized + Ord>(&self, key: &Q) -> usize
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.key().borrow() <= key {
                current = node.right;
            } else {
                best = current;
                current = node.left;
            }
        }
        best
    }

    /// `key` 以下で最大のノードのインデックスを返します (無ければ `usize::MAX`)
    fn last_le<Q: ?Sized + Ord>(&self, key: &Q) -> usize
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.key().borrow() <= key {
                best = current;
                current = node.right;
            } else {
                current = node.left;
            }
        }
        best
    }

    /// `key` より小さい最大のノードのインデックスを返します (無ければ `usize::MAX`)
    fn last_lt<Q: ?Sized + Ord>(&self, key: &Q) -> usize
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        let mut best = usize::MAX;
        while current != usize::MAX {
            let node = &self.nodes[current];
            if node.element.key().borrow() < key {
                best = current;
                current = node.right;
            } else {
                current = node.left;
            }
        }
        best
    }

    /// 2つのノードの要素を交換します (木構造・高さは変更しません)
    fn swap_contents(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let (head, tail) = self.nodes.split_at_mut(hi);
        std::mem::swap(&mut head[lo].element, &mut tail[0].element);
    }

    /// 指定ノードの高さを取得します
    fn node_height(&self, idx: usize) -> usize {
        if idx == usize::MAX { 0 } else { self.nodes[idx].height }
    }

    /// 指定ノードの高さと集約値を子から再計算します
    fn update_node(&mut self, idx: usize) {
        let left = self.nodes[idx].left;
        let right = self.nodes[idx].right;
        let height = self.node_height(left).max(self.node_height(right)) + 1;

        match (left == usize::MAX, right == usize::MAX) {
            (true, true) => self.nodes[idx].element.update(None, None),
            (false, true) => {
                let [node, l] = self.nodes.get_disjoint_mut([idx, left]).unwrap();
                node.element.update(Some(&l.element), None);
            }
            (true, false) => {
                let [node, r] = self.nodes.get_disjoint_mut([idx, right]).unwrap();
                node.element.update(None, Some(&r.element));
            }
            (false, false) => {
                let [node, l, r] = self.nodes.get_disjoint_mut([idx, left, right]).unwrap();
                node.element.update(Some(&l.element), Some(&r.element));
            }
        }

        self.nodes[idx].height = height;
    }

    /// 指定ノードの遅延タグを子へ流します
    ///
    /// `idx` はこのマップに対して有効なノードのインデックスである必要があります。
    fn push_node(&mut self, idx: usize) {
        let left = self.nodes[idx].left;
        let right = self.nodes[idx].right;

        match (left == usize::MAX, right == usize::MAX) {
            (true, true) => self.nodes[idx].element.push(None, None),
            (false, true) => {
                let [node, l] = self.nodes.get_disjoint_mut([idx, left]).unwrap();
                node.element.push(Some(&mut l.element), None);
            }
            (true, false) => {
                let [node, r] = self.nodes.get_disjoint_mut([idx, right]).unwrap();
                node.element.push(None, Some(&mut r.element));
            }
            (false, false) => {
                let [node, l, r] = self.nodes.get_disjoint_mut([idx, left, right]).unwrap();
                node.element.push(Some(&mut l.element), Some(&mut r.element));
            }
        }
    }

    /// 根から `target` までの経路の遅延タグを上から順に子へ流します
    ///
    /// `target` はこのマップに対して有効なノードのインデックスである必要があります。
    fn push_path_to(&mut self, target: usize) {
        let mut current = self.root;
        while current != usize::MAX {
            if current == target {
                self.push_node(current);
                return;
            }
            let cmp = self.nodes[target].element.key().cmp(self.nodes[current].element.key());
            self.push_node(current);
            current = match cmp {
                Ordering::Less => self.nodes[current].left,
                Ordering::Greater => self.nodes[current].right,
                // キーは一意なので、target に向かう途中で Equal にはならない
                Ordering::Equal => return,
            };
        }
    }

    /// 根から `key` の位置までの経路の遅延タグを上から順に子へ流します
    fn push_path_to_key<Q: ?Sized + Ord>(&mut self, key: &Q)
    where
        E::Key: Borrow<Q>,
    {
        let mut current = self.root;
        while current != usize::MAX {
            let cmp = key.cmp(self.nodes[current].element.key().borrow());
            self.push_node(current);
            current = match cmp {
                Ordering::Equal => return,
                Ordering::Less => self.nodes[current].left,
                Ordering::Greater => self.nodes[current].right,
            };
        }
    }

    /// 指定ノードのバランス係数を取得します
    fn get_balance(&self, idx: usize) -> i32 {
        if idx == usize::MAX {
            return 0;
        }
        let left = self.nodes[idx].left;
        let right = self.nodes[idx].right;
        self.node_height(left) as i32 - self.node_height(right) as i32
    }

    /// `current` から根に向かって高さ・集約値を更新し、AVL の再平衡化を行います
    fn rebalance_from(&mut self, mut current: usize) {
        while current != usize::MAX {
            self.update_node(current);
            let balance = self.get_balance(current);
            if balance > 1 {
                let left = self.nodes[current].left;
                if left != usize::MAX && self.get_balance(left) < 0 {
                    self.rotate_left(left);
                }
                self.rotate_right(current);
            } else if balance < -1 {
                let right = self.nodes[current].right;
                if right != usize::MAX && self.get_balance(right) > 0 {
                    self.rotate_right(right);
                }
                self.rotate_left(current);
            }
            current = self.nodes[current].parent;
        }
    }

    /// 左回転を行います
    fn rotate_left(&mut self, x: usize) {
        let y = self.nodes[x].right;

        // 回転で部分木のキー集合が入れ替わるため、先に遅延タグを流しておく。
        // 回転後に流すとタグの適用範囲がずれる。
        self.push_node(x);
        self.push_node(y);

        let t2 = self.nodes[y].left;

        self.nodes[y].left = x;
        self.nodes[x].right = t2;

        if t2 != usize::MAX {
            self.nodes[t2].parent = x;
        }

        let x_parent = self.nodes[x].parent;
        self.nodes[y].parent = x_parent;
        self.nodes[x].parent = y;

        if x_parent == usize::MAX {
            self.root = y;
        } else if self.nodes[x_parent].left == x {
            self.nodes[x_parent].left = y;
        } else {
            self.nodes[x_parent].right = y;
        }

        self.update_node(x);
        self.update_node(y);
    }

    /// 右回転を行います
    fn rotate_right(&mut self, y: usize) {
        let x = self.nodes[y].left;

        // 回転で部分木のキー集合が入れ替わるため、先に遅延タグを流しておく。
        // 回転後に流すとタグの適用範囲がずれる。
        self.push_node(y);
        self.push_node(x);

        let t2 = self.nodes[x].right;

        self.nodes[x].right = y;
        self.nodes[y].left = t2;

        if t2 != usize::MAX {
            self.nodes[t2].parent = y;
        }

        let y_parent = self.nodes[y].parent;
        self.nodes[x].parent = y_parent;
        self.nodes[y].parent = x;

        if y_parent == usize::MAX {
            self.root = x;
        } else if self.nodes[y_parent].left == y {
            self.nodes[y_parent].left = x;
        } else {
            self.nodes[y_parent].right = x;
        }

        self.update_node(y);
        self.update_node(x);
    }

    /// in-order で次のノードのインデックスを返します
    fn next_node(&self, idx: usize) -> usize {
        let mut current = idx;
        if self.nodes[current].right != usize::MAX {
            current = self.nodes[current].right;
            while self.nodes[current].left != usize::MAX {
                current = self.nodes[current].left;
            }
            return current;
        }
        let mut parent = self.nodes[current].parent;
        while parent != usize::MAX && current == self.nodes[parent].right {
            current = parent;
            parent = self.nodes[parent].parent;
        }
        parent
    }

    /// in-order で前のノードのインデックスを返します
    fn prev_node(&self, idx: usize) -> usize {
        let mut current = idx;
        if self.nodes[current].left != usize::MAX {
            current = self.nodes[current].left;
            while self.nodes[current].right != usize::MAX {
                current = self.nodes[current].right;
            }
            return current;
        }
        let mut parent = self.nodes[current].parent;
        while parent != usize::MAX && current == self.nodes[parent].left {
            current = parent;
            parent = self.nodes[parent].parent;
        }
        parent
    }

    /// 要素への不変イテレータを返します
    ///
    /// キーと値の組が欲しい場合は [`Map::iter`] を使ってください (要素が
    /// [`MapElement`] を実装している場合のみ)。
    pub fn iter_elements(&self) -> IterElements<'_, E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        IterElements { map: self, front, back, len: self.len }
    }

    /// 要素への可変イテレータを返します
    ///
    /// 集約値を持つ要素の値を書き換えた場合は、[`Map::slot_refresh`] で
    /// 根まで再計算する必要があります。
    pub fn iter_mut_elements(&mut self) -> IterMutElements<'_, E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let len = self.len;
        IterMutElements { map: self as *mut Map<E>, front, back, len, _marker: PhantomData }
    }

    /// 所有権を消費して要素を取り出すイテレータを返します
    pub fn into_elements(self) -> IntoElements<E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let len = self.len;
        IntoElements { map: ManuallyDrop::new(self), front, back, len }
    }

    /// 条件を満たす要素を削除しながらイテレートします
    pub fn extract_elements_if<F>(&mut self, f: F) -> ExtractElementsIf<'_, E, F>
    where
        F: FnMut(&E) -> bool,
    {
        let current = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        ExtractElementsIf { map: self, current, f }
    }
}

impl<E: Element> Default for Map<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Element + Clone> Clone for Map<E> {
    fn clone(&self) -> Self {
        Map { nodes: self.nodes.clone(), root: self.root, len: self.len }
    }
}

impl<E: MapElement> Map<E> {
    /// キーと値を追加し、以前の値を返します
    ///
    /// 同じキーがすでにある場合は値を置き換えて、以前の値を返します。
    /// キーが無ければ `None` を返します。
    ///
    /// ```
    /// use avl_tree::{Map, SimpleElement};
    ///
    /// let mut map: Map<SimpleElement<i32, i32>> = Map::new();
    /// assert_eq!(map.insert(1, 10), None);
    /// assert_eq!(map.insert(1, 20), Some(10));
    /// assert_eq!(map.get(&1), Some(&20));
    /// ```
    pub fn insert(&mut self, key: E::Key, value: E::Value) -> Option<E::Value> {
        match self.entry(key) {
            Entry::Occupied(mut entry) => Some(entry.insert(value)),
            Entry::Vacant(entry) => {
                entry.insert(value);
                None
            }
        }
    }

    /// キーと値を追加し、配置されたスロットを返します
    ///
    /// 同じキーがすでにある場合は値を置き換えて、同じスロットを返します。
    pub fn put(&mut self, key: E::Key, value: E::Value) -> Slot {
        self.insert_element(E::new(key, value))
    }

    /// キーに対応する entry を返します
    ///
    /// ```
    /// use avl_tree::{Map, SimpleElement};
    ///
    /// let mut map: Map<SimpleElement<i32, i32>> = Map::new();
    /// map.entry(1).or_insert(10);
    /// map.entry(1).and_modify(|value| *value += 1).or_insert(0);
    /// assert_eq!(map.get(&1), Some(&11));
    /// ```
    pub fn entry(&mut self, key: E::Key) -> Entry<'_, E> {
        match self.search(&key) {
            Ok(slot) => Entry::Occupied(OccupiedEntry { map: self, slot }),
            Err(vacant) => Entry::Vacant(VacantEntry { map: self, key, vacant }),
        }
    }

    /// キーに対応する値への参照を返します
    pub fn get<Q>(&self, key: &Q) -> Option<&E::Value>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_ref(slot) }.value()),
            Err(_) => None,
        }
    }

    /// キーと値への参照を返します
    pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&E::Key, &E::Value)>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => {
                let element = unsafe { self.slot_ref(slot) };
                Some((element.key(), element.value()))
            }
            Err(_) => None,
        }
    }

    /// キーを持つ要素が存在するかを返します
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        self.search(key).is_ok()
    }

    /// キーに対応する値への可変参照を返します
    ///
    /// 集約値が値に依存する要素では、編集後に [`Map::slot_refresh`] が必要です。
    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut E::Value>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_mut(slot) }.value_mut()),
            Err(_) => None,
        }
    }

    /// 最大 `N` 個の値への可変参照を一度に返します
    ///
    /// 見つからないキーの位置には `None` が入ります。
    ///
    /// # Panics
    ///
    /// 同じキーが複数渡された場合に panic します。
    pub fn get_disjoint_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<&mut E::Value>; N]
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        for i in 0..N {
            for j in 0..i {
                assert!(keys[i] != keys[j], "duplicate keys found");
            }
        }

        let mut slots = [usize::MAX; N];
        for i in 0..N {
            if let Ok(slot) = self.search(keys[i]) {
                slots[i] = slot.index;
            }
        }

        let ptr = self.nodes.as_mut_ptr();
        std::array::from_fn(|i| {
            if slots[i] == usize::MAX {
                None
            } else {
                // SAFETY: slots の有効な要素は search が返した相異なるスロットで、
                // それぞれ別のノードを指す。
                Some(unsafe { (*ptr.add(slots[i])).element.value_mut() })
            }
        })
    }

    /// in-order で最初の要素へのキーと値の参照を返します
    pub fn first_key_value(&self) -> Option<(&E::Key, &E::Value)> {
        let slot = self.first()?;
        let element = unsafe { self.slot_ref(slot) };
        Some((element.key(), element.value()))
    }

    /// in-order で最初の要素への entry を返します
    pub fn first_entry(&mut self) -> Option<OccupiedEntry<'_, E>> {
        let slot = self.first()?;
        Some(OccupiedEntry { map: self, slot })
    }

    /// in-order で最初の要素を削除して返します
    pub fn pop_first(&mut self) -> Option<(E::Key, E::Value)> {
        let slot = self.first()?;
        Some(unsafe { self.slot_remove(slot) }.into_kv())
    }

    /// in-order で最後の要素へのキーと値の参照を返します
    pub fn last_key_value(&self) -> Option<(&E::Key, &E::Value)> {
        let slot = self.last()?;
        let element = unsafe { self.slot_ref(slot) };
        Some((element.key(), element.value()))
    }

    /// in-order で最後の要素への entry を返します
    pub fn last_entry(&mut self) -> Option<OccupiedEntry<'_, E>> {
        let slot = self.last()?;
        Some(OccupiedEntry { map: self, slot })
    }

    /// in-order で最後の要素を削除して返します
    pub fn pop_last(&mut self) -> Option<(E::Key, E::Value)> {
        let slot = self.last()?;
        Some(unsafe { self.slot_remove(slot) }.into_kv())
    }

    /// キーに対応する要素を削除し、値を返します
    pub fn remove<Q>(&mut self, key: &Q) -> Option<E::Value>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_remove(slot) }.into_kv().1),
            Err(_) => None,
        }
    }

    /// キーに対応する要素を削除し、キーと値を返します
    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(E::Key, E::Value)>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_remove(slot) }.into_kv()),
            Err(_) => None,
        }
    }

    /// 条件を満たす要素だけを残します
    ///
    /// `f` にはキーと値への可変参照が渡され、`false` を返した要素は削除されます。
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&E::Key, &mut E::Value) -> bool,
    {
        self.extract_if(.., |key, value| !f(key, value)).for_each(drop);
    }

    /// `other` の要素をすべて `self` に移動し、`other` を空にします
    ///
    /// 同じキーが `self` にある場合は `other` の値で上書きされます。
    pub fn append(&mut self, other: &mut Self) {
        for (key, value) in other.drain() {
            self.insert(key, value);
        }
    }

    /// `key` 以上の要素を `self` から取り出し、新しいマップとして返します
    pub fn split_off<Q>(&mut self, key: &Q) -> Self
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
    {
        let mut other = Map::new();
        loop {
            let index = self.first_ge(key);
            if index == usize::MAX {
                break;
            }
            let (key, value) = unsafe { self.slot_remove(Slot { index }) }.into_kv();
            other.insert(key, value);
        }
        other
    }

    /// キーが `range` に入る要素の不変イテレータを返します
    ///
    /// 返るのはキーと値の組です。要素そのものを取り出したい場合は
    /// [`Map::iter_elements`] を使ってください。
    pub fn range<Q, R>(&self, range: R) -> Range<'_, E>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        let (front, back) = self.range_edges::<Q, R>(&range);
        Range { map: self, front, back }
    }

    /// キーが `range` に入る要素の可変イテレータを返します
    pub fn range_mut<Q, R>(&mut self, range: R) -> RangeMut<'_, E>
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        let (front, back) = self.range_edges::<Q, R>(&range);
        RangeMut { map: self as *mut Map<E>, front, back, _marker: PhantomData }
    }

    /// 全要素のキーと値の組の不変イテレータを返します
    pub fn iter(&self) -> Iter<'_, E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let len = self.len;
        Iter { inner: Range { map: self, front, back }, len }
    }

    /// 全要素のキーと値の組の可変イテレータを返します
    ///
    /// キーは不変で、値だけを書き換えられます。
    pub fn iter_mut(&mut self) -> IterMut<'_, E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let len = self.len;
        let inner = RangeMut { map: self as *mut Map<E>, front, back, _marker: PhantomData };
        IterMut { inner, len }
    }

    /// 全要素のキーの不変イテレータを返します
    pub fn keys(&self) -> Keys<'_, E> {
        Keys { inner: self.iter() }
    }

    /// 全要素の値の不変イテレータを返します
    pub fn values(&self) -> Values<'_, E> {
        Values { inner: self.iter() }
    }

    /// 全要素の値の可変イテレータを返します
    pub fn values_mut(&mut self) -> ValuesMut<'_, E> {
        ValuesMut { inner: self.iter_mut() }
    }

    /// 所有権を消費してキーを取り出すイテレータを返します
    pub fn into_keys(self) -> IntoKeys<E> {
        IntoKeys { inner: self.into_iter() }
    }

    /// 所有権を消費して値を取り出すイテレータを返します
    pub fn into_values(self) -> IntoValues<E> {
        IntoValues { inner: self.into_iter() }
    }

    /// キーが `range` に入り、かつ条件を満たす要素を削除しながら返します
    ///
    /// `pred` にはキーと値への可変参照が渡され、`true` を返した要素が
    /// その場で削除されてイテレータから返ります。
    /// 全要素を対象にする場合は `range` に `..` を渡します。
    pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> ExtractIf<'_, E, R, F>
    where
        R: RangeBounds<E::Key>,
        F: FnMut(&E::Key, &mut E::Value) -> bool,
    {
        check_range(range.start_bound(), range.end_bound());
        let front = match range.start_bound() {
            Bound::Included(key) => self.first_ge(key),
            Bound::Excluded(key) => self.first_gt(key),
            Bound::Unbounded => match self.first() {
                Some(slot) => slot.index,
                None => usize::MAX,
            },
        };
        let done = front == usize::MAX
            || !within_upper(self.nodes[front].element.key(), range.end_bound());
        ExtractIf { map: self, range, pred, front, done }
    }

    /// すべての要素を削除しながら返すイテレータを返します
    ///
    /// イテレータを drop すると、まだ取り出していない要素も含めて
    /// マップは空になります。
    pub fn drain(&mut self) -> Drain<'_, E> {
        Drain { map: self }
    }

    /// `range` の両端に対応するノードのインデックスを返します
    ///
    /// 範囲が空なら `(usize::MAX, usize::MAX)` を返します。
    fn range_edges<Q, R>(&self, range: &R) -> (usize, usize)
    where
        Q: ?Sized + Ord,
        E::Key: Borrow<Q>,
        R: RangeBounds<Q>,
    {
        // std の BTreeMap と同じ条件・メッセージで panic する。
        check_range(range.start_bound(), range.end_bound());
        let front = match range.start_bound() {
            Bound::Included(key) => self.first_ge(key),
            Bound::Excluded(key) => self.first_gt(key),
            Bound::Unbounded => match self.first() {
                Some(slot) => slot.index,
                None => usize::MAX,
            },
        };
        if front == usize::MAX || !within_upper(self.nodes[front].element.key(), range.end_bound())
        {
            return (usize::MAX, usize::MAX);
        }
        let back = match range.end_bound() {
            Bound::Included(key) => self.last_le(key),
            Bound::Excluded(key) => self.last_lt(key),
            Bound::Unbounded => match self.last() {
                Some(slot) => slot.index,
                None => usize::MAX,
            },
        };
        (front, back)
    }
}

impl<E: MapElement> fmt::Debug for Map<E>
where
    E::Key: fmt::Debug,
    E::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<E: MapElement> PartialEq for Map<E>
where
    E::Key: PartialEq,
    E::Value: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}

impl<E: MapElement> Eq for Map<E>
where
    E::Key: Eq,
    E::Value: Eq,
{
}

impl<E: MapElement> PartialOrd for Map<E>
where
    E::Key: PartialOrd,
    E::Value: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<E: MapElement> Ord for Map<E>
where
    E::Key: Ord,
    E::Value: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<E: MapElement> Hash for Map<E>
where
    E::Key: Hash,
    E::Value: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for (key, value) in self.iter() {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<E: MapElement, Q> Index<&Q> for Map<E>
where
    Q: ?Sized + Ord,
    E::Key: Borrow<Q>,
{
    type Output = E::Value;

    fn index(&self, key: &Q) -> &E::Value {
        self.get(key).expect("no entry found for key")
    }
}

impl<E: MapElement> FromIterator<(E::Key, E::Value)> for Map<E> {
    fn from_iter<T: IntoIterator<Item = (E::Key, E::Value)>>(iter: T) -> Self {
        let mut map = Map::new();
        map.extend(iter);
        map
    }
}

impl<E: MapElement> Extend<(E::Key, E::Value)> for Map<E> {
    fn extend<T: IntoIterator<Item = (E::Key, E::Value)>>(&mut self, iter: T) {
        for (key, value) in iter {
            self.insert(key, value);
        }
    }
}

impl<'a, K: Ord + Copy, V: Copy> Extend<(&'a K, &'a V)> for Map<SimpleElement<K, V>> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (&key, &value) in iter {
            self.insert(key, value);
        }
    }
}

impl<'a, K: Ord + Copy, V: Copy> Extend<(&'a K, &'a V)> for Map<Indexed<SimpleElement<K, V>>> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (&key, &value) in iter {
            self.insert(key, value);
        }
    }
}

impl<E: MapElement> IntoIterator for Map<E> {
    type Item = (E::Key, E::Value);
    type IntoIter = IntoIter<E>;

    fn into_iter(self) -> IntoIter<E> {
        let front = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let back = match self.last() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        let len = self.len;
        IntoIter { map: ManuallyDrop::new(self), front, back, len }
    }
}

impl<'a, E: MapElement> IntoIterator for &'a Map<E> {
    type Item = (&'a E::Key, &'a E::Value);
    type IntoIter = Iter<'a, E>;

    fn into_iter(self) -> Iter<'a, E> {
        self.iter()
    }
}

impl<'a, E: MapElement> IntoIterator for &'a mut Map<E> {
    type Item = (&'a E::Key, &'a mut E::Value);
    type IntoIter = IterMut<'a, E>;

    fn into_iter(self) -> IterMut<'a, E> {
        self.iter_mut()
    }
}

// ===== Element レベルのイテレータ =====

/// [`Map::iter_elements`] が返すイテレータ
pub struct IterElements<'a, E: Element> {
    map: &'a Map<E>,
    front: usize,
    back: usize,
    len: usize,
}

impl<'a, E: Element> Iterator for IterElements<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.front;
        let element = &self.map.nodes[index].element;
        self.front = if self.len == 1 {
            self.back = usize::MAX;
            usize::MAX
        } else {
            self.map.next_node(index)
        };
        self.len -= 1;
        Some(element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<E: Element> DoubleEndedIterator for IterElements<'_, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.back;
        let element = &self.map.nodes[index].element;
        self.back = if self.len == 1 {
            self.front = usize::MAX;
            usize::MAX
        } else {
            self.map.prev_node(index)
        };
        self.len -= 1;
        Some(element)
    }
}

impl<E: Element> ExactSizeIterator for IterElements<'_, E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: Element> std::iter::FusedIterator for IterElements<'_, E> {}

/// [`Map::iter_mut_elements`] が返すイテレータ
pub struct IterMutElements<'a, E: Element> {
    map: *mut Map<E>,
    front: usize,
    back: usize,
    len: usize,
    _marker: PhantomData<&'a mut Map<E>>,
}

impl<'a, E: Element> Iterator for IterMutElements<'a, E> {
    type Item = &'a mut E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.front;
        // SAFETY: 呼び出しごとに重複しないノードを返し、`self.map` は有効な
        // マップを指している。
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[index];
            let element = &mut *(&mut node.element as *mut E);
            self.front = if self.len == 1 {
                self.back = usize::MAX;
                usize::MAX
            } else {
                map.next_node(index)
            };
            self.len -= 1;
            Some(element)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, E: Element> DoubleEndedIterator for IterMutElements<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.back;
        // SAFETY: 呼び出しごとに重複しないノードを返し、`self.map` は有効な
        // マップを指している。
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[index];
            let element = &mut *(&mut node.element as *mut E);
            self.back = if self.len == 1 {
                self.front = usize::MAX;
                usize::MAX
            } else {
                map.prev_node(index)
            };
            self.len -= 1;
            Some(element)
        }
    }
}

impl<E: Element> ExactSizeIterator for IterMutElements<'_, E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: Element> std::iter::FusedIterator for IterMutElements<'_, E> {}

/// [`Map::into_elements`] が返すイテレータ
pub struct IntoElements<E: Element> {
    map: ManuallyDrop<Map<E>>,
    front: usize,
    back: usize,
    len: usize,
}

impl<E: Element> Iterator for IntoElements<E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.front;
        self.front = if self.len == 1 {
            self.back = usize::MAX;
            usize::MAX
        } else {
            self.map.next_node(index)
        };
        self.len -= 1;
        // SAFETY: このスロットはまだ読み出しておらず、以降も読み出さない。
        Some(unsafe { std::ptr::read(&self.map.nodes[index].element) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<E: Element> DoubleEndedIterator for IntoElements<E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.back;
        self.back = if self.len == 1 {
            self.front = usize::MAX;
            usize::MAX
        } else {
            self.map.prev_node(index)
        };
        self.len -= 1;
        // SAFETY: このスロットはまだ読み出しておらず、以降も読み出さない。
        Some(unsafe { std::ptr::read(&self.map.nodes[index].element) })
    }
}

impl<E: Element> ExactSizeIterator for IntoElements<E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: Element> std::iter::FusedIterator for IntoElements<E> {}

impl<E: Element> Drop for IntoElements<E> {
    fn drop(&mut self) {
        unsafe {
            // 残りの要素を in-order で手動 Drop する
            while self.len > 0 {
                let index = self.front;
                self.front = if self.len == 1 {
                    self.back = usize::MAX;
                    usize::MAX
                } else {
                    self.map.next_node(index)
                };
                self.len -= 1;
                std::ptr::drop_in_place(&mut self.map.nodes[index].element);
            }
            // 読み出し済みの要素は move 済みなので、Vec には要素を触らせず
            // 確保したメモリだけを解放させる。
            self.map.nodes.set_len(0);
            ManuallyDrop::drop(&mut self.map);
        }
    }
}

/// [`Map::extract_elements_if`] が返すイテレータ
pub struct ExtractElementsIf<'a, E: Element, F>
where
    F: FnMut(&E) -> bool,
{
    map: &'a mut Map<E>,
    current: usize,
    f: F,
}

impl<'a, E: Element, F> Iterator for ExtractElementsIf<'a, E, F>
where
    F: FnMut(&E) -> bool,
{
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current != usize::MAX {
            let idx = self.current;
            let next_idx = self.map.next_node(idx);
            let last_idx = self.map.nodes.len() - 1;
            // 子が2つのノードを削除すると、後継の要素が idx に移動する
            let has_two_children =
                self.map.nodes[idx].left != usize::MAX && self.map.nodes[idx].right != usize::MAX;

            let should_remove = (self.f)(&self.map.nodes[idx].element);

            if should_remove {
                // SAFETY: idx は extract_elements_if が保持している有効なノードのインデックス。
                let res = unsafe { self.map.slot_remove(Slot { index: idx }) };
                self.current =
                    self.map.next_after_removal(idx, next_idx, last_idx, has_two_children);
                return Some(res);
            }
            self.current = next_idx;
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.map.len))
    }
}

impl<E: Element, F> std::iter::FusedIterator for ExtractElementsIf<'_, E, F> where
    F: FnMut(&E) -> bool
{
}

// ===== キーと値のイテレータ =====

/// [`Map::range`] が返すイテレータ
pub struct Range<'a, E: MapElement> {
    map: &'a Map<E>,
    front: usize,
    back: usize,
}

impl<'a, E: MapElement> Iterator for Range<'a, E> {
    type Item = (&'a E::Key, &'a E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == usize::MAX {
            return None;
        }
        let index = self.front;
        self.front = if self.front == self.back {
            self.back = usize::MAX;
            usize::MAX
        } else {
            self.map.next_node(index)
        };
        let element = &self.map.nodes[index].element;
        Some((element.key(), element.value()))
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for Range<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == usize::MAX {
            return None;
        }
        let index = self.back;
        self.back = if self.back == self.front {
            self.front = usize::MAX;
            usize::MAX
        } else {
            self.map.prev_node(index)
        };
        let element = &self.map.nodes[index].element;
        Some((element.key(), element.value()))
    }
}

impl<E: MapElement> std::iter::FusedIterator for Range<'_, E> {}

/// [`Map::range_mut`] が返すイテレータ
pub struct RangeMut<'a, E: MapElement> {
    map: *mut Map<E>,
    front: usize,
    back: usize,
    _marker: PhantomData<&'a mut Map<E>>,
}

impl<'a, E: MapElement> Iterator for RangeMut<'a, E> {
    type Item = (&'a E::Key, &'a mut E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.front == usize::MAX {
            return None;
        }
        let index = self.front;
        // SAFETY: 呼び出しごとに重複しないノードを返し、`self.map` は有効な
        // マップを指している。
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[index];
            let element = &mut *(&mut node.element as *mut E);
            self.front = if self.front == self.back {
                self.back = usize::MAX;
                usize::MAX
            } else {
                map.next_node(index)
            };
            let (key, value) = element.key_value_mut();
            Some((key, value))
        }
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for RangeMut<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back == usize::MAX {
            return None;
        }
        let index = self.back;
        // SAFETY: 呼び出しごとに重複しないノードを返し、`self.map` は有効な
        // マップを指している。
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[index];
            let element = &mut *(&mut node.element as *mut E);
            self.back = if self.back == self.front {
                self.front = usize::MAX;
                usize::MAX
            } else {
                map.prev_node(index)
            };
            let (key, value) = element.key_value_mut();
            Some((key, value))
        }
    }
}

impl<E: MapElement> std::iter::FusedIterator for RangeMut<'_, E> {}

/// [`Map::iter`] が返すイテレータ
pub struct Iter<'a, E: MapElement> {
    inner: Range<'a, E>,
    len: usize,
}

impl<'a, E: MapElement> Iterator for Iter<'a, E> {
    type Item = (&'a E::Key, &'a E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for Iter<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.inner.next_back()
    }
}

impl<E: MapElement> ExactSizeIterator for Iter<'_, E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: MapElement> std::iter::FusedIterator for Iter<'_, E> {}

/// [`Map::iter_mut`] が返すイテレータ
pub struct IterMut<'a, E: MapElement> {
    inner: RangeMut<'a, E>,
    len: usize,
}

impl<'a, E: MapElement> Iterator for IterMut<'a, E> {
    type Item = (&'a E::Key, &'a mut E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for IterMut<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        self.inner.next_back()
    }
}

impl<E: MapElement> ExactSizeIterator for IterMut<'_, E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: MapElement> std::iter::FusedIterator for IterMut<'_, E> {}

/// [`Map::into_iter`](Map::into_keys) が返すイテレータ
pub struct IntoIter<E: MapElement> {
    map: ManuallyDrop<Map<E>>,
    front: usize,
    back: usize,
    len: usize,
}

impl<E: MapElement> Iterator for IntoIter<E> {
    type Item = (E::Key, E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.front;
        self.front = if self.len == 1 {
            self.back = usize::MAX;
            usize::MAX
        } else {
            self.map.next_node(index)
        };
        self.len -= 1;
        // SAFETY: このスロットはまだ読み出しておらず、以降も読み出さない。
        let element = unsafe { std::ptr::read(&self.map.nodes[index].element) };
        Some(element.into_kv())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<E: MapElement> DoubleEndedIterator for IntoIter<E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let index = self.back;
        self.back = if self.len == 1 {
            self.front = usize::MAX;
            usize::MAX
        } else {
            self.map.prev_node(index)
        };
        self.len -= 1;
        // SAFETY: このスロットはまだ読み出しておらず、以降も読み出さない。
        let element = unsafe { std::ptr::read(&self.map.nodes[index].element) };
        Some(element.into_kv())
    }
}

impl<E: MapElement> ExactSizeIterator for IntoIter<E> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<E: MapElement> std::iter::FusedIterator for IntoIter<E> {}

impl<E: MapElement> Drop for IntoIter<E> {
    fn drop(&mut self) {
        unsafe {
            // 残りの要素を in-order で手動 Drop する
            while self.len > 0 {
                let index = self.front;
                self.front = if self.len == 1 {
                    self.back = usize::MAX;
                    usize::MAX
                } else {
                    self.map.next_node(index)
                };
                self.len -= 1;
                std::ptr::drop_in_place(&mut self.map.nodes[index].element);
            }
            // 読み出し済みの要素は move 済みなので、Vec には要素を触らせず
            // 確保したメモリだけを解放させる。
            self.map.nodes.set_len(0);
            ManuallyDrop::drop(&mut self.map);
        }
    }
}

/// [`Map::keys`] が返すイテレータ
pub struct Keys<'a, E: MapElement> {
    inner: Iter<'a, E>,
}

impl<'a, E: MapElement> Iterator for Keys<'a, E> {
    type Item = &'a E::Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for Keys<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(key, _)| key)
    }
}

impl<E: MapElement> ExactSizeIterator for Keys<'_, E> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<E: MapElement> std::iter::FusedIterator for Keys<'_, E> {}

/// [`Map::values`] が返すイテレータ
pub struct Values<'a, E: MapElement> {
    inner: Iter<'a, E>,
}

impl<'a, E: MapElement> Iterator for Values<'a, E> {
    type Item = &'a E::Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for Values<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, value)| value)
    }
}

impl<E: MapElement> ExactSizeIterator for Values<'_, E> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<E: MapElement> std::iter::FusedIterator for Values<'_, E> {}

/// [`Map::values_mut`] が返すイテレータ
pub struct ValuesMut<'a, E: MapElement> {
    inner: IterMut<'a, E>,
}

impl<'a, E: MapElement> Iterator for ValuesMut<'a, E> {
    type Item = &'a mut E::Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<'a, E: MapElement> DoubleEndedIterator for ValuesMut<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, value)| value)
    }
}

impl<E: MapElement> ExactSizeIterator for ValuesMut<'_, E> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<E: MapElement> std::iter::FusedIterator for ValuesMut<'_, E> {}

/// [`Map::into_keys`] が返すイテレータ
pub struct IntoKeys<E: MapElement> {
    inner: IntoIter<E>,
}

impl<E: MapElement> Iterator for IntoKeys<E> {
    type Item = E::Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<E: MapElement> DoubleEndedIterator for IntoKeys<E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(key, _)| key)
    }
}

impl<E: MapElement> ExactSizeIterator for IntoKeys<E> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<E: MapElement> std::iter::FusedIterator for IntoKeys<E> {}

/// [`Map::into_values`] が返すイテレータ
pub struct IntoValues<E: MapElement> {
    inner: IntoIter<E>,
}

impl<E: MapElement> Iterator for IntoValues<E> {
    type Item = E::Value;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<E: MapElement> DoubleEndedIterator for IntoValues<E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|(_, value)| value)
    }
}

impl<E: MapElement> ExactSizeIterator for IntoValues<E> {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<E: MapElement> std::iter::FusedIterator for IntoValues<E> {}

/// [`Map::extract_if`] が返すイテレータ
pub struct ExtractIf<'a, E: MapElement, R, F>
where
    R: RangeBounds<E::Key>,
    F: FnMut(&E::Key, &mut E::Value) -> bool,
{
    map: &'a mut Map<E>,
    range: R,
    pred: F,
    front: usize,
    done: bool,
}

impl<E: MapElement, R, F> Iterator for ExtractIf<'_, E, R, F>
where
    R: RangeBounds<E::Key>,
    F: FnMut(&E::Key, &mut E::Value) -> bool,
{
    type Item = (E::Key, E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.done {
            let idx = self.front;
            if idx == usize::MAX
                || !within_upper(self.map.nodes[idx].element.key(), self.range.end_bound())
            {
                self.done = true;
                return None;
            }

            let should_remove = {
                let (key, value) = self.map.nodes[idx].element.key_value_mut();
                (self.pred)(key, value)
            };

            if should_remove {
                let next_idx = self.map.next_node(idx);
                let last_idx = self.map.nodes.len() - 1;
                let has_two_children = self.map.nodes[idx].left != usize::MAX
                    && self.map.nodes[idx].right != usize::MAX;
                // SAFETY: idx は直前までこのマップの有効なノードを指していた。
                let (key, value) = unsafe { self.map.slot_remove(Slot { index: idx }) }.into_kv();
                self.front = self.map.next_after_removal(idx, next_idx, last_idx, has_two_children);
                return Some((key, value));
            }
            self.front = self.map.next_node(idx);
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.map.len))
    }
}

impl<E: MapElement, R, F> std::iter::FusedIterator for ExtractIf<'_, E, R, F>
where
    R: RangeBounds<E::Key>,
    F: FnMut(&E::Key, &mut E::Value) -> bool,
{
}

/// [`Map::drain`] が返すイテレータ
pub struct Drain<'a, E: MapElement> {
    map: &'a mut Map<E>,
}

impl<'a, E: MapElement> Iterator for Drain<'a, E> {
    type Item = (E::Key, E::Value);

    fn next(&mut self) -> Option<Self::Item> {
        let slot = self.map.first()?;
        // SAFETY: first が返した有効な Slot である。
        Some(unsafe { self.map.slot_remove(slot) }.into_kv())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.map.len()))
    }
}

impl<E: MapElement> Drop for Drain<'_, E> {
    fn drop(&mut self) {
        self.map.clear();
    }
}

impl<E: MapElement> std::iter::FusedIterator for Drain<'_, E> {}

// ===== entry =====

/// [`Map::entry`] が返す entry
pub enum Entry<'a, E: MapElement> {
    /// キーが存在しない場合の entry
    Vacant(VacantEntry<'a, E>),
    /// キーが存在する場合の entry
    Occupied(OccupiedEntry<'a, E>),
}

impl<'a, E: MapElement> Entry<'a, E> {
    /// entry のキーへの参照を返します
    pub fn key(&self) -> &E::Key {
        match self {
            Entry::Vacant(entry) => entry.key(),
            Entry::Occupied(entry) => entry.key(),
        }
    }

    /// キーが無ければ `default` を挿入し、値への可変参照を返します
    pub fn or_insert(self, default: E::Value) -> &'a mut E::Value {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// キーが無ければ `default()` を挿入し、値への可変参照を返します
    pub fn or_insert_with<F>(self, default: F) -> &'a mut E::Value
    where
        F: FnOnce() -> E::Value,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// キーが無ければ `default(&key)` を挿入し、値への可変参照を返します
    pub fn or_insert_with_key<F>(self, default: F) -> &'a mut E::Value
    where
        F: FnOnce(&E::Key) -> E::Value,
    {
        match self {
            Entry::Vacant(entry) => {
                let value = default(entry.key());
                entry.insert(value)
            }
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// キーが無ければ [`Default`] を挿入し、値への可変参照を返します
    pub fn or_default(self) -> &'a mut E::Value
    where
        E::Value: Default,
    {
        self.or_insert_with(E::Value::default)
    }

    /// キーが存在する場合に、その値へ `f` を適用します
    pub fn and_modify<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut E::Value),
    {
        if let Entry::Occupied(entry) = &mut self {
            f(entry.get_mut());
        }
        self
    }

    /// 値を設定し、その [`OccupiedEntry`] を返します
    pub fn insert_entry(self, value: E::Value) -> OccupiedEntry<'a, E> {
        match self {
            Entry::Vacant(entry) => entry.insert_entry(value),
            Entry::Occupied(mut entry) => {
                entry.insert(value);
                entry
            }
        }
    }
}

impl<E: MapElement> fmt::Debug for Entry<'_, E>
where
    E::Key: fmt::Debug,
    E::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::Vacant(entry) => f.debug_tuple("Entry").field(entry).finish(),
            Entry::Occupied(entry) => f.debug_tuple("Entry").field(entry).finish(),
        }
    }
}

/// キーが存在しない場合の entry ([`Entry::Vacant`] の中身)
pub struct VacantEntry<'a, E: MapElement> {
    map: &'a mut Map<E>,
    key: E::Key,
    vacant: VacantSlot,
}

impl<'a, E: MapElement> VacantEntry<'a, E> {
    /// 挿入に使われるキーへの参照を返します
    pub fn key(&self) -> &E::Key {
        &self.key
    }

    /// 挿入に使われるキーを取り出します
    pub fn into_key(self) -> E::Key {
        self.key
    }

    /// 値を挿入し、値への可変参照を返します
    pub fn insert(self, value: E::Value) -> &'a mut E::Value {
        self.insert_entry(value).into_mut()
    }

    /// 値を挿入し、その [`OccupiedEntry`] を返します
    pub fn insert_entry(self, value: E::Value) -> OccupiedEntry<'a, E> {
        let VacantEntry { map, key, vacant } = self;
        let slot = map.slot_insert(vacant, E::new(key, value));
        OccupiedEntry { map, slot }
    }
}

impl<E: MapElement> fmt::Debug for VacantEntry<'_, E>
where
    E::Key: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(self.key()).finish()
    }
}

/// キーが存在する場合の entry ([`Entry::Occupied`] の中身)
pub struct OccupiedEntry<'a, E: MapElement> {
    map: &'a mut Map<E>,
    slot: Slot,
}

impl<'a, E: MapElement> OccupiedEntry<'a, E> {
    /// キーへの参照を返します
    pub fn key(&self) -> &E::Key {
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_ref(self.slot) }.key()
    }

    /// 値への参照を返します
    pub fn get(&self) -> &E::Value {
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_ref(self.slot) }.value()
    }

    /// 値への可変参照を返します
    pub fn get_mut(&mut self) -> &mut E::Value {
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_mut(self.slot) }.value_mut()
    }

    /// 値への可変参照を返します
    pub fn into_mut(self) -> &'a mut E::Value {
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_mut(self.slot) }.value_mut()
    }

    /// 値を置き換え、以前の値を返します
    pub fn insert(&mut self, value: E::Value) -> E::Value {
        let old = std::mem::replace(self.get_mut(), value);
        // 集約値が値に依存する要素でも整合するように祖先を再計算する
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_refresh(self.slot) };
        old
    }

    /// 要素を削除し、値を返します
    pub fn remove(self) -> E::Value {
        self.remove_entry().1
    }

    /// 要素を削除し、キーと値を返します
    pub fn remove_entry(self) -> (E::Key, E::Value) {
        // SAFETY: この Slot は entry 作成時に有効だった。
        unsafe { self.map.slot_remove(self.slot) }.into_kv()
    }
}

impl<E: MapElement> fmt::Debug for OccupiedEntry<'_, E>
where
    E::Key: fmt::Debug,
    E::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry").field("key", self.key()).field("value", self.get()).finish()
    }
}

/// キーと値をそのままノードに載せる要素
///
/// # Examples
///
/// ```
/// use avl_tree::{Map, SimpleElement};
///
/// let mut map: Map<SimpleElement<i32, &str>> = Map::new();
/// map.put(2, "two");
/// map.put(1, "one");
/// assert_eq!(map.get(&1), Some(&"one"));
/// assert_eq!(map.remove(&2), Some("two"));
/// assert_eq!(map.len(), 1);
/// ```
#[derive(Clone, Debug)]
pub struct SimpleElement<K, V> {
    key: K,
    value: V,
}

impl<K, V> SimpleElement<K, V> {
    /// キーと値の組を生成します
    pub fn new(key: K, value: V) -> Self {
        SimpleElement { key, value }
    }

    /// キーへの参照を返します
    pub fn key(&self) -> &K {
        &self.key
    }

    /// 値への参照を返します
    pub fn value(&self) -> &V {
        &self.value
    }

    /// 値への可変参照を返します
    pub fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }
}

impl<K: Ord, V> Element for SimpleElement<K, V> {
    type Key = K;

    fn key(&self) -> &K {
        &self.key
    }
}

impl<K: Ord, V> MapElement for SimpleElement<K, V> {
    type Value = V;

    fn new(key: K, value: V) -> Self {
        SimpleElement { key, value }
    }

    fn value(&self) -> &V {
        &self.value
    }

    fn value_mut(&mut self) -> &mut V {
        &mut self.value
    }

    fn key_value_mut(&mut self) -> (&K, &mut V) {
        (&self.key, &mut self.value)
    }

    fn into_kv(self) -> (K, V) {
        (self.key, self.value)
    }
}

macro_rules! impl_plain_element {
    ($($t:ty),* $(,)?) => {
        $(
            impl Element for $t {
                type Key = $t;

                fn key(&self) -> &$t {
                    self
                }
            }
        )*
    };
}

impl_plain_element!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, char, bool, String
);

#[cfg(test)]
mod tests;
