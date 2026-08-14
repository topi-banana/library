# Mo — Mo's algorithm

列に対する `q` 個の区間クエリ `[l, r)` を、オフラインでまとめて処理します。
見ている区間を 1 要素ずつ伸縮させながら状態を更新していくので、
「区間の端を 1 要素だけ動かす」操作が軽い問題に使えます。

- 実装: [`crates/mo/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/mo/src/lib.rs)
- verify: なし (doctest のみ)

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

対応するジャッジ問題での verify はまだありません。
crate ドキュメントの doctest が回帰テストを兼ねています。

verify を追加するなら
[Library Checker: Static Range Inversions Query](https://judge.yosupo.jp/problem/static_range_inversions_query)
が定番です。その際はルートの `Cargo.toml` の `[workspace.dependencies]` と
`verify/Cargo.toml` の `[dependencies]` に `mo` を足す必要があります
(現状はどちらにも入っていません)。
手順は [verify を書く](../dev/verify.md) を参照してください。

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
