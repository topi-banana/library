# Rle — 連長圧縮

列を「同じ値が連続する区間」ごとに `(値, 連続する個数)` へまとめます。
元の列の長さを `n` として `O(n)` 時間で、追加のメモリは出力を除いて定数です。

- 実装: [`crates/rle/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/rle/src/lib.rs)
- verify: [yukicoder No.1469 programing](https://yukicoder.me/problems/no/1469)

`aaabbc` は `[('a', 3), ('b', 2), ('c', 1)]` になります。
同じ値が離れた場所に現れる場合は別々の区間として数えるため、
`abbaaa` は `[('a', 1), ('b', 2), ('a', 3)]` です。

## API

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `rle(iter)` | `O(n)` | イテレータを最後まで消費し、`Vec<(値, 個数)>` を返す |
| `.rle()` (`Rle` トレイト) | `O(1)` | 連長圧縮する [`RleIter`](#rleiter) を返す。元のイテレータをこの時点で 1 要素読む |
| `RleIter::next()` | 返す区間の長さに比例。最後まで回すと合計 `O(n)` | 次の `(値, 個数)` を返す。元が尽きたら `None` |

`Rle` は [`Iterator`] を実装するすべての型に対して実装されているので、
`use rle::Rle;` すれば任意のイテレータに `.rle()` が生えます。

要素の型に必要な境界は `PartialEq` だけです。`Eq` は要求しません。

## 使用例

一括で `Vec` が欲しいときは関数版を使います。

```rust
let runs = rle::rle(&mut "aaabbc".chars());
assert_eq!(runs, vec![('a', 3), ('b', 2), ('c', 1)]);
```

イテレータのまま繋ぎたいときはアダプタ版を使います。

```rust
use rle::Rle;

// 最長の連続区間の長さ
let longest = "aabbbbc".chars().rle().map(|(_, cnt)| cnt).max();
assert_eq!(longest, Some(4));
```

## RleIter

`Rle::rle` が返すイテレータです。1 要素を返すのに必要な分だけ元のイテレータを
進めるため、無限列に対しても使えます。

```rust
use rle::Rle;

let heads: Vec<_> = (0..).map(|i| i / 3).rle().take(2).collect();
assert_eq!(heads, vec![(0, 3), (1, 3)]);
```

## verify

[yukicoder No.1469 programing](https://yukicoder.me/problems/no/1469) で検証しています。
「隣り合う 2 文字が同じならその 1 つを消す」を可能な限り繰り返す問題で、
これは各区間の値を 1 つずつ並べたものに一致します。

```rust
use rle::Rle;

let ans: String = "programming".chars().rle().map(|(c, _)| c).collect();
assert_eq!(ans, "programing");
```

実際の verify コードは
[`verify/src/bin/yukicoder-1469.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-1469.rs)
です。`|S| <= 5 * 10^6` と入力が大きいため、`chars` ではなく `bytes` で走査しています。

## 実装メモ

どちらの実装も、直前に見た値 `pre` とその連続数 `cnt` だけを持ち、
値が切り替わった瞬間に区間を確定させます。
最後の区間はループを抜けたあとに確定させる必要があるため、
`rle` では `res.push((pre, cnt))` が、`RleIter` では `for` を抜けた先の
`Some((pre, ...))` がその役割を担います。

`RleIter::next` は先頭で `self.pre.take()?` を呼びます。
元のイテレータが尽きて最後の区間を返した時点で `pre` は `None` になっているため、
以降は何度呼んでも `None` が返ります。

`Rle::rle` は返り値を作る時点で元のイテレータを 1 要素読みます。
純粋なイテレータでは観測できませんが、副作用を持つイテレータに対しては
最初の `next()` を呼ぶ前に 1 要素進む点に注意してください。

[`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
