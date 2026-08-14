# Permutation — 順列

順列を扱う 2 通りの方法を提供します。
スライスを辞書順で 1 つ進める / 戻す `next_permutation`・`prev_permutation` と、
多重集合から長さ `len` の順列をまとめて作る `permutation` です。

- 実装: [`crates/permutation/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/permutation/src/lib.rs)
- verify: なし (対応するジャッジ問題が無いため、ユニットテストと doctest だけで検証しています)

## API

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `next_permutation(a)` | 最悪 `O(n)`、全列挙では償却 `O(1)` | 辞書順で次に大きい順列へ遷移。最大なら `false` |
| `prev_permutation(a)` | 最悪 `O(n)`、全列挙では償却 `O(1)` | 辞書順で次に小さい順列へ遷移。最小なら `false` |
| `permutation(counts, len)` | 出力の総サイズに比例 (`len` が合計個数を超えるときを除く) | 多重集合から長さ `len` の順列を全列挙 |

前者 2 つは追加のメモリを使わず `a` をその場で書き換えます。
`permutation` は結果をすべて [`Vec`] にまとめて構築するため、
順列の個数分のメモリを消費します。列挙するだけなら `next_permutation` のループを使ってください。

要素の型に必要な境界は、`next_permutation` / `prev_permutation` が `Ord`、
`permutation` が `Ord + Clone` です。

## next_permutation / prev_permutation

C++ の `std::next_permutation` と同じ遷移をしますが、**折り返しません**。
`a` が既に最大の順列 (降順) なら `false` を返し、`a` は変更されません。
そのため「昇順に整列してから `while` を回す」で全列挙になります。

```rust
use permutation::next_permutation;

let mut a = [1, 2, 3];
let mut ps = vec![a.to_vec()];
while next_permutation(&mut a) {
    ps.push(a.to_vec());
}
assert_eq!(ps.len(), 6);
assert_eq!(ps[0], vec![1, 2, 3]);
assert_eq!(ps[5], vec![3, 2, 1]);
```

昇順以外から始めると、その手前の順列を取りこぼします。
全列挙したいときは必ず `a.sort()` してから入ってください。

```rust
use permutation::next_permutation;

// [2, 1, 3] から始めると 3! = 6 通りのうち 4 通りしか回らない。
let mut a = [2, 1, 3];
let mut cnt = 1;
while next_permutation(&mut a) {
    cnt += 1;
}
assert_eq!(cnt, 4);
```

同じ値が複数あっても重複した順列は返しません。
多重集合の順列 (multiset permutation) がそのまま得られます。

```rust
use permutation::next_permutation;

let mut a = [1, 1, 2];
let mut ps = vec![a.to_vec()];
while next_permutation(&mut a) {
    ps.push(a.to_vec());
}
// 3! = 6 ではなく 3!/2! = 3 通り。
assert_eq!(ps, vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]]);
```

`prev_permutation` は逆向きです。降順から始めれば昇順まで遡れます。

```rust
use permutation::prev_permutation;

let mut a = [3, 2, 1];
let mut ps = vec![a.to_vec()];
while prev_permutation(&mut a) {
    ps.push(a.to_vec());
}
assert_eq!(ps.len(), 6);
assert_eq!(ps.last().unwrap(), &vec![1, 2, 3]);
```

長さ 0 と 1 のスライスはどちらも `false` を返します。

## permutation

`(値, 個数)` の並びで多重集合を渡すと、そこから `len` 個を選んで並べた順列を
すべて返します。`counts` は探索中に個数を貸し借りするため `&mut` ですが、
呼び出しが終わった時点で元の個数に戻ります。

```rust
use permutation::permutation;

let mut counts = [(1, 2), (2, 1)];
assert_eq!(
    permutation(&mut counts, 3),
    vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]]
);
// 個数は呼び出し前と同じ。
assert_eq!(counts, [(1, 2), (2, 1)]);
```

`len` が合計個数より小さければ部分順列 (順列 nPr) になります。

```rust
use permutation::permutation;

let mut counts = [('a', 1), ('b', 1), ('c', 1)];
assert_eq!(
    permutation(&mut counts, 2),
    vec![
        vec!['a', 'b'],
        vec!['a', 'c'],
        vec!['b', 'a'],
        vec!['b', 'c'],
        vec!['c', 'a'],
        vec!['c', 'b']
    ]
);
```

境界の挙動は次の通りです。

| 入力 | 返り値 |
| --- | --- |
| `len == 0` | 空の順列 1 つ `[[]]` |
| 合計個数 `< len` | 空の [`Vec`] |
| 個数が `0` の要素 | 使われずに読み飛ばされる |

**合計個数 `< len` は返り値が空になるだけで、計算量は軽くなりません。**
探索は `len` に届かないことを事前に判定せず、合計個数までの並べ方を
すべて辿ってから空を返します。要素 9 個に対して `len = 10` を渡すと、
結果は空でも 100 万ノード近くを探索します。`len` が合計個数を超えるときは
呼び出す前に弾いてください。

### 並び順は `counts` の順に従う

出力は `counts` に書いた**要素の並び順**を基準にした辞書順で、
値の大小順ではありません。辞書順で欲しければ `counts` を値の昇順に並べてください。

```rust
use permutation::permutation;

// counts が降順なので、出力も [2, 1] が先。
let mut counts = [(2, 1), (1, 1)];
assert_eq!(permutation(&mut counts, 2), vec![vec![2, 1], vec![1, 2]]);

let mut sorted = [(1, 1), (2, 1)];
assert_eq!(permutation(&mut sorted, 2), vec![vec![1, 2], vec![2, 1]]);
```

### 同じ値は 1 エントリにまとめる

`counts` の要素は値ごとに一意である前提です。
同じ値を別々のエントリに分けると、それらは区別されるため出力が重複します。

```rust
use permutation::permutation;

// 2 つの `1` を別物として扱うので同じ順列が 2 回出る。
let mut split = [(1, 1), (1, 1)];
assert_eq!(permutation(&mut split, 2), vec![vec![1, 1], vec![1, 1]]);

// 個数をまとめれば 1 通り。
let mut merged = [(1, 2)];
assert_eq!(permutation(&mut merged, 2), vec![vec![1, 1]]);
```

## どちらを使うか

| | `next_permutation` のループ | `permutation` |
| --- | --- | --- |
| 追加メモリ | 定数 | 順列の個数 × `len` |
| 部分順列 (nPr) | 直接は不可 | 可 |
| 途中で打ち切る | 可 | 不可 (全件作ってから返る) |
| 入力の前処理 | 昇順に整列 | 値ごとに個数を集約 |

要素をすべて使い切る順列を最後まで走査するだけなら `next_permutation` が有利です。
`n = 10` でも `10! = 3628800` 通りあり、`permutation` だとこれを全部保持することになります。

`len` 個だけ選んで並べたい、あるいは結果を後から並べ替えたり index で参照したいときは
`permutation` を使ってください。

## 使用例

`n` が小さいときの全探索です。訪問順を全列挙して最小コストを取ります。

```rust
use permutation::next_permutation;

let d = [[0, 3, 1], [3, 0, 2], [1, 2, 0]];
let mut order = [0, 1, 2];
let mut best = usize::MAX;
loop {
    let cost: usize = order.windows(2).map(|w| d[w[0]][w[1]]).sum();
    best = best.min(cost);
    if !next_permutation(&mut order) {
        break;
    }
}
assert_eq!(best, 3);
```

`loop` + 末尾で `break` にしているのは、`while next_permutation(..)` だと
最初の順列 (`[0, 1, 2]`) が評価されないためです。
`next_permutation` は「次へ進めてから `true`」を返すので、
先頭の順列は必ずループの外か本体の先頭で処理します。

## 実装メモ

`next_permutation` は標準的な 3 ステップです。

1. 末尾から見て `a[i] < a[i + 1]` となる最大の `i` を探す (無ければ最大の順列)
2. `i` より後ろで `a[i]` より大きい最後の要素と `a[i]` を交換する
3. `i` より後ろを反転する

ステップ 1 で見つかる `i` より後ろは広義降順 (等しい要素が隣り合ってもよい) に
並んでいるため、ステップ 3 の反転だけで最小の並びになります。整列は不要です。
同じ値があっても順列が重複しないのはこの性質のおかげです。
`prev_permutation` は比較の向きを全て逆にしただけで、構造は同じです。

1 回の呼び出しは最悪 `O(n)` ですが、反転する長さは末尾の広義降順区間の長さに等しく、
全列挙を通して均すと 1 回あたり定数回の操作になります。

`permutation` は DFS です。`counts` の個数を 1 減らして潜り、戻ってきたら 1 足す
という貸し借りで多重集合の状態を持ち回るため、`counts` の内容は呼び出しの前後で一致します。
探索中に持つのは選んだ要素の**添字**の列で、`len` に達した時点で初めて値を clone します。

順列の個数を `m`、`counts` の要素の種数を `k` とすると、各ノードで `counts` を
一通り走査するので探索に `O(m k)`、値の複製に `O(m len)` かかります。
メモリも出力そのもので `O(m len)` です。

`permutation` の型境界は `Ord + Clone` ですが、内部では要素を clone するだけで
比較はしていません。出力順が値の大小ではなく `counts` の並び順で決まるのはこのためです。

[`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
