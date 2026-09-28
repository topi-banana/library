# AvlTree — Element で拡張できる順序付きマップ

`Element` トレイトを実装した要素を載せられる AVL 木のマップです。
キーの比較と木の平衡化はライブラリ側が行うので、利用者は
「部分木にどんな集約値を持たせるか」だけを `Element::update` に書きます。
集約値の例は部分木サイズ (→ k 番目アクセス)、部分木和、重なる区間の統合などです。
遅延タグを使う要素を載せれば、キーの区間に対する作用と区間の集約
(遅延セグメント木) もできます。

- 実装: [`crates/avl-tree/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/avl-tree/src/lib.rs) — [全文はこのページの末尾](#ソース)
    - 順序統計 (`Indexed`) は [`crates/avl-tree/src/indexed.rs`](https://github.com/topi-banana/library/blob/main/crates/avl-tree/src/indexed.rs) に分離
    - 遅延セグメント木 (`LazySegmentTree`) は [`crates/avl-tree/src/lazy_segment_tree.rs`](https://github.com/topi-banana/library/blob/main/crates/avl-tree/src/lazy_segment_tree.rs) に分離
- verify:
    - `Indexed` — [yukicoder No.649 ここでちょっとQK！](https://yukicoder.me/problems/no/649),
      [No.3298 K-th Slime](https://yukicoder.me/problems/no/3298)
    - `LazySegmentTree` — [Library Checker Range Affine Range Sum](https://judge.yosupo.jp/problem/range_affine_range_sum)

以下 `n` はマップの要素数です。

素のキーと値の組には `SimpleElement`、k 番目に小さい要素の取得 (順序統計) には
`Indexed`、キー範囲への作用と区間の集約には `LazySegmentTree` を使います。
どれも `Element` を実装した普通の要素なので、
必要なら自分で `Element` を実装して差し替えられます。
`i32` などのプリミティブ型と `String` には `Element` を実装済みで、
`Map<i32>` のようにそのまま重複なしの集合として使えます。

`SimpleElement` と `Indexed` は `MapElement` も実装しています。`MapElement` を
実装した要素を載せると、`Map` に `BTreeMap` 互換の API (`insert` / `entry` /
`range` / `append` / `split_off` / `retain` など) が生えます。

## API

木に載せる要素が実装する `Element` / `MapElement` トレイトと、それを載せる
`Map`、標準の要素である `SimpleElement` / `Indexed` の順に並べます。

### Element

木に載せる要素が実装するトレイトです。キーの型 `Key` と `key()` は必須で、
残りは既定実装を持ちます。

| 項目                               | 計算量 | 説明                                                        |
| ---------------------------------- | ------ | ----------------------------------------------------------- |
| `type Key: Ord`                    | —      | 順序付けに使うキーの型                                      |
| `key(&self) -> &Key`               | `O(1)` | キーへの参照を返す (必須)                                   |
| `update(&mut self, left, right)`   | `O(1)` | 子が確定したあと部分木の集約値を再計算する                  |
| `push(&mut self, left, right)`     | `O(1)` | 溜めた遅延タグを子へ流す。回転前と挿入・削除・区間操作の経路で呼ばれる |
| `can_absorb(&self, other) -> bool` | `O(1)` | 追加時に `other` を吸収できるかを返す。既定では常に `false` |
| `absorb(&mut self, other)`         | `O(1)` | `can_absorb` が `true` のときに `other` を取り込む          |

`update` の引数は `Option<&Self>` で、挿入・削除・回転の再平衡パスで呼ばれます。
集約値を持つ要素は

```rust,ignore
self.sum = self.value + left.map_or(0, |l| l.sum) + right.map_or(0, |r| r.sum);
```

のように、子の集約値から自分の集約値を計算します。

### MapElement

キーと値の組を載せる要素が実装するトレイトです。`Element` を継承し、
`SimpleElement<K, V>` と `Indexed<E: MapElement>` に実装しています。

| 項目                                             | 説明                             |
| ------------------------------------------------ | -------------------------------- |
| `type Value`                                     | 値の型                           |
| `new(key, value) -> Self`                        | キーと値から要素を作る           |
| `value(&self) -> &Value`                         | 値への参照                       |
| `value_mut(&mut self) -> &mut Value`             | 値への可変参照                   |
| `key_value_mut(&mut self) -> (&Key, &mut Value)` | キーと値への可変参照を同時に返す |
| `into_kv(self) -> (Key, Value)`                  | キーと値を取り出す               |

`Element` だけを実装した要素でも `Map` は使えますが、`MapElement` を実装すると
`Map` に `BTreeMap` 互換の API (`insert` / `entry` / `range` / `append` など) が
生えます。集約値は値に依存しないように実装してください。値の変更は `get_mut` などで
直接行えるため、値に依存する集約値は更新されません。

### Map (Element レベル)

`Map<E>` が本体です。要素 `E` が `Element` を実装している必要があります。
こちらは要素そのものと木の構造を扱う API です。

| 項目                                      | 計算量                      | 説明                                                |
| ----------------------------------------- | --------------------------- | --------------------------------------------------- |
| `Map::new()`                              | `O(1)`                      | 空のマップを作る                                    |
| `Map::with_capacity(n)`                   | `O(1)`                      | `n` 個分の容量を確保した空のマップを作る            |
| `len()` / `is_empty()` / `capacity()`     | `O(1)`                      | 要素数 / 空かどうか / 内部のノード配列の容量        |
| `reserve` / `try_reserve` (`_exact` 付き) | `O(n)`                      | 追加の容量を確保する (`Vec` と同じ意味論)           |
| `shrink_to_fit()` / `shrink_to(n)`        | `O(n)`                      | 余分な容量を解放する                                |
| `clear()`                                 | `O(n)`                      | すべての要素を削除する (容量は保持)                 |
| `search(&key)`                            | `O(log n)`                  | 一致する Slot、無ければ挿入位置の VacantSlot を返す |
| `contains(&key)`                          | `O(log n)`                  | キーを持つ要素があるか                              |
| `predecessor(&key)` / `successor(&key)`   | `O(log n)`                  | `key` より小さい最大 / 大きい最小の要素の Slot      |
| `first()` / `last()`                      | `O(log n)`                  | in-order の最初 / 最後の要素の Slot                 |
| `next(slot)` / `prev(slot)`               | `O(log n)`                  | Slot の in-order 後継 / 先行                        |
| `insert_element(element)`                 | ならし `O(log n)`           | 要素を追加し Slot を返す。同一キーは吸収か置き換え  |
| `slot_insert(vacant, element)`            | `O(log n)`                  | 空きスロットへ要素を追加する                        |
| `slot_remove(slot)`                       | `O(log n)`                  | Slot の要素を削除して返す (unsafe)                  |
| `slot_ref(slot)` / `slot_mut(slot)`       | `O(1)`                      | 要素への参照 / 可変参照 (unsafe)                    |
| `slot_refresh(slot)`                      | `O(log n)`                  | Slot の要素と祖先の集約値を再計算する (unsafe)      |
| `iter_elements()` / `iter_mut_elements()` | 1 要素あたりならし `O(1)`   | in-order の要素の不変 / 可変イテレータ              |
| `into_elements()`                         | `O(n)`                      | 所有権を消費して要素を取り出すイテレータ            |
| `extract_elements_if(f)`                  | 1 個あたりならし `O(log n)` | 条件を満たす要素を削除しながら返す                  |
| `default()`                               | `O(1)`                      | 空のマップ                                          |

`search` などが返す `Slot` は `slot_ref` や `slot_remove` に渡すハンドルです。

| 型           | 説明                                                                         |
| ------------ | ---------------------------------------------------------------------------- |
| `Slot`       | 存在する要素へのハンドル。`Copy` / `Eq` / `Debug`                            |
| `VacantSlot` | `search` が失敗したときの挿入位置。`None` / `Left(parent)` / `Right(parent)` |

`Slot` を組み立てられるのは木自身だけです。逆に、任意の `Slot` を渡してよい
わけではなく、**そのマップに対して有効な `Slot`** を渡す責任が利用者にあります。
だから `slot_*` は `unsafe` なのです。

`insert_element` は同じキーの要素を見つけると `Element::can_absorb` を試し、
`true` なら相手を削除して `Element::absorb` で吸収します。
さらに前後の隣接要素に対しても吸収できる限り繰り返します。
1 回の `insert_element` で複数の要素を吸収することがありますが、
吸収されて消える要素は 1 回しか消えないため、ならし計算量は `O(log n)` です。

イテレータ型 `IterElements` / `IterMutElements` / `IntoElements` /
`ExtractElementsIf` は `Iterator` を実装しています。

### Map (MapElement レベル)

要素が `MapElement` を実装しているとき、`Map<E>` に `BTreeMap` 互換の API が
生えます。以下 `K = E::Key`、`V = E::Value` とします。

| 項目                                       | 計算量                    | 説明                                             |
| ------------------------------------------ | ------------------------- | ------------------------------------------------ |
| `insert(key, value)`                       | `O(log n)`                | 追加し、同じキーの以前の値を返す                 |
| `put(key, value)`                          | ならし `O(log n)`         | 追加し Slot を返す。同じキーは置き換え           |
| `entry(key)`                               | `O(log n)`                | `Entry` (`Occupied` / `Vacant`) を返す           |
| `get(&key)` / `get_mut(&key)`              | `O(log n)`                | 値への参照 / 可変参照                            |
| `get_key_value(&key)`                      | `O(log n)`                | キーと値への参照                                 |
| `contains_key(&key)`                       | `O(log n)`                | キーが存在するか                                 |
| `get_disjoint_mut([&k1, &k2, ..])`         | `O(N^2 + N log n)`        | 最大 `N` 個の値への可変参照。重複キーは panic    |
| `first_key_value()` / `last_key_value()`   | `O(log n)`                | 最小 / 最大キーの組                              |
| `first_entry()` / `last_entry()`           | `O(log n)`                | 最小 / 最大キーの `OccupiedEntry`                |
| `pop_first()` / `pop_last()`               | `O(log n)`                | 最小 / 最大キーの組を取り出して削除              |
| `remove(&key)` / `remove_entry(&key)`      | `O(log n)`                | 値を削除して返す / キーと値を削除して返す        |
| `retain(f)`                                | `O(n log n)`              | 条件を満たさない要素を削除する                   |
| `append(&mut other)`                       | `O(m log(n + m))`         | `other` の全要素 (`m` 個) を移動し `other` を空に |
| `split_off(&key)`                          | `O(k log n)`              | `key` 以上の要素 (`k` 個) を新しい `Map` に移す  |
| `range(range)` / `range_mut(range)`        | 1 要素あたりならし `O(1)` | キー範囲の `(&K, &V)` / `(&K, &mut V)`           |
| `iter()` / `iter_mut()`                    | 1 要素あたりならし `O(1)` | 全要素の `(&K, &V)` / `(&K, &mut V)`             |
| `keys()` / `values()` / `values_mut()`     | 1 要素あたりならし `O(1)` | キー / 値 / 値の可変イテレータ                   |
| `into_keys()` / `into_values()`            | `O(n)`                    | 所有権を消費してキー / 値を取り出す              |
| `extract_if(range, pred)`                  | 1 個あたりならし `O(log n)` | 範囲内で条件を満たす要素を削除しながら返す     |
| `drain()`                                  | `O(n log n)`              | すべての要素を削除しながら返す                   |

`entry` は `or_insert` / `or_insert_with` / `or_insert_with_key` / `or_default` /
`and_modify` / `insert_entry` / `remove` などを持つ `Entry` を返します。
`range` は `start..end` などの `RangeBounds` を取り、`..` で全要素になります。
範囲の両端が逆転している場合は `BTreeMap` と同じく panic します
(`range_mut` / `extract_if` も同様)。

`Index` (`map[&key]`)、`FromIterator` / `Extend` (`collect` / `extend`)、
`IntoIterator` (所有で `(K, V)`、`&Map` で `(&K, &V)`、`&mut Map` で `(&K, &mut V)`)、
`Clone` / `Debug` / `PartialEq` / `Eq` / `PartialOrd` / `Ord` / `Hash` も
実装しています。

イテレータ型 `Iter` / `IterMut` / `IntoIter` / `Range` / `RangeMut` / `Keys` /
`Values` / `ValuesMut` / `IntoKeys` / `IntoValues` / `ExtractIf` / `Drain` は
`Iterator` を実装しています。

### SimpleElement

キーと値をそのまま載せる要素です。これを使うと普通のマップになります。
`MapElement` を実装しているので、`Map<SimpleElement<K, V>>` では
`MapElement` レベルの API (`insert` / `get` / `entry` / `range` など、
値の型は `V`) がそのまま使えます。

| 項目                                | 計算量 | 説明                     |
| ----------------------------------- | ------ | ------------------------ |
| `SimpleElement::new(key, value)`    | `O(1)` | キーと値の組を作る       |
| `key()` / `value()` / `value_mut()` | `O(1)` | キー / 値 / 値の可変参照 |

### Indexed

部分木の頂点数を保持する要素です。任意の要素を包み、`Map<Indexed<E>>` に載せると
in-order の位置 (index) で要素を引けます。

`Indexed` は `indexed.rs` に分かれています。順序統計を使う問題では `lib.rs` の
`mod indexed;` を `indexed.rs` のインラインモジュールに置き換えます ([ソース](#indexed-も使う) 参照)。

| 項目                                       | 計算量 | 説明                                       |
| ------------------------------------------ | ------ | ------------------------------------------ |
| `Indexed::new(element)`                    | `O(1)` | 要素を包む。部分木サイズは 1               |
| `size()`                                   | `O(1)` | 自身を含む部分木のノード数                 |
| `inner()` / `inner_mut()` / `into_inner()` | `O(1)` | 包んでいる要素の参照 / 可変参照 / 取り出し |

`Map<Indexed<E>>` には追加で次が生えます。

| 項目                   | 計算量     | 説明                                |
| ---------------------- | ---------- | ----------------------------------- |
| `slot_by_index(index)` | `O(log n)` | index 番目 (0 始まり) の要素の Slot |
| `get_by_index(index)`  | `O(log n)` | index 番目の要素への参照            |
| `index_of(slot)`       | `O(log n)` | Slot の in-order 位置 (0 始まり)    |

`Indexed<E>` は `E` が `MapElement` のときに `MapElement` を実装するので、
`Map<Indexed<SimpleElement<K, V>>>` などでも `MapElement` レベルの API が
使えます (`Map<SimpleElement<K, V>>` と同じ意味論)。

`size()` は「自身を含む」部分木のノード数です。`slot_by_index` は節点で
左部分木のサイズと求める index を比べて左右に降りるので、
`O(log n)` で目的の要素に着きます。`index_of` は逆に、親をたどりながら
「自分が右の子なら、1 + 左兄弟のサイズ」を足し上げます。

### LazySegmentTree

キーの区間に対する作用 (遅延タグ) と区間の集約を載せる要素です。
集約は `Monoid`、作用は `Action` として実装します。キーは index に限らず、
任意の順序キーを使えます。

`LazySegmentTree` は `lazy_segment_tree.rs` に分かれています。使う問題では
`lib.rs` の `mod lazy_segment_tree;` を `lazy_segment_tree.rs` の
インラインモジュールに置き換えます ([ソース](#lazysegmenttree-も使う) 参照)。

| トレイト  | 項目                        | 説明                                 |
| --------- | --------------------------- | ------------------------------------ |
| `Monoid`  | `type S: Clone`             | 値と集約値の型                       |
| `Monoid`  | `op(&a, &b) -> S`           | 2 つの値の結合                       |
| `Monoid`  | `identity() -> S`           | 空区間の値 (単位元)                  |
| `Action`  | `type F: Clone`             | 作用 (遅延タグ) の型                 |
| `Action`  | `mapping(&f, &s) -> S`      | 値 `s` に作用 `f` を適用した結果     |
| `Action`  | `composition(&f, &g) -> F`  | `g` のあとに `f` を適用する作用      |
| `Action`  | `id() -> F`                 | 何もしない作用 (恒等作用)            |

`Action` は `Monoid` を継承します。`S` は部分木の集約値にもなるので、
`mapping` は「部分木の各要素に作用を適用した集約値」を返す必要があります。
たとえば区間加算・区間和なら `S = (和, 要素数)` にして
`mapping(f, s) = (s.0 + f * s.1, s.1)` とします。

`LazySegmentTree` は次のメソッドを持ちます。

| 項目                                | 計算量 | 説明                                   |
| ----------------------------------- | ------ | -------------------------------------- |
| `LazySegmentTree::new(key, value)`  | `O(1)` | キーと値から要素を作る                 |
| `key()` / `value()` / `aggregate()` | `O(1)` | キー / タグ未適用の値 / 部分木の集約値 |
| `min_key()` / `max_key()`           | `O(1)` | 部分木のキー範囲                       |
| `into_value()`                      | `O(1)` | 値を取り出す                           |

`Map<LazySegmentTree<K, A>>` には追加で次が生えます。

| 項目              | 計算量            | 説明                                               |
| ----------------- | ----------------- | -------------------------------------------------- |
| `put(key, value)` | ならし `O(log n)` | キーと値を追加し Slot を返す。同じキーは置き換え   |
| `get(&key)`       | `O(log n)`        | キーに対応する値を返す                             |
| `remove(&key)`    | `O(log n)`        | キーに対応する要素を削除し、作用適用済みの値を返す |
| `prod(range)`     | `O(log n)`        | キーが `range` に入る要素の集約値                  |
| `apply(range, f)` | `O(log n)`        | キーが `range` に入る要素に作用 `f` を適用         |
| `all_prod()`      | `O(1)`            | すべての要素の集約値                               |

`get` / `remove` / `prod` / `apply` は経路の遅延タグを子へ流すため
`&mut self` を取ります。`range` はキーの半開区間 `l..r` です。
空区間を渡した場合、`prod` は `identity()` を返し、`apply` は何もしません。

`Indexed` と併用するときは `Map<Indexed<LazySegmentTree<K, A>>>` と外側に重ねます。
`put` / `get` / `remove` / `prod` / `apply` / `all_prod` と、
`slot_by_index` / `get_by_index` / `index_of` の両方が使えます。

挿入・削除・回転のときは、遅延タグの適用範囲が変わらないように木が先に
`Element::push` を呼びます。そのため `put` で新しく入る値に、それまで同じ
位置にあった作用は乗りません (先に作用を適用してから `put` したのと同じです)。

## 使用例

### ソート済みマップとして使う

```rust
use avl_tree::{Map, SimpleElement};

let mut map: Map<SimpleElement<u32, &str>> = Map::new();
map.put(2, "two");
map.put(1, "one");
map.put(3, "three");
map.put(2, "TWO"); // 同じキーは値を置き換える

assert_eq!(map.get(&2), Some(&"TWO"));
assert_eq!(map.remove(&2), Some("TWO"));

// BTreeMap と同じ entry API
map.entry(4).or_insert("four");
map.entry(4).and_modify(|value| *value = "FOUR");
assert_eq!(map.get_key_value(&4), Some((&4, &"FOUR")));

// キー範囲のイテレータ (start..end, ..=end など)
let range: Vec<(u32, &str)> = map.range(2..=4).map(|(&k, &v)| (k, v)).collect();
assert_eq!(range, vec![(3, "three"), (4, "FOUR")]);

let keys: Vec<u32> = map.keys().copied().collect();
assert_eq!(keys, vec![1, 3, 4]);
```

### Element を自分で実装する

部分木和を持つ要素です。`update` が子の `sum` から自分の `sum` を作ります。

```rust
use avl_tree::{Element, Map};

struct SubtreeSum {
    key: i32,
    value: i64,
    sum: i64,
}

impl Element for SubtreeSum {
    type Key = i32;

    fn key(&self) -> &i32 {
        &self.key
    }

    fn update(&mut self, left: Option<&Self>, right: Option<&Self>) {
        self.sum = self.value + left.map_or(0, |l| l.sum) + right.map_or(0, |r| r.sum);
    }
}

let mut map: Map<SubtreeSum> = Map::new();
for key in [2, 1, 3] {
    map.insert_element(SubtreeSum { key, value: i64::from(key), sum: 0 });
}

// この入力ではキー 2 のノードが根になるので、その集約値は全体の和
let root = map.search(&2).unwrap();
assert_eq!(unsafe { map.slot_ref(root) }.sum, 6);
```

### k 番目に小さい要素を取る

重複を許す集合は、値に通し番号を足してキーを一意にします。

```rust
use avl_tree::{Indexed, Map, SimpleElement};

let mut set: Map<Indexed<SimpleElement<(i64, usize), i64>>> = Map::new();
set.put((5, 0), 5);
set.put((1, 1), 1);
set.put((5, 2), 5);

assert_eq!(set.len(), 3);
assert_eq!(*set.get_by_index(0).unwrap().value(), 1); // 最小
assert_eq!(*set.get_by_index(1).unwrap().value(), 5); // 2 番目
assert_eq!(*set.get_by_index(2).unwrap().value(), 5); // 最大
assert!(set.get_by_index(3).is_none());

// Slot から位置 (rank) を引くこともできる
let slot = set.slot_by_index(2).unwrap();
assert_eq!(set.index_of(slot), 2);

// K 番目を取り出して削除する
let slot = set.slot_by_index(1).unwrap();
assert_eq!(*unsafe { set.slot_ref(slot) }.inner().value(), 5);
unsafe { set.slot_remove(slot) };
assert_eq!(set.len(), 2);
```

### キー範囲への作用と区間の集約 (遅延セグメント木)

区間加算・区間和を載せた例です。部分木の集約値は複数要素の和になるので、
`mapping` が足し込む個数を知れるように `S` は `(和, 要素数)` にします。

```rust
use avl_tree::{Action, LazySegmentTree, Map, Monoid};

/// 区間加算・区間和
enum AddSum {}

impl Monoid for AddSum {
    /// (和, 要素数)
    type S = (i64, i64);

    fn op(a: &(i64, i64), b: &(i64, i64)) -> (i64, i64) {
        (a.0 + b.0, a.1 + b.1)
    }

    fn identity() -> (i64, i64) {
        (0, 0)
    }
}

impl Action for AddSum {
    type F = i64;

    fn mapping(f: &i64, s: &(i64, i64)) -> (i64, i64) {
        (s.0 + f * s.1, s.1)
    }

    fn composition(f: &i64, g: &i64) -> i64 {
        f + g
    }

    fn id() -> i64 {
        0
    }
}

let mut seg: Map<LazySegmentTree<u32, AddSum>> = Map::new();
seg.put(1, (10, 1));
seg.put(3, (20, 1));
seg.put(5, (30, 1));

// キーが [1, 4) の要素に 5 を足す (キー 1 と 3)
seg.apply(1..4, 5);

assert_eq!(seg.prod(1..4), (35, 2));
assert_eq!(seg.prod(0..10), (65, 3));
assert_eq!(seg.all_prod(), (65, 3));
assert_eq!(seg.get(&5), Some(&(30, 1))); // 区間の外は変わらない
assert_eq!(seg.remove(&3), Some((25, 1))); // 削除は作用適用済みの値を返す
```

`Indexed` を外側に重ねると、キー範囲の操作と index アクセスを併用できます
(`AddSum` は上と同じものを定義してください)。

```rust
use avl_tree::{Indexed, LazySegmentTree, Map};

let mut seg: Map<Indexed<LazySegmentTree<u32, AddSum>>> = Map::new();
seg.put(1, (10, 1));
seg.put(3, (20, 1));

seg.apply(1..4, 5);
assert_eq!(seg.prod(1..4), (35, 1));

// in-order の index アクセス
assert_eq!(*seg.get_by_index(0).unwrap().key(), 1);
assert_eq!(seg.index_of(seg.slot_by_index(1).unwrap()), 1);
```

## 注意点

### Slot は削除を挟むと別の要素を指しうる

`slot_remove` は要素数 `n` の木から `O(log n)` で削除するために、
ノード配列の末尾のノードを削除位置へ移すスワップ削除で実装しています。
また、子が 2 つあるノードの削除では後継の要素と中身を交換します。
そのため、**ある要素を削除すると、その Slot だけでなく他の Slot も
無効になったり別の要素を指したりします**。

削除のあとも Slot を使いたい場合は `search` や `slot_by_index` で取り直してください。
上の例のように「取ってきたらすぐ `slot_remove` に渡す」使い方なら問題ありません。

### キーを書き換えない

`slot_mut` や `Indexed::inner_mut` でキーを書き換えると木の順序が壊れ、
検索や削除が誤動作します。書き換えてよいのは値だけです。
キーを変えたい場合は一度削除して追加し直してください。
`MapElement` レベルの `get_mut` / `iter_mut` / `range_mut` はキーを不変で
返すので、こちらを使えばキーを壊す心配はありません。

### 集約値の編集には slot_refresh が要る

`slot_mut` で `update` の対象になるフィールド (部分木和など) を直接編集した場合、
木は変更に気づけません。`slot_refresh(slot)` でそのノードから根まで
集約値を再計算してください。

`MapElement` レベルの `get_mut` / `iter_mut` / `values_mut` / `range_mut` や、
Element レベルの `iter_mut_elements` で値を書き換えた場合も同じです。
値が集約値に影響する要素では、書き換えたノードごとに `slot_refresh` を呼んで
ください (`Indexed` のように集約値が値によらない要素では不要です)。

`Indexed` の部分木サイズはキーと木の構造だけで決まるので、
`Indexed` を使っているときはこの再計算は不要です。

`LazySegmentTree` では `get` / `remove` / `prod` / `apply` が内部で経路の遅延タグを
子へ流すため、集約値の再計算は木が面倒を見ます。値の更新は `put` や `apply` で
行ってください (`slot_mut` で `LazySegmentTree` の中身を直接書き換える方法は
用意していません)。

### 重複キーは持てない

`Map` はキーが一意であることを前提にしています。同じキーを追加すると
吸収されるか置き換えられます。重複を許すマルチセットが必要なときは、
`Indexed` の例のようにキーに通し番号を足して一意にしてください。

## verify

- [yukicoder No.649 ここでちょっとQK！](https://yukicoder.me/problems/no/649)
  — 集合への追加と「K 番目に小さい値を出力して削除」を処理します。
  `put` / `slot_by_index` / `slot_ref` / `slot_remove` が検証されます。
- [yukicoder No.3298 K-th Slime](https://yukicoder.me/problems/no/3298)
  — 追加と「K 番目を取り出して値を増やして戻す」、K 番目の出力を処理します。
  `slot_by_index` / `slot_remove` / 再挿入の `put` / `get_by_index` が検証されます。
- [Library Checker Range Affine Range Sum](https://judge.yosupo.jp/problem/range_affine_range_sum)
  — 区間アフィン変換と区間和のクエリを処理します。
  `put` / `apply` / `prod` が検証されます。

yukicoder の 2 問は値が重複しうるので、キーに通し番号を足して一意化しています。

## 実装メモ

ノードは `Vec<Node<E>>` のアリーナに置き、親・左・右の添字と高さを持たせています。
`Slot` はこの配列の添字そのものです。

`update` は挿入・削除のあと `rebalance_from` が根に向かって
`update_node` を呼ぶ形で伝わります。回転の直後は
「下のノード → 上のノード」の順に `update_node` を呼ぶため、
子の集約値が確定してから親がそれを読めます。

削除はスワップ削除です。要素を 1 つ消すには
「削除対象を配列の末尾のノードと置き換えてから長さを 1 減らす」だけで済み、
配列の穴を埋め直す `O(n)` の仕事がありません。代わりに `Slot` の安定性を
犠牲にしています ([注意点](#slot-は削除を挟むと別の要素を指しうる) 参照)。

`Indexed` は `update` で `1 + 左の size + 右の size` を計算するだけです。
木側は `Indexed` の内部 (サイズ) を特別扱いせず、
「要素の `update` を呼ぶ」という一般の仕組みだけで順序統計を実現しています。
`slot_by_index` と `index_of` だけは集約値の意味を知っている必要があるため、
`Map<Indexed<E>>` の専用メソッドとして実装しています。

`LazySegmentTree` は各ノードに「未適用の生の値」「作用適用済みの集約値」
「部分木に溜まった作用」を持ちます。集約値は常に
`mapping(lazy, op(value, 左の集約値, 右の集約値))` で計算できるので、
遅延タグが溜まっていても `update` だけで正しく再計算できます。

木が遅延タグを子へ流す (`Element::push`) のは、回転の直前と、
挿入・削除・置換の経路、区間操作の途中です。回転や後継との交換のあとに流すと
タグの適用範囲がずれるため、構造を変える前に流すのが要点です。
`push` が既定の no-op なので、遅延タグを使わない要素の挙動は変わりません。
部分木のキー範囲 (`min` / `max`) を各ノードに持たせておき、
`prod` / `apply` は「範囲と交差しない」「完全に含まれる」を判定して
再帰を打ち切ります。

## ソース

`crates/avl-tree/src/lib.rs` の全文です。コードブロック右上のボタンでまるごとコピーできます。
リポジトリのファイルをそのまま埋め込んでいるので、この表示が実装とずれることはありません。

`#[cfg(test)] mod tests;` は提出先では無効になるので、そのまま貼って構いません。
`Indexed` と `LazySegmentTree` を使わない提出では、`lib.rs` の
`mod indexed;` / `pub use indexed::Indexed;` と
`mod lazy_segment_tree;` / `pub use lazy_segment_tree::{Action, LazySegmentTree, Monoid};`
の 4 行を消せば、この `lib.rs` だけで済みます。

```rust,ignore
{{#include ../../../crates/avl-tree/src/lib.rs}}
```

### Indexed も使う

順序統計 (`Indexed` / `slot_by_index` / `index_of`) を使う提出では、
`crates/avl-tree/src/indexed.rs` も必要です。`lib.rs` の `mod indexed;` を次の
ブロックで置き換えてください。真ん中が `indexed.rs` の全文です。
`mod indexed { ... }` で包むと、`indexed.rs` の先頭の `use super::*;` が
`lib.rs` のアイテムを指します。

```rust,ignore
mod indexed {
{{#include ../../../crates/avl-tree/src/indexed.rs}}
}
```

### LazySegmentTree も使う

遅延セグメント木 (`LazySegmentTree` / `Monoid` / `Action` / キー範囲の
`prod` / `apply`) を使う提出では `crates/avl-tree/src/lazy_segment_tree.rs` も
必要です。`lib.rs` の `mod lazy_segment_tree;` を次のブロックで置き換えてください。
真ん中が `lazy_segment_tree.rs` の全文です。

```rust,ignore
mod lazy_segment_tree {
{{#include ../../../crates/avl-tree/src/lazy_segment_tree.rs}}
}
```
