# Segtree — セグメント木

モノイドの列に対して、1 点更新と区間積の取得をどちらも `O(log n)` で行います。

- 実装: [`crates/segtree/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/segtree/src/lib.rs)
- verify: [point_add_range_sum](https://judge.yosupo.jp/problem/point_add_range_sum)

## モノイドの定義

`Monoid` トレイトを実装した型を型引数に渡します。

```rust
pub trait Monoid {
    type S: Clone;
    fn identity() -> Self::S;
    fn binary_operation(a: &Self::S, b: &Self::S) -> Self::S;
}
```

`binary_operation` は結合律を満たす必要があります。可換である必要はありません。
`identity` が返す値は `binary_operation` の単位元でなければなりません。

加法のモノイド `Additive<T>` は同梱しています。単位元には `T::default()` を使います。

```rust
use segtree::{Additive, Segtree};

let mut seg: Segtree<Additive<i64>> = vec![1, 2, 3, 4, 5].into();
assert_eq!(seg.prod(1..4), 9);

seg.set(2, 10);
assert_eq!(seg.prod(1..4), 16);
assert_eq!(seg.all_prod(), 22);
```

独自のモノイドを使う場合は次のように書きます。

```rust
use segtree::{Monoid, Segtree};

enum Max {}

impl Monoid for Max {
    type S = i64;
    fn identity() -> i64 { i64::MIN }
    fn binary_operation(a: &i64, b: &i64) -> i64 { (*a).max(*b) }
}

let seg: Segtree<Max> = vec![3, 1, 4, 1, 5].into();
assert_eq!(seg.prod(1..4), 4);
```

## API

| メソッド | 計算量 | 説明 |
| --- | --- | --- |
| `Segtree::new(n)` | `O(n)` | 長さ `n`、全要素が単位元 |
| `Segtree::from(vec)` | `O(n)` | 列から構築する |
| `set(p, x)` | `O(log n)` | `p` 番目を `x` に書き換える |
| `get(p)` | `O(1)` | `p` 番目の要素 |
| `prod(range)` | `O(log n)` | `range` の総積。空区間なら単位元 |
| `all_prod()` | `O(1)` | 列全体の総積 |

`prod` は [`RangeBounds<usize>`](https://doc.rust-lang.org/std/ops/trait.RangeBounds.html)
を受け取るため、`l..r` `l..=r` `..r` `..` のいずれも渡せます。

## 実装メモ

葉の数を 2 の冪 `size` に切り上げ、`data[1]` を根とする 1-indexed の完全二分木に格納しています。
`p` 番目の葉は `data[size + p]` に対応し、節点 `k` の子は `2k` と `2k + 1` です。

`prod` は左右から同時に区間を詰めていきます。
非可換なモノイドでも正しく動くよう、左からの積 `sml` と右からの積 `smr` を別々に持ち、
最後に `binary_operation(&sml, &smr)` の順で合成しています。
