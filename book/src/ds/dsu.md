# Dsu — 素集合データ構造

無向グラフの連結成分を管理します。経路圧縮と union by size を併用しているため、
1 操作あたりならし `O(α(n))` (α は逆アッカーマン関数) で動作します。

- 実装: [`crates/dsu/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/dsu/src/lib.rs)
- verify: [unionfind](https://judge.yosupo.jp/problem/unionfind)

## API

| メソッド | 計算量 | 説明 |
| --- | --- | --- |
| `Dsu::new(n)` | `O(n)` | `n` 頂点、辺なしで初期化する |
| `leader(a)` | ならし `O(α(n))` | `a` の属する成分の代表元 |
| `merge(a, b)` | ならし `O(α(n))` | `a` と `b` を連結し、併合後の代表元を返す |
| `same(a, b)` | ならし `O(α(n))` | 同じ成分に属するか |
| `size(a)` | ならし `O(α(n))` | `a` の属する成分の大きさ |
| `groups()` | `O(n)` | 成分ごとに頂点番号を昇順で並べたリスト |

`leader` は経路圧縮のため `&mut self` を取ります。`same` や `size` も同様です。

## 使用例

```rust
use dsu::Dsu;

let mut dsu = Dsu::new(4);
dsu.merge(0, 1);
dsu.merge(1, 2);

assert!(dsu.same(0, 2));
assert!(!dsu.same(0, 3));
assert_eq!(dsu.size(0), 3);
assert_eq!(dsu.groups(), vec![vec![0, 1, 2], vec![3]]);
```

## 実装メモ

内部状態は `parent_or_size: Vec<isize>` の 1 本だけです。
値が負のときその頂点は代表元で、絶対値が成分の大きさを表します。
非負のときは親の頂点番号です。

`leader` は再帰ではなくループで実装しています。
競技プログラミングでは `n` が 10<sup>6</sup> 規模になることがあり、
経路圧縮前の木が一直線に伸びた状態で再帰するとスタックが溢れる可能性があるためです。
