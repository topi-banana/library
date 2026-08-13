# Ranges — 半開区間の集合

互いに重ならない半開区間 `[start, end)` の集合を管理します。
区間を追加・削除するたびに、重なった区間や接した区間は 1 本にまとめられます。

- 実装: [`crates/ranges/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/ranges/src/lib.rs)
- verify: 対応するジャッジ問題が無いため、ユニットテストのみ

`1..3` と `3..5` は端点が接しているので `1..5` にまとまります。
一方 `1..2` と `3..4` は離れているため別々のままです。
整数だからといって「隣り合う値」を繋げることはしません。

更新しながら使う [`Ranges`](#ranges) と、
構築後は読むだけの [`ImmutableRanges`](#immutableranges) の 2 つがあります。

## Ranges

`BTreeMap<T, T>` で `start -> end` を持ちます。要素の型に必要な境界は
更新系が `Ord + Clone`、参照系が `Ord` です。

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `Ranges::new()` | `O(1)` | 空の集合を作る |
| `insert(start..end)` | ならし `O(log n)` | 区間を追加し、重なる区間を統合する |
| `remove(start..end)` | ならし `O(log n)` | 区間を削除する。跨いだ区間は分割される |
| `contains(&v)` | `O(log n)` | `v` がいずれかの区間に含まれるか |
| `covering(&v)` | `O(log n)` | `v` を含む区間を `Option<(&start, &end)>` で返す |
| `contains_range(&r)` | `O(log n)` | `r` 全体が 1 本の区間に覆われるか |
| `overlaps(&r)` | `O(log n)` | `r` がいずれかの区間と少しでも重なるか |
| `len()` / `is_empty()` | `O(1)` | 保持している区間の**本数** (要素数ではない) |
| `iter()` | 1 区間あたり `O(1)` | `start` 昇順に `(&start, &end)` を返す |

`insert` と `remove` は 1 回の呼び出しで複数の区間を巻き込むことがありますが、
巻き込まれた区間はその時点で消えるため、ならし計算量は `O(log n)` です。

`start >= end` の区間は空とみなし、`insert` / `remove` とも何もしません。

### 使用例

```rust
use ranges::Ranges;

let mut r = Ranges::new();
r.insert(1..3);
r.insert(5..10);
r.insert(3..5); // 両隣に接するので 1 本になる
assert_eq!(r.len(), 1);
assert!(r.contains(&7));

r.remove(4..6); // 中央をくり抜くと 2 本に割れる
let got: Vec<_> = r.iter().map(|(s, e)| (*s, *e)).collect();
assert_eq!(got, vec![(1, 4), (6, 10)]);
```

区間の列からまとめて作れます。順不同・重なりありの入力で構いません。

```rust
use ranges::Ranges;

let r: Ranges<i32> = [10..20, 1..5, 3..7].into_iter().collect();
let got: Vec<_> = r.iter().map(|(s, e)| (*s, *e)).collect();
assert_eq!(got, vec![(1, 7), (10, 20)]);
```

半開区間なので `end` は含みません。`covering` は区間そのものを返すため、
「どの区間に入ったか」で分岐したいときに使えます。

```rust
use ranges::Ranges;

let r: Ranges<i32> = [1..5, 10..20].into_iter().collect();
assert!(r.contains(&4));
assert!(!r.contains(&5));
assert_eq!(r.covering(&12), Some((&10, &20)));

assert!(r.contains_range(&(10..20))); // 1 本に収まっている
assert!(!r.contains_range(&(4..11))); // 隙間があるので覆えていない
assert!(r.overlaps(&(4..11))); // 重なってはいる
assert!(!r.overlaps(&(5..10))); // 隙間そのものとは重ならない
```

`Ord` があれば整数以外でも使えます。

```rust
use ranges::Ranges;

let mut r = Ranges::new();
r.insert("a".to_string().."m".to_string());
r.insert("m".to_string().."z".to_string());
assert_eq!(r.len(), 1);
assert!(r.contains(&"q".to_string()));
```

## ImmutableRanges

`Vec<(T, T)>` を `start` 昇順で持ちます。更新はできませんが、
参照系は二分探索で `O(log n)`、要素は連番に並ぶためキャッシュに乗りやすく、
何度も引くだけの用途では `Ranges` より軽く動きます。

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `contains(&v)` | `O(log n)` | `v` がいずれかの区間に含まれるか |
| `covering(&v)` | `O(log n)` | `v` を含む区間を `Option<&(start, end)>` で返す |
| `covering_index(&v)` | `O(log n)` | `v` を含む区間の添字を返す |
| `contains_range(&r)` | `O(log n)` | `r` 全体が 1 本の区間に覆われるか |
| `overlaps(&r)` | `O(log n)` | `r` がいずれかの区間と少しでも重なるか |
| `as_slice()` / `iter()` | `O(1)` | 内部の `&[(start, end)]` をそのまま見る |

`Ranges` からは `From` で移せます。逆向きの変換も用意しています。

```rust
use ranges::{ImmutableRanges, Ranges};

let r: Ranges<i32> = [10..20, 1..5, 3..7].into_iter().collect();
let frozen: ImmutableRanges<_> = r.into();
assert_eq!(frozen.as_slice(), &[(1, 7), (10, 20)]);
assert_eq!(frozen.covering_index(&3), Some(0));
```

区間の列から直接作ることもできます。こちらは
ソートしてから畳み込むだけなので `Clone` を要求しません。
`Ranges` を経由できない型はこちらを使ってください。

```rust
use ranges::ImmutableRanges;

let frozen: ImmutableRanges<i32> = [10..20, 3..7, 1..5, 5..5, 7..10]
    .into_iter()
    .collect();
assert_eq!(frozen.as_slice(), &[(1, 20)]);
```

`covering_index` が添字を返すので、区間と同じ長さの別配列を持たせて
「この区間に紐づく値」を引く、といった使い方ができます。

## 実装メモ

`insert` はまず `..=start` を後ろから 1 つ見て、
左隣の区間が `start` まで届いていれば、そこまで含めた `start` に広げます。
そのうえで `start..=end` に入る区間をすべて取り除き、
取り除いた区間の `end` が伸びていればそちらを採用します。
`range(..=&start)` と `range(&start..=&end)` の境界が両方とも閉じているのは、
端点で接するだけの区間も統合の対象にするためです。

`remove` は左隣の区間を先に処理します。
削除範囲を内包していた場合は 1 本が 2 本に割れるため、
`start` で打ち切った左半分と `end` から始まる右半分の両方を入れ直します。
その後 `start..end` に始点を持つ区間を消し、
はみ出した分だけ `end` から再挿入します。

`ImmutableRanges` の `FromIterator` は `sort_unstable_by` のあと
`dedup_by` で畳み込みます。`dedup_by` は残す側 (`prev`) を書き換えられるので、
`prev.1` に伸びた `end` を書き戻すことで、
大きな区間が後続を次々に飲み込む形にも対応できます。
比較を「直前の入力」ではなく「残っている区間」に対して行うのがポイントで、
`[0..100, 10..20, 30..40]` のような入力が分裂しないのはこのためです。
