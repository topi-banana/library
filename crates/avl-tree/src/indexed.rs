//! 順序統計 (k 番目アクセス) 用の [`Indexed`]。
//!
//! `Indexed` を使わない提出では `lib.rs` から `mod indexed;` と
//! `pub use indexed::Indexed;` の 2 行を消すだけで済むように、別ファイルへ
//! 分けています。
//!
//! `Indexed` を使う提出で 1 ファイルにまとめるときは、`lib.rs` の `mod indexed;`
//! を次の形で置き換えると、先頭の `use super::*;` がそのまま通ります。
//!
//! ```text
//! mod indexed {
//!     // ここに indexed.rs の全文
//! }
//! ```

use super::*;

/// 部分木の頂点数を保持する要素
///
/// 任意の [`Element`] を包み、[`Element::update`] のたびに自身を含む部分木の
/// ノード数を数え直します。[`Map`] に載せると [`Map::slot_by_index`] /
/// [`Map::get_by_index`] / [`Map::index_of`] による index アクセス (順序統計) が使えます。
///
/// # Examples
///
/// ```
/// use avl_tree::{Indexed, Map, SimpleElement};
///
/// let mut map: Map<Indexed<SimpleElement<i32, &str>>> = Map::new();
/// map.insert(Indexed::new(SimpleElement::new(2, "two")));
/// map.insert(Indexed::new(SimpleElement::new(1, "one")));
/// map.insert(Indexed::new(SimpleElement::new(3, "three")));
///
/// // in-order で index 1 の要素はキー 2
/// let element = map.get_by_index(1).unwrap();
/// assert_eq!(*element.key(), 2);
/// assert_eq!(*element.value(), "two");
/// assert_eq!(map.index_of(map.slot_by_index(1).unwrap()), 1);
/// ```
#[derive(Clone, Debug)]
pub struct Indexed<E> {
    element: E,
    size: usize,
}

impl<E> Indexed<E> {
    /// 要素を包みます。部分木サイズは 1 で初期化されます。
    pub fn new(element: E) -> Self {
        Indexed { element, size: 1 }
    }

    /// 自身を含む部分木のノード数を返します
    pub fn size(&self) -> usize {
        self.size
    }

    /// 包んでいる要素への参照を返します
    pub fn inner(&self) -> &E {
        &self.element
    }

    /// 包んでいる要素への可変参照を返します
    ///
    /// キーを書き換えると木の順序が壊れるため、値の編集にのみ使ってください。
    pub fn inner_mut(&mut self) -> &mut E {
        &mut self.element
    }

    /// 包んでいる要素を取り出します
    pub fn into_inner(self) -> E {
        self.element
    }
}

impl<E: Element> Element for Indexed<E> {
    type Key = E::Key;

    fn key(&self) -> &Self::Key {
        self.element.key()
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.size = 1 + left.map_or(0, |l| l.size) + right.map_or(0, |r| r.size);
    }

    fn push(&mut self, left: Option<&mut Self>, right: Option<&mut Self>) {
        self.element.push(left.map(|l| &mut l.element), right.map(|r| &mut r.element));
    }

    fn can_absorb(&self, other: &Self) -> bool {
        self.element.can_absorb(&other.element)
    }

    fn absorb(&mut self, other: Self) {
        self.element.absorb(other.element);
    }
}

impl<E: Element> Map<Indexed<E>> {
    /// in-order で `index` 番目 (0 始まり) の要素の Slot を返します
    ///
    /// 計算量は木の高さに比例し `O(log n)` です。
    /// `index` が要素数以上ならば `None` を返します。
    pub fn slot_by_index(&self, mut index: usize) -> Option<Slot> {
        let mut current = self.root;
        while current != usize::MAX {
            let node = &self.nodes[current];
            let left_size =
                if node.left == usize::MAX { 0 } else { self.nodes[node.left].element.size };
            match index.cmp(&left_size) {
                Ordering::Less => current = node.left,
                Ordering::Equal => return Some(Slot { index: current }),
                Ordering::Greater => {
                    index -= left_size + 1;
                    current = node.right;
                }
            }
        }
        None
    }

    /// `index` 番目 (0 始まり) の要素への参照を返します
    ///
    /// 計算量は `O(log n)` です。`index` が要素数以上ならば `None` を返します。
    pub fn get_by_index(&self, index: usize) -> Option<&E> {
        let slot = self.slot_by_index(index)?;
        // SAFETY: slot はこのマップの直前の探索が返した有効な Slot である。
        Some(unsafe { self.slot_ref(slot) }.inner())
    }

    /// Slot の in-order 位置 (0 始まり) を返します
    ///
    /// 計算量は `O(log n)` です。
    ///
    /// `s` はこのマップに対して有効な Slot である必要があります。
    pub fn index_of(&self, s: Slot) -> usize {
        let node = &self.nodes[s.index];
        let mut index =
            if node.left == usize::MAX { 0 } else { self.nodes[node.left].element.size };
        let mut current = s.index;
        loop {
            let parent = self.nodes[current].parent;
            if parent == usize::MAX {
                return index;
            }
            if self.nodes[parent].right == current {
                let left = self.nodes[parent].left;
                index += 1 + if left == usize::MAX { 0 } else { self.nodes[left].element.size };
            }
            current = parent;
        }
    }
}

impl<K: Ord, V> Map<Indexed<SimpleElement<K, V>>> {
    /// キーと値を追加し、スロットを返します
    ///
    /// 同じキーがすでにある場合は値を置き換えます。
    pub fn put(&mut self, key: K, value: V) -> Slot {
        self.insert(Indexed::new(SimpleElement::new(key, value)))
    }

    /// キーに対応する値への参照を返します
    pub fn get<Q: ?Sized + Ord>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_ref(slot) }.inner().value()),
            Err(_) => None,
        }
    }

    /// キーに対応する値への可変参照を返します
    ///
    /// `Indexed` の部分木サイズはキーと構造だけで決まるため、
    /// 値を書き換えても [`Map::slot_refresh`] は不要です。
    pub fn get_mut<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_mut(slot) }.inner_mut().value_mut()),
            Err(_) => None,
        }
    }

    /// キーに対応する要素を削除し、値を返します
    pub fn remove<Q: ?Sized + Ord>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        match self.search(key) {
            Ok(slot) => Some(unsafe { self.slot_remove(slot) }.into_inner().value),
            Err(_) => None,
        }
    }
}
