# DivConquer — 平方分割

列を長さ `N` のブロックに区切り、ブロックごとの事前計算結果 (キャッシュ) を持っておきます。
区間 `[l, r)` のクエリは「両端の半端な部分は生の要素から」
「間に挟まる完全なブロックはキャッシュから」の 2 通りに分けて解き、
部分結果をマージして答えます。

- 実装: [`crates/div_conquer/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/div_conquer/src/lib.rs) — [全文はこのページの末尾](#ソース)
- verify:
  - Library Checker: [Static Range Frequency](https://judge.yosupo.jp/problem/static_range_frequency)
  - yukicoder: [No.1332 Range Nearest Query](https://yukicoder.me/problems/no/1332),
    [No.3078 Difference Sum Query](https://yukicoder.me/problems/no/3078)

crate 名は `div_conquer` ですが、扱っているのは分割統治ではなく**平方分割 (ブロック分割)** です。

区間内の特定の値の個数、区間内の値と `x` の距離の和、区間内で `x` に最も近い値など、
**クエリごとに引数 `x` が付いてくる**ために区間をマージできず、
セグメント木に載らない種類の問い合わせが対象です。
ブロック内で整列や累積和を作っておけば、そのブロック分の答えは二分探索で取り出せます。

[Mo's algorithm](./mo.md) と同じく「区間クエリを `√` のオーダーで捌く」道具ですが、
性質は逆です。Mo はクエリを全部読んでから並べ替えるオフライン専用で、
その代わり状態を差分更新できれば何でも扱えます。
こちらはクエリを溜めずにオンラインで 1 問ずつ答えられますが、
その代わり「ブロック単位で答えを作れる」ことを要求します。
列の更新にも対応していて、書き換えたブロックのキャッシュだけを張り直します。

## アルゴリズム

長さ `n` の列を先頭から `N` 要素ずつのブロックに区切ります。
区間 `[l, r)` は、ブロック境界で 3 つに分かれます。

```text
+-----------+-----------+-----------+-----------+
|  block 0  |  block 1  |  block 2  |  block 3  |
+-----------+-----------+-----------+-----------+
       l                                  r
       |<-->|<--------------------->|<--->|
         (1)           (2)            (3)
```

- (1) 左の半端な部分 — 生の要素を `resolver` で解く
- (2) 完全に含まれるブロック — キャッシュを `cache_resolver` で解く
- (3) 右の半端な部分 — 生の要素を `resolver` で解く

半端な部分は高々 `2N` 要素なので、生の要素をそのまま走査します。
間の完全なブロックは高々 `n / N` 個で、それぞれキャッシュから答えを取り出します。
`resolver` の 1 要素あたりを `O(f)`、`cache_resolver` 1 回を `O(g)` として、
1 クエリは `O(N · f + n / N · g)` 時間です。

`N = √n` と取れば `O(√n)` になります。
ただし実測ではもっと大きく取った方が速いことが多く、その理由は次の通りです。

### ブロック長の決め方

半端な部分の走査は 1 要素あたり比較や加減算が数回で、
連続したメモリを舐めるだけなのでベクトル化も効きます。
一方ブロックごとの二分探索は、1 段ごとに次に読む場所が決まる**依存した読み出し**で、
段ごとにキャッシュミスしうるため 1 回が重くなります。
`O` の中では対等でも定数倍が 1 桁以上違うので、
走査する要素を増やして二分探索の回数を減らす方に倒します。

このライブラリで verify した 3 問では、`n` の上限が `10^5` / `3 * 10^5` / `5 * 10^5` に対して
ブロック長を `2048` / `4096` / `8192` に取っています。
`√n` は `320` から `710` 程度なので、6 倍から 12 倍にあたります。
問題ごとに `resolver` と `cache_resolver` の重さの比が違うため、
最終的には提出して測るのが早いです。

## API

列を持って更新する [`DivConquer`](#divconquer) と、
そこから借りてクエリを解く [`ImmutableDivConquer`](#immutabledivconquer) の 2 つに分かれています。
列を書き換えたら `into_immut` でキャッシュを張り直し、返ってきた側に `resolve` を投げます。

ブロック長 `N` は const generics で、要素・キャッシュ・答え・引数の型は型引数で受け取ります。
処理の中身は 4 つの関数ポインタで注入します。

以下、列の長さを `n`、ブロック数を `b = ⌈n / N⌉` と書きます。
計算量は注入した関数の**呼び出し回数**で示します。実際の時間はこれに 1 回の重さを掛けたものです。

### DivConquer

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `DivConquer::<N, _, _, _, _>::new(v, cacher, resolver, cache_resolver, merger)` | `cacher` を `b` 回 | 列 `v` を `N` ごとに区切り、ブロックごとのキャッシュを作る |
| `push(element)` | ならし `O(1)` | 末尾に 1 要素足す。触れたブロックに印を付けるだけでキャッシュは作り直さない |
| `pop()` | `O(1)` | 末尾の 1 要素を取り出して `Option<E>` で返す。空なら `None` |
| `set(index, element)` | `O(1)` | `index` 番目を書き換える。印を付けるだけでキャッシュは作り直さない |
| `into_immut()` | `O(b)` + `cacher` を「印の付いたブロック数」回 | 印の付いたブロックだけキャッシュを張り直し、[`ImmutableDivConquer`](#immutabledivconquer) を返す |

更新は印 (dirty フラグ) を立てるだけで、キャッシュの再計算は `into_immut` まで遅延します。
1 点更新の直後に `into_immut` を呼ぶ形でも、張り直すのは 1 ブロックだけです。
`push` を `k` 回続けてから 1 回呼ぶ形なら、跨いだブロックの分だけで済みます。

ただし印を調べる走査は毎回全ブロックを見るので、
何も更新していなくても 1 回につき `O(b)` かかります。
これは `resolve` がキャッシュを引く回数と同じオーダーなので、
更新とクエリを交互に回しても全体の見積もりは変わりませんが、
「更新していないから `into_immut` は無料」ではない点に注意してください。

### ImmutableDivConquer

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `ImmutableDivConquer::<N, _, _, _, _>::new(slice, cache, cacher, resolver, cache_resolver, merger)` | `cacher` を `b` 回 | 借りた `slice` のキャッシュを、渡された `cache` に作る |
| `resolve(range, &arg)` | `resolver` を高々 2 回 + `cache_resolver` を高々 `b` 回 + `merger` を高々 `b + 1` 回 | 区間クエリを解く。`range` は `RangeBounds<usize>` なら何でもよい |

`resolver` に渡る列は 1 回あたり `N` 要素未満です。
`resolver` の 1 要素あたりを `O(f)`、`cache_resolver` 1 回を `O(g)` とすれば
1 クエリは `O(N · f + n / N · g)` 時間で、`N = √n` と取れば `O(√n)` になります。

`ImmutableDivConquer::new` はキャッシュの置き場所 `&mut Vec<C>` を外から受け取ります。
`DivConquer` を経由せず、既にある `&[E]` から直接クエリだけを投げたいときに使います。
`cache` が空でないとパニックします。

### コールバック

| 項目 | 型 | 説明 |
| --- | --- | --- |
| `cacher` | `fn(&[E]) -> C` | ブロックの要素から事前計算結果を作る |
| `resolver` | `fn(&[E], &A) -> R` | 生の要素の列と引数からクエリを解く |
| `cache_resolver` | `fn(&C, &A) -> R` | キャッシュと引数からクエリを解く |
| `merger` | `fn(R, R) -> R` | 2 つの部分結果をマージする |

4 つとも `Fn` トレイトではなく**関数ポインタ**で受け取ります。
`fn` で定義した関数と、何も捕捉しないクロージャはそのまま渡せますが、
捕捉するクロージャは渡せません。
ブロックの外にある表 (座標圧縮の対応表など) を引きたい場合は、
クエリごとの引数 `Arg` に載せて `resolve` から渡してください。

型引数の名前 `Result` は答えの型という意味で、`std::result::Result` とは関係ありません。

## 使用例

区間 `[l, r)` に含まれる `x` の個数を数えます。
ブロックごとに値を整列しておけば、完全なブロックは二分探索で数えられます。

```rust
use div_conquer::DivConquer;

/// ブロックの要素を昇順に並べたもの。
type Cache = Vec<u32>;

fn cacher(block: &[u32]) -> Cache {
    let mut sorted = block.to_vec();
    sorted.sort_unstable();
    sorted
}

/// 半端な部分は生の要素をそのまま走査する。
fn resolver(block: &[u32], &x: &u32) -> usize {
    block.iter().filter(|&&v| v == x).count()
}

/// 完全なブロックは整列済みなので、上界と下界の差で個数が求まる。
fn cache_resolver(sorted: &Cache, &x: &u32) -> usize {
    sorted.partition_point(|&v| v <= x) - sorted.partition_point(|&v| v < x)
}

/// 個数の和。`usize::default()` は `0` で、これは加算の単位元。
fn merger(a: usize, b: usize) -> usize {
    a + b
}

let a = vec![1, 2, 1, 3, 2, 1, 2, 1];
// 最初の型引数がブロック長。実際の問題では `√n` より大きめに取る。
let mut dc = DivConquer::<4, _, _, _, _>::new(a, cacher, resolver, cache_resolver, merger);

// クエリを投げるのは `into_immut` で受け取った側。
let im = dc.into_immut();
assert_eq!(im.resolve(0..8, &1), 4);
assert_eq!(im.resolve(1..6, &2), 2); // 半端な部分だけで解く
assert_eq!(im.resolve(2..3, &1), 1); // 1 ブロックに収まる区間
assert_eq!(im.resolve(.., &2), 3); // `RangeBounds` なら何でも渡せる

// 更新して、もう一度キャッシュを確定させてから引く。
dc.set(0, 2);
dc.push(2);
let im = dc.into_immut();
assert_eq!(im.resolve(.., &2), 5);
```

`into_immut` は `&mut self` を借りるので、返ってきた `im` が生きている間は
`dc` を更新できません。更新とクエリを交互にする場合は、上のように
`im` を使い終えてから次の更新に進みます。

`resolver` と `cache_resolver` は同じ問いに答える 2 つの実装です。
どちらも「与えられた範囲に `x` が何個あるか」を返し、
前者は生の列を走査し、後者は整列済みのキャッシュを二分探索します。
このペアが食い違っていると、区間の位置によって答えが変わる厄介なバグになります。
ブロック長を小さくした乱数テストで突き合わせておくと安全です。

## 注意点

**`Result::default()` は `merger` の単位元であること。**
`resolve` は `Result::default()` から部分結果を左から順に畳み込みます (`merger` は結合的であること)。
和や個数のように単位元が `0` の演算はそのままで構いませんが、
最小値のように `0` が単位元でない演算では答えを間違えます。
その場合は型を新しく作って `Default` を与えます。

```rust
/// 距離の最小値。単位元は `i32::MAX`。
#[derive(Clone, Copy)]
struct Dist(i32);

impl Default for Dist {
    fn default() -> Self {
        Self(i32::MAX)
    }
}
```

**`resolver` は空の列にも同じ単位元を返すこと。**
`l == r` のクエリでは `resolver` に空の列が渡ります。
`iter().min()` のように `Option` を返す処理は、`unwrap_or_default()` で単位元に落とします。

**単位元の取り違えは、ブロックをまたぐ区間でだけ現れます。**
区間が 1 ブロックに収まる場合、`resolve` は `resolver` の結果をそのまま返し、
`Result::default()` も `merger` も経由しません。
サンプルが小さいと素通りするので、手元では長い列で試してください。

**更新したら `into_immut` を呼び直します。**
`push` / `pop` / `set` はキャッシュをその場で作り直さず、印を立てるだけです。
`resolve` は `ImmutableDivConquer` にしかないので、更新後にクエリを投げるには
必ず `into_immut` を通ることになり、印の付いたブロックはそこで張り直されます。
`resolve` 自体は `&self` を取るだけなので、オンラインクエリに使えます。

**`N` は `0` にできません。** ブロックに区切る時点でパニックします。

**区間は半開区間に直してから検査されます。**
`l > r` や `r > n` はパニックしますが、その検査は `l..=r` を `l..r + 1` に直した後に行われます。
`..=usize::MAX` のような端点は検査に届く前に加算が溢れ、
リリースビルドでは `r` が `0` に巻き戻って単位元が返ります。

## verify

Library Checker 1 問と yukicoder 2 問で検証しています。
3 問とも「ブロック内を整列しておき、完全なブロックは二分探索で答える」形ですが、
キャッシュに何を持たせるかが少しずつ違います。

### Library Checker: Static Range Frequency

[Static Range Frequency](https://judge.yosupo.jp/problem/static_range_frequency)
は区間 `[l, r)` に `x` が何回出現するかを答える問題です。
`N, Q <= 5 * 10^5`、値は `10^9` 以下なので `u32` に収まります。

キャッシュはブロックに現れる値を**重複を潰して**昇順に並べた列と、その累積個数です。
単に整列した列を持って `x` の下界と上界を二分探索してもよいのですが、
重複を潰しておけば二分探索は 1 回で済みます。
`uniq[k]` の個数は `cum[k + 1] - cum[k]` で取り出せます。

ブロック長は `8192` で、`√n` の 10 倍以上です。
この問題の `resolver` は 1 要素あたり比較 1 回とベクトル化が効く形なので、
半端な部分を大きく取る方が有利になります。

実際の verify コードは
[`verify/src/bin/library-checker-static-range-frequency.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/library-checker-static-range-frequency.rs)
です。

### yukicoder No.1332 Range Nearest Query

[No.1332 Range Nearest Query](https://yukicoder.me/problems/no/1332) は、
区間内の座標と `x` の距離の最小値を答える問題です。
`N <= 3 * 10^5`、`Q <= 10^5` で、座標は `10^9` 以下です。

**単位元が `0` にならない唯一の verify** で、上の[注意点](#注意点)で挙げた `Dist` はこの問題のものです。
`i32` のまま扱うと `Result::default()` が `0` になり、
「距離 0 の座標があった」ことにされて答えが常に `0` になります。
1 ブロックに収まる短い区間では正しい答えが返るため、
サンプルだけでは気づけない類のバグです。

キャッシュはブロックの座標を昇順に並べた列だけです。
整列してあれば、`x` に最も近いのは `x` 未満の最大の座標か `x` 以上の最小の座標の
どちらかに限られるので、`partition_point` 1 回とその前後を見るだけで済みます。

座標も差も `i32` に収まるため、`i64` にはしていません。
ブロックごとの整列済み配列を小さく保つ方が、二分探索のキャッシュミスが減ります。

実際の verify コードは
[`verify/src/bin/yukicoder-1332.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-1332.rs)
です。

### yukicoder No.3078 Difference Sum Query

[No.3078 Difference Sum Query](https://yukicoder.me/problems/no/3078) は、
区間に対して `Σ|A_i - x|` を答える問題です。`N, Q <= 10^5` です。

キャッシュはブロックの要素を昇順に並べた列と、その累積和です。
`x` 未満の要素は `x - A_i` を、`x` 以上の要素は `A_i - x` を足すので、
境界を `partition_point` で 1 回求めれば、
それぞれの和は累積和の差から `O(1)` で計算できます。

累積和は先頭に `0` を置いた長さ `len + 1` の列にしておくと、
「小さい方から `k` 個の和」が `sum[k]` で引けて、境界の場合分けが消えます。

実際の verify コードは
[`verify/src/bin/yukicoder-3078.rs`](https://github.com/topi-banana/library/blob/main/verify/src/bin/yukicoder-3078.rs)
です。

## 実装メモ

`resolve` は最初に `RangeBounds` を半開区間 `[l, r)` に正規化し、
`lb = l / N` と `rb = r / N` でブロック番号を求めます。

`lb == rb` のときは区間が 1 ブロックに収まっているので、
`resolver` を 1 回呼んで即座に返します。
この分岐は速度のためだけでなく、正しさのために必要です。
一般の経路に流すと、左の半端な部分が `r` を越えてブロックの終わりまで伸び、
右の半端な部分がブロックの先頭から始まるため、
区間の外まで数えたうえに重なった分を二重に数えてしまいます。

`lb != rb` のときは、まず左の半端な部分を処理します。
`l % N == 0` なら `lb` 番のブロックは丸ごと使えるので、キャッシュのループに含めます。
そうでなければ `l` からブロックの終わりまでを `resolver` で解き、
ループは `lb + 1` から始めます。
右側は `r % N != 0` のときだけ、`rb` 番のブロックの先頭から `r` までを `resolver` で解きます。
`rb` 番のブロックはキャッシュのループ (`head..rb`) に含まれないので、
右端の半端な部分と二重に数えることはありません。

`cache` は `slice.chunks(N)` から作るため、末尾のブロックだけは `N` 要素に満たないことがあります。
`r == n` かつ `n % N != 0` のときは `rb` が末尾のブロックを指し、
その分は `resolver` で処理されるので、短いブロックのキャッシュが使われることはありません。

`DivConquer` が `cacher` を保持しているのは、`into_immut` でキャッシュを張り直すためです。
`push` / `pop` / `set` は `dirty[block]` を立てるだけで、
`into_immut` が立っているブロックだけを `cacher` で作り直してから印を下ろします。
`push` がブロックを 1 つ増やしたときは `MaybeUninit::uninit()` を積むので、
その時点ではキャッシュは未初期化です。
`into_immut` を通れば必ず初期化されるため、
そこでの `assume_init_ref` が成り立ちます。

キャッシュを `Vec<MaybeUninit<C>>` で持っている副作用として、
**`C` のデストラクタは一度も走りません**。
`pop` でブロックが減ったときも、`DivConquer` 自体を捨てたときも同じです。
`C` が `Vec` などヒープを持つ型だと、その分は解放されないまま残ります。
1 回の実行で捨てて終わるコンテスト用途では問題になりませんが、
同じプロセスで作っては捨てるループを回す場合は積み上がります。

`resolve` の `where` 節にある `R: Clone` は、現状の畳み込みでは使っていません。
`merger` が値を受け取って返すため、`acc` はムーブで渡せています。

## ソース

`crates/div_conquer/src/lib.rs` の全文です。コードブロック右上のボタンでまるごとコピーできます。
リポジトリのファイルをそのまま埋め込んでいるので、この表示が実装とずれることはありません。

末尾の `#[cfg(test)] mod tests;` はユニットテストを読み込む 2 行です。
提出先ではテストがコンパイルされないため、貼り付けたままで構いません。

```rust,ignore
{{#include ../../../crates/div_conquer/src/lib.rs}}
```
