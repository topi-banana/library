//! [`Element`] トレイトで集約値を差し替えられる AVL 木マップです。
//!
//! キーの比較と平衡化は木側が行い、部分木の集約値 (サイズ、和、マージ結果など) は
//! 要素側の [`Element::update`] が子から計算します。素のキーと値の組には
//! [`SimpleElement`] を使ってください。
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

use std::alloc::{Layout, dealloc};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::mem::ManuallyDrop;

mod indexed;

pub use indexed::Indexed;

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

/// ノード構造体 (アリーナ形式の要素)
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
/// 用途ごとの振る舞いを差し替えられます。
pub struct Map<E: Element> {
    nodes: Vec<Node<E>>,
    root: usize,
    len: usize,
}

impl<E: Element> Map<E> {
    /// 新しい空のマップを生成します
    pub fn new() -> Self {
        Map { nodes: Vec::new(), root: usize::MAX, len: 0 }
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
    /// [`Map::insert`] と違い、同一キーの置き換えや吸収は行いません。
    /// 挿入する要素は葉として正規化するため、先に [`Element::update`] を呼びます。
    pub fn slot_insert(&mut self, v: VacantSlot, mut element: E) -> Slot {
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
    pub fn insert(&mut self, element: E) -> Slot {
        let mut element = element;
        element.update(None, None);

        // 同一キーの既存要素を吸収するか、そのスロットを上書きする
        if let Ok(slot) = self.search(element.key()) {
            if element.can_absorb(unsafe { self.slot_ref(slot) }) {
                let other = unsafe { self.slot_remove(slot) };
                element.absorb(other);
            } else {
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

        let has_left = self.nodes[del_idx].left != usize::MAX;
        let has_right = self.nodes[del_idx].right != usize::MAX;

        // 子が2つある場合、後継とデータを交換して削除対象を移動
        if has_left && has_right {
            let mut s_idx = self.nodes[del_idx].right;
            while self.nodes[s_idx].left != usize::MAX {
                s_idx = self.nodes[s_idx].left;
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

    /// 不変イテレータを返します
    pub fn iter(&self) -> Iter<'_, E> {
        let current = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        Iter { map: self, current, len: self.len }
    }

    /// 可変イテレータを返します
    ///
    /// 集約値を持つ要素の値を書き換えた場合は、[`Map::slot_refresh`] で
    /// 根まで再計算する必要があります。
    pub fn iter_mut(&mut self) -> IterMut<'_, E> {
        let current = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        IterMut {
            map: self as *mut Map<E>,
            current,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }

    /// 条件を満たす要素を削除しながらイテレートします
    pub fn extract_if<F>(&mut self, f: F) -> ExtractIf<'_, E, F>
    where
        F: FnMut(&E) -> bool,
    {
        let current = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        ExtractIf { map: self, current, f }
    }
}

impl<E: Element> Default for Map<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Element> IntoIterator for Map<E> {
    type Item = E;
    type IntoIter = IntoIter<E>;

    /// 所有権を消費するイテレータを返します (Drop 込みで安全な実装)
    fn into_iter(self) -> IntoIter<E> {
        let len = self.len;
        let current = match self.first() {
            Some(slot) => slot.index,
            None => usize::MAX,
        };
        IntoIter { map: ManuallyDrop::new(self), current, len }
    }
}

// イテレータ実装

pub struct Iter<'a, E: Element> {
    map: &'a Map<E>,
    current: usize,
    len: usize,
}

impl<'a, E: Element> Iterator for Iter<'a, E> {
    type Item = &'a E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        let node = &self.map.nodes[self.current];
        let element = &node.element;
        self.current = self.map.next_node(self.current);
        self.len -= 1;
        Some(element)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IterMut<'a, E: Element> {
    map: *mut Map<E>,
    current: usize,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut Map<E>>,
}

impl<'a, E: Element> Iterator for IterMut<'a, E> {
    type Item = &'a mut E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[self.current];
            let element = &mut *(&mut node.element as *mut E);
            self.current = map.next_node(self.current);
            self.len -= 1;
            Some(element)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IntoIter<E: Element> {
    map: ManuallyDrop<Map<E>>,
    current: usize,
    len: usize,
}

impl<E: Element> Iterator for IntoIter<E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        unsafe {
            let element = std::ptr::read(&self.map.nodes[self.current].element);
            self.current = self.map.next_node(self.current);
            self.len -= 1;
            Some(element)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<E: Element> Drop for IntoIter<E> {
    fn drop(&mut self) {
        unsafe {
            // 残りの要素を in-order で辿って手動で Drop
            while self.len > 0 && self.current != usize::MAX {
                let node = &mut self.map.nodes[self.current];
                std::ptr::drop_in_place(&mut node.element);
                self.current = self.map.next_node(self.current);
                self.len -= 1;
            }
            // Vec のメモリを手動で解放
            let ptr = self.map.nodes.as_mut_ptr();
            let cap = self.map.nodes.capacity();
            if cap > 0 {
                let layout = Layout::array::<Node<E>>(cap).unwrap();
                dealloc(ptr as *mut u8, layout);
            }
        }
    }
}

pub struct ExtractIf<'a, E: Element, F>
where
    F: FnMut(&E) -> bool,
{
    map: &'a mut Map<E>,
    current: usize,
    f: F,
}

impl<'a, E: Element, F> Iterator for ExtractIf<'a, E, F>
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
                // SAFETY: idx は extract_if が保持している有効なノードのインデックス。
                let res = unsafe { self.map.slot_remove(Slot { index: idx }) };
                // 削除後の次のノード:
                // - 子が2つの場合、後継の要素の移動先は、idx 自身が最後尾なら
                //   後継のスロット (next_idx)、そうでなければ idx
                // - 子が1つ以下の場合、最後尾のノード (next_idx) が idx に
                //   移動していれば idx、そうでなければ next_idx がそのまま次のノード
                self.current = if has_two_children {
                    if last_idx == idx { next_idx } else { idx }
                } else if next_idx == last_idx {
                    idx
                } else {
                    next_idx
                };
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

impl<K: Ord, V> Map<SimpleElement<K, V>> {
    /// キーと値を追加し、スロットを返します
    ///
    /// 同じキーがすでにある場合は値を置き換えます。
    pub fn put(&mut self, key: K, value: V) -> Slot {
        self.insert(SimpleElement::new(key, value))
    }

    /// キーに対応する値への参照を返します
    pub fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_ref(slot) }.value()),
            Err(_) => None,
        }
    }

    /// キーに対応する値への可変参照を返します
    pub fn get_mut<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_mut(slot) }.value_mut()),
            Err(_) => None,
        }
    }

    /// キーに対応する要素を削除し、値を返します
    pub fn remove<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_remove(slot) }.value),
            Err(_) => None,
        }
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
