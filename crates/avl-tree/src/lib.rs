use std::alloc::{Layout, dealloc};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::mem::ManuallyDrop;

/// ノード構造体 (アリーナ形式の要素)
struct Node<K, V> {
    key: K,
    val: V,
    parent: usize,
    left: usize,
    right: usize,
    height: usize,
}

/// スロット (存在する要素へのハンドル)
pub struct Slot {
    index: usize,
}

/// 空きスロット (挿入位置の情報を保持)
pub struct VacantSlot {
    parent: usize,
    is_left: bool,
}

/// Slot API ベースの AVL木マップ
pub struct Map<K, V> {
    nodes: Vec<Node<K, V>>,
    root: usize,
    len: usize,
}

impl<K, V> Map<K, V> {
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
    pub fn search<Q: ?Sized + Ord>(&self, k: &Q) -> Result<Slot, VacantSlot>
    where
        K: Borrow<Q>,
    {
        let mut current = self.root;
        let mut parent = usize::MAX;
        let mut is_left = false;

        while current != usize::MAX {
            let node = &self.nodes[current];
            match k.cmp(node.key.borrow()) {
                Ordering::Equal => return Ok(Slot { index: current }),
                Ordering::Less => {
                    parent = current;
                    is_left = true;
                    current = node.left;
                }
                Ordering::Greater => {
                    parent = current;
                    is_left = false;
                    current = node.right;
                }
            }
        }
        Err(VacantSlot { parent, is_left })
    }

    /// Slot からキーと値の不変参照を取得します
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_ref(&self, s: Slot) -> (&K, &V) {
        let node = &self.nodes[s.index];
        (&node.key, &node.val)
    }

    /// Slot からキーの不変参照と値の可変参照を取得します
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_mut(&mut self, s: Slot) -> (&K, &mut V) {
        let node = &mut self.nodes[s.index];
        (&node.key, &mut node.val)
    }

    /// VacantSlot に要素を挿入し、Slot を返します
    pub fn slot_insert(&mut self, v: VacantSlot, k: K, val: V) -> Slot {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            key: k,
            val,
            parent: v.parent,
            left: usize::MAX,
            right: usize::MAX,
            height: 1,
        });

        if v.parent == usize::MAX {
            self.root = new_idx;
        } else {
            if v.is_left {
                self.nodes[v.parent].left = new_idx;
            } else {
                self.nodes[v.parent].right = new_idx;
            }
        }

        // AVL 再平衡化
        let mut current = v.parent;
        while current != usize::MAX {
            self.update_height(current);
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

        self.len += 1;
        Slot { index: new_idx }
    }

    /// 2つのノードのキーと値を交換します (木構造・高さは変更しません)
    fn swap_contents(&mut self, a: usize, b: usize) {
        if a == b {
            return;
        }
        let (lo, hi) = if a < b { (a, b) } else { (b, a) };
        let (head, tail) = self.nodes.split_at_mut(hi);
        std::mem::swap(&mut head[lo].key, &mut tail[0].key);
        std::mem::swap(&mut head[lo].val, &mut tail[0].val);
    }

    /// Slot の要素を削除し、キーと値を返します
    ///
    /// # Safety
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub unsafe fn slot_remove(&mut self, s: Slot) -> (K, V) {
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
        } else {
            if self.nodes[parent].left == del_idx {
                self.nodes[parent].left = child;
            } else {
                self.nodes[parent].right = child;
            }
        }

        if child != usize::MAX {
            self.nodes[child].parent = parent;
        }

        // 削除するキーと値を取り出す。
        // このスロットは直後の上書き (ptr::write) か set_len で消えるため、
        // 取り出した値が二重に Drop されることはない。
        // SAFETY: del_idx は直前まで有効なノードを指している。
        let k = unsafe { std::ptr::read(&self.nodes[del_idx].key) };
        let v = unsafe { std::ptr::read(&self.nodes[del_idx].val) };

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

        // AVL 再平衡化
        let mut current = parent_to_balance;
        while current != usize::MAX {
            self.update_height(current);
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

        self.len -= 1;
        (k, v)
    }

    /// 指定ノードの高さを更新します
    fn update_height(&mut self, idx: usize) {
        let left = self.nodes[idx].left;
        let right = self.nodes[idx].right;
        let lh = if left != usize::MAX { self.nodes[left].height } else { 0 };
        let rh = if right != usize::MAX { self.nodes[right].height } else { 0 };
        self.nodes[idx].height = std::cmp::max(lh, rh) + 1;
    }

    /// 指定ノードのバランス係数を取得します
    fn get_balance(&self, idx: usize) -> i32 {
        if idx == usize::MAX {
            return 0;
        }
        let left = self.nodes[idx].left;
        let right = self.nodes[idx].right;
        let lh = if left != usize::MAX { self.nodes[left].height as i32 } else { 0 };
        let rh = if right != usize::MAX { self.nodes[right].height as i32 } else { 0 };
        lh - rh
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
        } else {
            if self.nodes[x_parent].left == x {
                self.nodes[x_parent].left = y;
            } else {
                self.nodes[x_parent].right = y;
            }
        }

        self.update_height(x);
        self.update_height(y);
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
        } else {
            if self.nodes[y_parent].left == y {
                self.nodes[y_parent].left = x;
            } else {
                self.nodes[y_parent].right = x;
            }
        }

        self.update_height(y);
        self.update_height(x);
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

    /// 不変イテレータを返します
    pub fn iter(&self) -> Iter<'_, K, V> {
        let mut current = self.root;
        if current != usize::MAX {
            while self.nodes[current].left != usize::MAX {
                current = self.nodes[current].left;
            }
        }
        Iter { map: self, current, len: self.len }
    }

    /// 可変イテレータを返します
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        let mut current = self.root;
        if current != usize::MAX {
            while self.nodes[current].left != usize::MAX {
                current = self.nodes[current].left;
            }
        }
        IterMut {
            map: self as *mut Map<K, V>,
            current,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }

    /// 条件を満たす要素を削除しながらイテレートします
    pub fn extract_if<F>(&mut self, f: F) -> ExtractIf<'_, K, V, F>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        let mut current = self.root;
        if current != usize::MAX {
            while self.nodes[current].left != usize::MAX {
                current = self.nodes[current].left;
            }
        }
        ExtractIf { map: self, current, f }
    }
}

impl<K, V> Default for Map<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;

    /// 所有権を消費するイテレータを返します (Drop 込みで安全な実装)
    fn into_iter(self) -> IntoIter<K, V> {
        let len = self.len;
        let mut current = self.root;
        if current != usize::MAX {
            while self.nodes[current].left != usize::MAX {
                current = self.nodes[current].left;
            }
        }
        IntoIter { map: ManuallyDrop::new(self), current, len }
    }
}

// イテレータ実装

pub struct Iter<'a, K, V> {
    map: &'a Map<K, V>,
    current: usize,
    len: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        let node = &self.map.nodes[self.current];
        let res = (&node.key, &node.val);
        self.current = self.map.next_node(self.current);
        self.len -= 1;
        Some(res)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IterMut<'a, K, V> {
    map: *mut Map<K, V>,
    current: usize,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut Map<K, V>>,
}

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        unsafe {
            let map = &mut *self.map;
            let node = &mut map.nodes[self.current];
            let k = &*(&node.key as *const K);
            let v = &mut *(&mut node.val as *mut V);
            self.current = map.next_node(self.current);
            self.len -= 1;
            Some((k, v))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

pub struct IntoIter<K, V> {
    map: ManuallyDrop<Map<K, V>>,
    current: usize,
    len: usize,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 || self.current == usize::MAX {
            return None;
        }
        unsafe {
            let node = std::ptr::read(&self.map.nodes[self.current]);
            self.current = self.map.next_node(self.current);
            self.len -= 1;
            Some((node.key, node.val))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<K, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        unsafe {
            // 残りの要素を in-order で辿って手動で Drop
            while self.len > 0 && self.current != usize::MAX {
                let node = &mut self.map.nodes[self.current];
                std::ptr::drop_in_place(&mut node.key);
                std::ptr::drop_in_place(&mut node.val);
                self.current = self.map.next_node(self.current);
                self.len -= 1;
            }
            // Vec のメモリを手動で解放
            let ptr = self.map.nodes.as_mut_ptr();
            let cap = self.map.nodes.capacity();
            if cap > 0 {
                let layout = Layout::array::<Node<K, V>>(cap).unwrap();
                dealloc(ptr as *mut u8, layout);
            }
        }
    }
}

pub struct ExtractIf<'a, K, V, F>
where
    F: FnMut(&K, &mut V) -> bool,
{
    map: &'a mut Map<K, V>,
    current: usize,
    f: F,
}

impl<'a, K, V, F> Iterator for ExtractIf<'a, K, V, F>
where
    F: FnMut(&K, &mut V) -> bool,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.current != usize::MAX {
            let idx = self.current;
            let next_idx = self.map.next_node(idx);
            let last_idx = self.map.nodes.len() - 1;
            // 子が2つのノードを削除すると、後継のキー・値が idx に移動する
            let has_two_children =
                self.map.nodes[idx].left != usize::MAX && self.map.nodes[idx].right != usize::MAX;

            let should_remove = {
                let node = &mut self.map.nodes[idx];
                (self.f)(&node.key, &mut node.val)
            };

            if should_remove {
                // SAFETY: idx は extract_if が保持している有効なノードのインデックス。
                let res = unsafe { self.map.slot_remove(Slot { index: idx }) };
                // 削除後の次のノード:
                // - 子が2つの場合、後継のキー・値の移動先は、idx 自身が最後尾なら
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
