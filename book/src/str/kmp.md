# KMP — Knuth-Morris-Pratt 法

パターンがテキストのどこに現れるかを、すべて列挙します。
パターンの長さを `m`、テキストの長さを `n` として、
前処理 `O(m)` 時間・メモリ、検索は最後まで列挙して `O(n)` 時間です。
素朴な突き合わせの `O(nm)` と違い、テキスト側の添字は一度も巻き戻りません。

- 実装: [`crates/kmp/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/kmp/src/lib.rs)
- verify: [yukicoder No.430 文字列検索](https://yukicoder.me/problems/no/430), [yukicoder No.2298 yukicounter](https://yukicoder.me/problems/no/2298)

「文字列」と名前が付いていますが、扱うのは `&[T]` です。
要素の型に必要な境界は `Eq` だけなので、`&[u8]` や `&[char]` のほか、
数値の列や自作の型の列にもそのまま使えます。
`&str` を検索するときは `s.as_bytes()` でバイト列として渡してください。

## API

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `KMP::new(pattern)` | 時間・メモリとも `O(m)` | パターンから LPS 配列を作り、検索器にする |
| `search(text)` | `O(1)` | テキストを借りて [`KMPIter`](#kmpiter) を返す。この時点ではまだ走査しない |
| `KMPIter::next()` | 次の出現まで進んだ分。最後まで回すと合計 `O(n)` | 次の出現の開始位置を昇順に返す |

前処理はパターンだけに依存するので、1 つの `KMP` を複数のテキストに使い回せます。

```rust
use kmp::KMP;

let kmp = KMP::new(b"ab");
assert_eq!(kmp.search(b"abcab").collect::<Vec<_>>(), vec![0, 3]);
assert_eq!(kmp.search(b"ba").collect::<Vec<_>>(), vec![]);
```

`KMP<'pattern, T>` はパターンを借用したまま持ちます。
一時変数を直接渡すと、パターンのほうが先に破棄されてコンパイルが通りません。

```rust,ignore
// error[E0716]: temporary value dropped while borrowed
let kmp = KMP::new(&vec![1, 2, 1]);
let hits: Vec<_> = kmp.search(&[1, 2, 1, 2, 1]).collect();
```

```rust
use kmp::KMP;

let pattern = vec![1, 2, 1];
let kmp = KMP::new(&pattern);
assert_eq!(kmp.search(&[1, 2, 1, 2, 1]).collect::<Vec<_>>(), vec![0, 2]);
```

## 出現の数え方

重なり合う出現も、それぞれ別の出現として数えます。
`aa` は `aaaa` の位置 0, 1, 2 に現れます。

```rust
use kmp::KMP;

let kmp = KMP::new(b"aa");
assert_eq!(kmp.search(b"aaaa").collect::<Vec<_>>(), vec![0, 1, 2]);
```

空のパターンは「どこにでも現れる」とは扱わず、1 つも位置を返しません。
標準ライブラリの [`str::matches`] は空パターンに対して `n + 1` 個返すので、
そのつもりで数えると食い違います。
パターンがテキストより長いときも、当然 1 つも返しません。

```rust
use kmp::KMP;

assert_eq!(KMP::new(b"").search(b"abc").count(), 0);
assert_eq!(KMP::new(b"abcde").search(b"ab").count(), 0);
```

## KMPIter

`search` が返すイテレータです。次の出現位置が確定するまでしかテキストを進めないため、
「最初の 1 つだけ欲しい」用途ではテキスト全体を見ずに済みます。

```rust
use kmp::KMP;

let kmp = KMP::new(b"ab");
assert_eq!(kmp.search(b"xxabxxab").next(), Some(2));
```

出現回数だけが欲しいなら `count()`、位置の一覧が欲しいなら `collect()` を使います。
`KMPIter` が借りているのは `KMP` とテキストの両方なので、
検索中にどちらも書き換えられません。

## verify

yukicoder 2 問で検証しています。

### yukicoder No.430 文字列検索

[No.430 文字列検索](https://yukicoder.me/problems/no/430) は、
`M` 個のパターン `C_i` が `S` に現れる回数の総和を答える問題です。
`C_i` ごとに `KMP` を作り直して `S` 全体を検索し、`count()` を足し合わせます。

```rust
use kmp::KMP;

let s = b"ABCDABCD";
let c: [&[u8]; 3] = [b"A", b"DA", b"ABCDABCD"];
let ans: usize = c.iter().map(|p| KMP::new(p).search(s).count()).sum();
assert_eq!(ans, 4);
```

サンプル 3 が `S = "AAAA"`, `C = {A, AA, AAA, AAAA}` に対して 10 を要求するので、
[重なり合う出現をそれぞれ数える](#出現の数え方)ことがそのまま問われます。
`|S| <= 5 * 10^4`, `M <= 5000` なので全体では 2.5 * 10^8 歩ほど走りますが、
S 側の添字が巻き戻らないぶん、素朴な突き合わせと違って本数分の線形時間で済みます。

実際の verify コードは
[`verify/src/bin/yukicoder-430.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-430.rs)
です。

### yukicoder No.2298 yukicounter

[No.2298 yukicounter](https://yukicoder.me/problems/no/2298) は、
`yukicoder` を `K` 回繰り返した文字列が `S` の部分文字列となる最大の `K` を答える問題です。

`yukicoder` は 9 文字すべてが相異なるので LPS は全て 0 になり、出現同士が重なることはありません。
そのため隣り合う出現位置の差がちょうど 9 であることと、
その 2 つが隙間なく繋がっていることが同値になります。
`search` が返す位置は昇順なので、差が 9 で繋がった並びの最長を数えれば `K` が求まります。

```rust
use kmp::KMP;

let hits: Vec<_> = KMP::new(b"yukicoder").search(b"yukicoderaayukicoderyukicoder").collect();
// 0 と 11 は離れているが、11 と 20 は差が 9 なので繋がっている。
assert_eq!(hits, vec![0, 11, 20]);
```

`|S| <= 10^6` と大きいため、位置を `Vec` に集めず [`KMPIter`](#kmpiter) のまま流し読みしています。
1 つも出現しない場合の `K = 0` は、空文字列が任意の `S` の部分文字列であることに対応します。

実際の verify コードは
[`verify/src/bin/yukicoder-2298.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-2298.rs)
です。

## 実装メモ

前処理で作るのは LPS 配列 (Longest Prefix Suffix) です。
`lps[i]` は `pattern[..=i]` の接頭辞と接尾辞が一致する最大の長さで、
ただし全体そのものは数えません。`aabaaab` なら `[0, 1, 0, 1, 2, 2, 3]` です。

照合が途中で失敗したとき、テキスト側を巻き戻す代わりに、
パターン側の添字を `lps[pattern_index - 1]` へ戻します。
そこまでは既に一致していることが LPS の定義から分かっているので、
比較をやり直す必要がありません。
`pattern_index` が 0 まで戻っても一致しない場合だけ、テキスト側を 1 つ進めます。

走査が `O(n)` になるのはならし計算量の議論です。
`text_index` は単調増加で高々 `n` 回しか進まず、
`pattern_index` は 1 回の一致につき 1 しか増えないため、
失敗時に減る量の合計も高々 `n` に収まります。

パターン全体が一致したときも `pattern_index` を `lps[m - 1]` に戻します。
0 に戻さないのは、直前の一致の接尾辞から続く出現を拾うためで、
重なり合う出現が列挙されるのはこの 1 行によります。

`KMP::new` の境界は `T: Eq` です。
`Iterator` の実装自体は `T: PartialEq` しか要求していませんが、
`KMPIter` は `search` を通してしか作れないため、実質 `Eq` が必要になります。

[`str::matches`]: https://doc.rust-lang.org/std/primitive.str.html#method.matches
