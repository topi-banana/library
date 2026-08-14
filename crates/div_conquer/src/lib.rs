//! 平方分割 (ブロック分割) による区間クエリ。
//!
//! 列を長さ `N` のブロックに区切り、ブロックごとの事前計算結果 (キャッシュ) を持っておく。
//! 区間 `[l, r)` のクエリは「両端の半端な部分は生の要素から」
//! 「間に挟まる完全なブロックはキャッシュから」解き、部分結果をマージして答える。
//!
//! セグメント木に載らない (マージできない) 種類の問い合わせでも、
//! ブロック内で整列や累積和を作っておけば答えられる、という問題が対象になる。
//! 区間内の特定の値の個数、区間内の値と `x` の距離の和、区間内で `x` に最も近い値などが典型。
//!
//! crate 名は `div_conquer` だが、分割統治ではなく平方分割 (ブロック分割) である。
//!
//! 列は構築時に固定され、更新はできない。
//! 同じ列に対して区間と引数を変えながら何度も問い合わせる用途を想定している。
//!
//! # 使い方
//!
//! [`DivConquer::new`] に列と 4 つの関数を渡し、[`DivConquer::resolve`] で問い合わせる。
//!
//! | 関数 | 役割 |
//! | --- | --- |
//! | `cacher` | ブロックの要素から事前計算結果を作る |
//! | `resolver` | 生の要素の列と引数からクエリを解く |
//! | `cache_resolver` | キャッシュと引数からクエリを解く |
//! | `merger` | 2 つの部分結果をマージする |
//!
//! 4 つとも [`Fn`] ではなく**関数ポインタ**で受け取る。
//! `fn` 項目と、何も捕捉しないクロージャはそのまま渡せるが、捕捉するクロージャは渡せない。
//! ブロックの外にある表 (座標圧縮の対応表など) を引きたい場合は、
//! クエリごとの引数 `Arg` に載せて `resolve` から渡す。
//!
//! # 要件
//!
//! - `merger` は結合的で、`Result` の [`Default`] がその**単位元**であること。
//!   部分結果は `Result::default()` から畳み込まれる。
//!   単位元が `0` でない演算 (最小値など) では、型を新しく作って [`Default`] を与える。
//! - `resolver` は空の列に対しても同じ単位元を返すこと。
//!   区間が 1 ブロックに収まるときは `resolver` だけが呼ばれ、`l == r` なら空の列が渡る。
//! - `cacher` と `resolver` と `cache_resolver` の答えが一致すること。
//!   すなわち、任意のブロック `b` と引数 `a` について
//!   `cache_resolver(&cacher(b), a) == resolver(b, a)` であること。
//!
//! # 例
//!
//! 区間 `[l, r)` に含まれる `x` の個数を数える。
//! ブロックごとに値を整列しておけば、完全なブロックは二分探索で数えられる。
//!
//! ```
//! use div_conquer::DivConquer;
//!
//! /// ブロックの要素を昇順に並べたもの。
//! type Cache = Vec<u32>;
//!
//! fn cacher(block: &[u32]) -> Cache {
//!     let mut sorted = block.to_vec();
//!     sorted.sort_unstable();
//!     sorted
//! }
//!
//! /// 半端な部分は生の要素をそのまま走査する。
//! fn resolver(block: &[u32], &x: &u32) -> usize {
//!     block.iter().filter(|&&v| v == x).count()
//! }
//!
//! /// 完全なブロックは整列済みなので、上界と下界の差で個数が求まる。
//! fn cache_resolver(sorted: &Cache, &x: &u32) -> usize {
//!     sorted.partition_point(|&v| v <= x) - sorted.partition_point(|&v| v < x)
//! }
//!
//! /// 個数の和。`usize::default()` は `0` で、これは加算の単位元。
//! fn merger(a: usize, b: usize) -> usize {
//!     a + b
//! }
//!
//! let a = vec![1, 2, 1, 3, 2, 1, 2, 1];
//! // 最初の型引数がブロック長。実際の問題では `√n` より大きめに取る。
//! let dc = DivConquer::<4, _, _, _, _>::new(a, cacher, resolver, cache_resolver, merger);
//!
//! assert_eq!(dc.resolve(0..8, &1), 4);
//! assert_eq!(dc.resolve(1..6, &2), 2); // 半端な部分だけで解く
//! assert_eq!(dc.resolve(2..3, &1), 1); // 1 ブロックに収まる区間
//! assert_eq!(dc.resolve(.., &2), 3); // `RangeBounds` なら何でも渡せる
//! ```
//!
//! 単位元が `0` にならない例として、区間内の値と `x` の距離の最小値を求める。
//! `i32` の [`Default`] は `0` で最小値の単位元ではないため、新しい型に包む。
//!
//! ```
//! use div_conquer::DivConquer;
//!
//! /// 距離の最小値。単位元は `i32::MAX`。
//! #[derive(Debug, Clone, Copy, PartialEq)]
//! struct Dist(i32);
//!
//! impl Default for Dist {
//!     fn default() -> Self {
//!         Self(i32::MAX)
//!     }
//! }
//!
//! /// ブロックの要素を昇順に並べたもの。
//! type Cache = Vec<i32>;
//!
//! fn cacher(block: &[i32]) -> Cache {
//!     let mut sorted = block.to_vec();
//!     sorted.sort_unstable();
//!     sorted
//! }
//!
//! fn resolver(block: &[i32], &x: &i32) -> Dist {
//!     // 空の列ではマージの単位元をそのまま返す。
//!     block.iter().map(|&v| (v - x).abs()).min().map(Dist).unwrap_or_default()
//! }
//!
//! fn cache_resolver(sorted: &Cache, &x: &i32) -> Dist {
//!     // 整列してあれば、`x` に最も近いのは `x` 未満の最大の値か `x` 以上の最小の値。
//!     let k = sorted.partition_point(|&v| v < x);
//!     let left = if k > 0 { x - sorted[k - 1] } else { i32::MAX };
//!     let right = if k < sorted.len() { sorted[k] - x } else { i32::MAX };
//!     Dist(left.min(right))
//! }
//!
//! fn merger(a: Dist, b: Dist) -> Dist {
//!     Dist(a.0.min(b.0))
//! }
//!
//! let x = vec![10, 40, 20, 50, 30, 90, 70, 60];
//! let dc = DivConquer::<4, _, _, _, _>::new(x, cacher, resolver, cache_resolver, merger);
//!
//! assert_eq!(dc.resolve(.., &45), Dist(5));
//! assert_eq!(dc.resolve(0..3, &45), Dist(5));
//! assert_eq!(dc.resolve(5..8, &45), Dist(15));
//! ```

/// 平方分割 (ブロック分割) された列。
///
/// 長さ `n` の列を長さ `N` のブロックに区切り、各ブロックのキャッシュを保持する。
/// 型引数はブロック長 `N` のほか、要素の型 `Element`、キャッシュの型 `Cache`、
/// 答えの型 `Result`、クエリごとの引数の型 `Arg` の 4 つを取る。
///
/// 構築は [`new`](Self::new)、問い合わせは [`resolve`](Self::resolve) で行う。
/// 満たすべき要件は [crate のドキュメント](crate#要件)を参照。
///
/// なお `Result` は型引数の名前であって、[`std::result::Result`] とは関係がない。
pub struct DivConquer<const N: usize, Element, Cache, Result, Arg> {
    slice: Vec<Element>,
    cache: Vec<Cache>,
    // 点更新でブロックのキャッシュを張り直すために保持している。更新系を実装したら expect を外す。
    #[expect(dead_code)]
    cacher: fn(&[Element]) -> Cache,
    resolver: fn(&[Element], &Arg) -> Result,
    cache_resolver: fn(&Cache, &Arg) -> Result,
    merger: fn(Result, Result) -> Result,
}

impl<const N: usize, E, C, R, A> DivConquer<N, E, C, R, A> {
    /// 列と 4 つの関数から構造を作る。
    ///
    /// 列を先頭から `N` 要素ずつに区切り、各ブロックに `cacher` を適用してキャッシュを作る。
    /// 末尾のブロックだけは `N` 要素に満たないことがある。
    ///
    /// 列の長さを `n`、`cacher` 1 回の計算量を `O(c)` として `O(n / N · c)` 時間。
    /// ブロック内の整列を `cacher` でやる場合は `O(n log N)` になる。
    ///
    /// ブロック長 `N` は `√n` 程度から始めて、実際の定数倍で決める。
    /// 半端な部分の走査は 1 要素あたりの処理が軽くベクトル化も効くのに対し、
    /// ブロックごとの二分探索は依存した読み出しでキャッシュミスしうるため、
    /// 実測では `√n` よりかなり大きく取った方が速いことが多い。
    ///
    /// # Panics
    ///
    /// ブロック長 `N` が `0` のときパニックする。
    pub fn new(
        slice: Vec<E>,
        cacher: fn(&[E]) -> C,
        resolver: fn(&[E], &A) -> R,
        cache_resolver: fn(&C, &A) -> R,
        merger: fn(R, R) -> R,
    ) -> Self {
        let cache = slice.chunks(N).map(&cacher).collect::<Vec<_>>();
        Self {
            slice,
            cache,
            cacher,
            resolver,
            cache_resolver,
            merger,
        }
    }
    /// 区間 `range` に対するクエリを引数 `arg` で解く。
    ///
    /// 区間は [`RangeBounds`](std::ops::RangeBounds) なら何でも渡せる。
    /// 内部では半開区間 `[l, r)` に直してから扱うので、`l..r` も `l..=r` も `..` も使える。
    ///
    /// 両端の半端な部分は `resolver` が高々 `2N` 要素を走査し、
    /// 間に挟まる完全なブロックは `cache_resolver` が高々 `n / N` 個を処理する。
    /// `resolver` の 1 要素あたりを `O(f)`、`cache_resolver` 1 回を `O(g)` として
    /// `O(N · f + n / N · g)` 時間。
    ///
    /// 区間が 1 ブロックに収まる場合は `resolver` だけを呼び、
    /// `Result::default()` も `merger` も経由しない。
    /// そのため単位元の取り違えは、ブロックをまたぐ区間でだけ答えを間違える形で現れる。
    ///
    /// # Panics
    ///
    /// 半開区間に直したうえで `l > r` または `r > n` のときパニックする。
    ///
    /// なお正規化は検査より先に行うため、`..=usize::MAX` のような端点は加算で溢れる。
    /// オーバーフロー検査が無い場合は `r` が `0` に巻き戻り、
    /// パニックせずに単位元が返る点に注意すること。
    pub fn resolve(&self, range: impl std::ops::RangeBounds<usize>, arg: &A) -> R
    where
        R: Default + Clone,
    {
        use std::ops::Bound;

        let l = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.slice.len(),
        };
        assert!(l <= r && r <= self.slice.len(), "range out of bounds");

        let (lb, rb) = (l / N, r / N);

        if lb == rb {
            return (self.resolver)(&self.slice[l..r], arg);
        }

        let mut acc = R::default();

        let head = if l % N == 0 {
            lb
        } else {
            acc = (self.merger)(acc, (self.resolver)(&self.slice[l..(lb + 1) * N], arg));
            lb + 1
        };

        for b in head..rb {
            acc = (self.merger)(acc, (self.cache_resolver)(&self.cache[b], arg));
        }

        if r % N != 0 {
            acc = (self.merger)(acc, (self.resolver)(&self.slice[rb * N..r], arg));
        }

        acc
    }
}
