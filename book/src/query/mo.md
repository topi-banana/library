# Mo — Mo's algorithm

列に対する `q` 個の区間クエリ `[l, r)` を、オフラインでまとめて処理します。
見ている区間を 1 要素ずつ伸縮させながら状態を更新していくので、
「区間の端を 1 要素だけ動かす」操作が軽い問題に使えます。

- 実装: [`crates/mo/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/mo/src/lib.rs)
- verify:
  - Library Checker: [Static Range Inversions Query](https://judge.yosupo.jp/problem/static_range_inversions_query),
    [Static Range Count Distinct](https://judge.yosupo.jp/problem/static_range_count_distinct),
    [Static Range Mode Query](https://judge.yosupo.jp/problem/static_range_mode_query)
  - yukicoder: [No.1471 Sort Queries](https://yukicoder.me/problems/no/1471),
    [No.924 紲星](https://yukicoder.me/problems/no/924)

区間内の異なる値の個数、転倒数、モードなど、
「要素を 1 個足す / 1 個引く」が `O(1)` や `O(log n)` でできて、
かつ区間をマージできない (セグメント木に載らない) 種類の問題が対象です。
すべてのクエリを先に読み切ってから処理する必要があるため、オンラインクエリには使えません。

## アルゴリズム

クエリ `[l, r)` を平面上の点 `(l, r)` とみなし、
その点を巡る順序を決めてから、現在の区間をその順に動かしていきます。
1 クエリあたりの計算量ではなく、**端の移動量の合計**が計算量になります。

このライブラリは巡回順に Hilbert 曲線を使います。
移動量の合計は `O(n √q)` 程度で、`l` をブロックで区切って `r` で整列する
古典的な順序と同じオーダーですが、定数倍が小さくなります。

## API

| 項目 | 説明 |
| --- | --- |
| `Mo::new()` | 空のバッファを作る |
| `Mo::push(l, r)` | 半開区間 `[l, r)` のクエリを積む |
| `Mo::execute(&mut state)` | 積んだクエリを処理し、push した順の `Box<[Ans]>` を返す |
| `MoSol::Ans` | 1 クエリの答え。`Default + Clone` が必要 |
| `MoSol::MAX_INDEX_POW2` | Hilbert 曲線の一辺を `2^MAX_INDEX_POW2` とする |
| `MoSol::add_l(i)` / `add_r(i)` | 区間を左 / 右へ 1 要素広げる。`i` は入る要素の添字 |
| `MoSol::del_l(i)` / `del_r(i)` | 区間を左 / 右から 1 要素狭める。`i` は出る要素の添字 |
| `MoSol::solve()` | 現在の区間に対する答えを返す |

4 つのコールバックが受け取るのは、いずれも**出入りする要素そのものの添字**であって、
移動後の区間の端ではありません。
`add_l` と `del_l` には左端に入る / 左端から出る要素の添字が、
`add_r` と `del_r` には右端に入る / 右端から出る要素の添字が渡ります。

## 使用例

区間内の異なる値の個数を求めます。

```rust
use mo::{Mo, MoSol};

/// 区間に含まれる異なる値の個数。
struct Distinct {
    a: Vec<usize>,
    cnt: Vec<usize>,
    distinct: usize,
}

impl Distinct {
    fn add(&mut self, i: usize) {
        if self.cnt[self.a[i]] == 0 {
            self.distinct += 1;
        }
        self.cnt[self.a[i]] += 1;
    }
    fn del(&mut self, i: usize) {
        self.cnt[self.a[i]] -= 1;
        if self.cnt[self.a[i]] == 0 {
            self.distinct -= 1;
        }
    }
}

impl MoSol for Distinct {
    type Ans = usize;
    // 添字は 0..=5 の範囲に収まる。2^3 = 8 > 5。
    const MAX_INDEX_POW2: usize = 3;
    fn add_l(&mut self, l_idx: usize) {
        self.add(l_idx);
    }
    fn add_r(&mut self, r_idx: usize) {
        self.add(r_idx);
    }
    fn del_l(&mut self, l_idx: usize) {
        self.del(l_idx);
    }
    fn del_r(&mut self, r_idx: usize) {
        self.del(r_idx);
    }
    fn solve(&mut self) -> Self::Ans {
        self.distinct
    }
}

let a = vec![1, 2, 1, 3, 2];
// 空区間 [0, 0) を表す状態から始める。
let mut state = Distinct {
    cnt: vec![0; 4],
    a,
    distinct: 0,
};

let mut mo = Mo::new();
mo.push(0, 3); // [1, 2, 1]    -> 2
mo.push(1, 5); // [2, 1, 3, 2] -> 3
mo.push(0, 1); // [1]          -> 1

assert_eq!(*mo.execute(&mut state), [2, 3, 1]);
```

`add_l` と `add_r` (および `del_l` と `del_r`) が同じ処理になるのは、
答えが要素の集合だけで決まる場合です。
転倒数のように左右で更新式が変わる問題では、それぞれ別の処理を書きます。

## 注意点

**渡す状態は空区間 `[0, 0)` を表していること。**
`execute` は現在の区間を `[0, 0)` から動かし始め、途中でリセットしません。

**`MAX_INDEX_POW2` は `2^MAX_INDEX_POW2 > n` を満たす最小の値。**
`r` は列の長さ `n` そのものになりうるので、`n` 未満では足りません。
`n = 10^5` なら `17` (`2^17 = 131072`) です。
範囲外の添字を渡してもパニックはしませんが、
並べ替えの順序が壊れて高速化の効果を失います。

**答えは push した順に並びます。**
内部では Hilbert 順に並べ替えて処理しますが、
`push` した時点の添字を覚えているため、返る列は push 順です。

**`l <= r` であること。** 逆転した区間は検査していません。

## verify

Library Checker 3 問と yukicoder 2 問で検証しています。

### Library Checker: Static Range Inversions Query

[Static Range Inversions Query](https://judge.yosupo.jp/problem/static_range_inversions_query)
は区間の転倒数を答える問題です。
5 問のうち、`add_l` と `add_r` の処理が変わるのはこの問題だけで、
トレイトの左右の非対称性を実際に踏むのはここになります。
左端に入る要素が作る転倒は「区間内の自分より小さい要素」との組、
右端に入る要素が作る転倒は「区間内の自分より大きい要素」との組です。
個数を Fenwick tree に載せ、どちらも `O(log n)` で数えています。

`del_l` / `del_r` では先に Fenwick tree から取り除いてから打ち消します。
順序を逆にすると、取り除く要素自身を数えてしまいます。

実際の verify コードは
[`verify/src/bin/library-checker-static-range-inversions-query.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/library-checker-static-range-inversions-query.rs)
です。

### Library Checker: Static Range Count Distinct

[Static Range Count Distinct](https://judge.yosupo.jp/problem/static_range_count_distinct)
は区間に現れる相異なる値の個数を答える問題で、上の[使用例](#使用例)そのものです。

`N, Q <= 5 * 10^5` と 5 問の中では最も大きく、端の移動は 3 * 10^8 回を超えます。
1 回あたりの処理は「順位を引いて個数を 1 増減する」だけなので、
実行時間はほぼメモリアクセスで決まります。
順位と個数を `u32` で持ち、内側のループが触る配列を小さくしています。

`l == r` の空クエリが来るのもこの問題だけです。
`execute` は区間を広げてから縮めるので、空区間を経由しても壊れません。

実際の verify コードは
[`verify/src/bin/library-checker-static-range-count-distinct.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/library-checker-static-range-count-distinct.rs)
です。

### Library Checker: Static Range Mode Query

[Static Range Mode Query](https://judge.yosupo.jp/problem/static_range_mode_query)
は区間の最頻値とその出現回数を答える問題です。

「個数の最大値だけを覚えておく」実装は誤りになります。
最大個数を持つ値が複数あるとき、そのうち 1 つを `del` で減らしても最大値は変わらないため、
覚えていた最頻値だけが古くなるからです。
そこで順位を個数の昇順に並べた配列を持ち、その末尾を最頻値としています。
同じ個数の順位は配列上で連続した 1 ブロックを占めるので、
`add` / `del` は「ブロックの端と入れ替えて境界を 1 ずらす」だけで済み、`O(1)` です。

実際の verify コードは
[`verify/src/bin/library-checker-static-range-mode-query.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/library-checker-static-range-mode-query.rs)
です。

### yukicoder No.1471 Sort Queries

[No.1471 Sort Queries](https://yukicoder.me/problems/no/1471) は、
部分文字列 `S[L..R]` を並べ替えた辞書順最小の文字列の `X` 文字目を答える問題です。
辞書順最小は「`a` から順に個数だけ並べた文字列」なので、
区間内の各文字の個数がわかれば `X` 文字目は決まります。

ただし `solve` にはクエリ固有の値 (この問題の `X`) を渡せません。
処理順は Hilbert 順に並べ替えられるため、状態の側から
「今どのクエリを処理しているか」を知る方法もありません。
そこで `Ans` を個数の表 `[usize; 26]` にして `solve` では状態をそのまま返し、
`execute` が返した答えを push 順のクエリと突き合わせてから `X` 文字目を求めています。
クエリごとのパラメータが必要な問題は、この形に落とすのが定石です。

実際の verify コードは
[`verify/src/bin/yukicoder-1471.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-1471.rs)
です。

### yukicoder No.924 紲星

[No.924 紲星](https://yukicoder.me/problems/no/924) は、
区間に対して `f(x) = Σ|x - A_k|` の最小値を答える問題です。
`f` は `x` が区間の中央値のとき最小になるので、
中央値と「中央値までの要素の和」がわかれば答えが求まります。

`N, Q <= 2 * 10^5` と大きく、端の移動は `O(n √q)` 回、
つまり 10^8 のオーダーで起こります。
そのため値を座標圧縮したうえで `√m` 個ずつのバケットに分けて個数と総和を持ち、
`add` / `del` を `O(1)`、`solve` の中央値探索を `O(√m)` にしています。
個数を BIT で持つと `add` / `del` が `O(log n)` になり、この呼び出し回数では
定数倍が厳しくなります。**回数が多いのは `solve` ではなく端の移動の方**という
Mo の計算量の形が、そのまま実装の選択に効いてくる例です。

実際の verify コードは
[`verify/src/bin/yukicoder-924.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-924.rs)
です。

## 実装メモ

`execute` は区間を動かすとき、**広げる操作を先に、縮める操作を後に**行います。

```rust,ignore
while nl > l { /* add_l */ }
while nr < r { /* add_r */ }
while nl < l { /* del_l */ }
while nr > r { /* del_r */ }
```

先に縮めると、現在の区間と次の区間が離れている場合に
途中で `nl > nr` (反転した区間) が発生します。
広げてから縮める順にすると、途中の区間は常に
「現在の区間と次の区間の両方を含む区間」になり、反転しません。

`hilbert_order` は `2^pow` 四方の正方形を 4 分割し、
点がどのマス (`seg`) にあるかを求めて再帰します。
各マスの中では曲線の向きが変わるため、`rotate` で回転を持ち回り、
入り口と出口が繋がるように `seg` が `1` か `2` のときとそれ以外で
マス内の順序を反転させています。

並べ替えには `sort_by_cached_key` を使っています。
`hilbert_order` は再帰呼び出しで `O(MAX_INDEX_POW2)` かかるため、
比較のたびに再計算しないようにするためです。
