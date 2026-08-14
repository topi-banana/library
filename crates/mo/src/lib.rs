//! Mo's algorithm (クエリ平方分割)。
//!
//! 列に対する `q` 個の区間クエリ `[l, r)` を、オフラインでまとめて処理する。
//! 見ている区間を 1 要素ずつ伸縮させながら状態を更新していくので、
//! 「区間の端を 1 要素だけ動かす」操作が軽い問題に使える。
//!
//! クエリを処理する順序は Hilbert 曲線の順で決める。
//! 区間の端の移動量の合計は `O(n √q)` 程度で、
//! ブロック分割による古典的な順序より定数倍が小さい。
//!
//! 使う側は [`MoSol`] を実装した状態を用意し、[`Mo::push`] でクエリを積んでから
//! [`Mo::execute`] を呼ぶ。答えは push した順に並んだ `Box<[Ans]>` で返る。
//!
//! ```
//! use mo::{Mo, MoSol};
//!
//! /// 区間に含まれる異なる値の個数。
//! struct Distinct {
//!     a: Vec<usize>,
//!     cnt: Vec<usize>,
//!     distinct: usize,
//! }
//!
//! impl Distinct {
//!     fn add(&mut self, i: usize) {
//!         if self.cnt[self.a[i]] == 0 {
//!             self.distinct += 1;
//!         }
//!         self.cnt[self.a[i]] += 1;
//!     }
//!     fn del(&mut self, i: usize) {
//!         self.cnt[self.a[i]] -= 1;
//!         if self.cnt[self.a[i]] == 0 {
//!             self.distinct -= 1;
//!         }
//!     }
//! }
//!
//! impl MoSol for Distinct {
//!     type Ans = usize;
//!     // 添字は 0..=5 の範囲に収まる。2^3 = 8 > 5。
//!     const MAX_INDEX_POW2: usize = 3;
//!     fn add_l(&mut self, l_idx: usize) {
//!         self.add(l_idx);
//!     }
//!     fn add_r(&mut self, r_idx: usize) {
//!         self.add(r_idx);
//!     }
//!     fn del_l(&mut self, l_idx: usize) {
//!         self.del(l_idx);
//!     }
//!     fn del_r(&mut self, r_idx: usize) {
//!         self.del(r_idx);
//!     }
//!     fn solve(&mut self) -> Self::Ans {
//!         self.distinct
//!     }
//! }
//!
//! let a = vec![1, 2, 1, 3, 2];
//! // 空区間 [0, 0) を表す状態から始める。
//! let mut state = Distinct {
//!     cnt: vec![0; 4],
//!     a,
//!     distinct: 0,
//! };
//!
//! let mut mo = Mo::new();
//! mo.push(0, 3); // [1, 2, 1]    -> 2
//! mo.push(1, 5); // [2, 1, 3, 2] -> 3
//! mo.push(0, 1); // [1]          -> 1
//!
//! assert_eq!(*mo.execute(&mut state), [2, 3, 1]);
//! ```

/// [`Mo`] が伸縮させる、区間 `[l, r)` に対する状態。
///
/// [`Mo::execute`] は現在の区間を 1 要素ずつ動かしながら
/// [`add_l`](Self::add_l) / [`add_r`](Self::add_r) /
/// [`del_l`](Self::del_l) / [`del_r`](Self::del_r) を呼び、
/// クエリの区間に一致した時点で [`solve`](Self::solve) を呼ぶ。
///
/// 4 つのメソッドが受け取るのは、いずれも**出入りする要素そのものの添字**であって、
/// 移動後の区間の端ではない。
///
/// 全体の計算量は、これらのメソッド 1 回あたりの計算量を `O(f)` として
/// `O(n √q · f + q · g)` 程度になる (`g` は [`solve`](Self::solve) 1 回の計算量)。
pub trait MoSol {
    /// 1 クエリの答え。
    ///
    /// [`Mo::execute`] が答えを push 順に並べ替えるため、`Default + Clone` が必要。
    type Ans;

    /// Hilbert 曲線の一辺を `2^MAX_INDEX_POW2` とする。
    ///
    /// すべてのクエリの `l` と `r` がこの値未満でなければならない。
    /// `r` は列の長さ `n` そのものになりうるので、`2^MAX_INDEX_POW2 > n` を
    /// 満たす最小の値を選ぶ。`n = 10^5` なら `17` (`2^17 = 131072`)。
    ///
    /// 範囲外の添字を渡してもパニックはしないが、
    /// 並べ替えの順序が壊れて高速化の効果を失う。
    const MAX_INDEX_POW2: usize;

    /// 区間を `[l..r)` から `[l-1..r)` へ広げる。`l_idx` は新しく入る要素の添字 (`l - 1`)。
    fn add_l(&mut self, l_idx: usize);
    /// 区間を `[l..r)` から `[l..r+1)` へ広げる。`r_idx` は新しく入る要素の添字 (`r`)。
    fn add_r(&mut self, r_idx: usize);
    /// 区間を `[l..r)` から `[l+1..r)` へ狭める。`l_idx` は取り除く要素の添字 (`l`)。
    fn del_l(&mut self, l_idx: usize);
    /// 区間を `[l..r)` から `[l..r-1)` へ狭める。`r_idx` は取り除く要素の添字 (`r - 1`)。
    fn del_r(&mut self, r_idx: usize);
    /// 現在の区間に対する答えを返す。
    fn solve(&mut self) -> Self::Ans;
}

/// クエリを貯めておき、Hilbert 曲線の順に処理するバッファ。
///
/// 使い方は [crate のドキュメント](crate)を参照。
#[derive(Debug, Clone, Default)]
pub struct Mo {
    queries: Vec<(usize, usize, usize)>,
}
impl Mo {
    /// 空の [`Mo`] を作る。
    pub fn new() -> Self {
        Self::default()
    }

    /// 半開区間 `[l..r)` に対するクエリを積む。
    ///
    /// `l <= r` かつ `r < 2^MoSol::MAX_INDEX_POW2` であること。
    /// このクエリの答えは、[`execute`](Self::execute) が返す列の
    /// 「何番目に push したか」の位置に入る。
    pub fn push(&mut self, l: usize, r: usize) {
        self.queries.push((l, r, self.queries.len()));
    }

    /// 積んだクエリをすべて処理し、push した順に並べた答えを返す。
    ///
    /// `state` は空区間 `[0..0)` を表す状態でなければならない
    /// (現在の区間は `[0..0)` から動かし始める)。
    pub fn execute<S: MoSol>(mut self, state: &mut S) -> Box<[S::Ans]>
    where
        S::Ans: Default + Clone,
    {
        /// `2^pow` 四方の Hilbert 曲線上で、点 `(x, y)` が何番目に来るかを返す。
        ///
        /// `rotate` は再帰の各段でのマスの向き。呼び出し側は `0` を渡す。
        fn hilbert_order(x: usize, y: usize, pow: usize, rotate: usize) -> usize {
            if pow == 0 {
                return 0;
            }
            let hpow = 1 << (pow - 1);
            let seg = match (x < hpow, y < hpow) {
                (true, true) => 0,
                (true, false) => 3,
                (false, true) => 1,
                (false, false) => 2,
            };
            let seg = (seg + rotate) & 3;
            let (nx, ny) = (x & (x ^ hpow), y & (y ^ hpow));
            let nrot = (rotate + [3, 0, 0, 1][seg]) & 3;
            let sub_square_size = 1usize << ((pow << 1) - 2);
            let ans = seg * sub_square_size;
            let add = hilbert_order(nx, ny, pow - 1, nrot);
            if seg == 1 || seg == 2 {
                ans + add
            } else {
                ans + sub_square_size - add - 1
            }
        }
        self.queries
            .sort_by_cached_key(|&(l, r, _)| hilbert_order(l, r, S::MAX_INDEX_POW2, 0));

        let mut ans = vec![S::Ans::default(); self.queries.len()].into_boxed_slice();
        let (mut nl, mut nr) = (0, 0);
        for (l, r, i) in self.queries {
            // 縮めるより先に広げることで、途中で `nl > nr` にならないようにする。
            while nl > l {
                nl -= 1;
                state.add_l(nl);
            }
            while nr < r {
                state.add_r(nr);
                nr += 1
            }
            while nl < l {
                state.del_l(nl);
                nl += 1
            }
            while nr > r {
                nr -= 1;
                state.del_r(nr);
            }
            ans[i] = state.solve();
        }
        ans
    }
}
