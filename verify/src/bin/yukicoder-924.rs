// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/924

use proconio::input;
use std::io::{BufWriter, Write};

use mo::{Mo, MoSol};

/// 区間の要素を、座標圧縮した値ごとに数える。
///
/// `f(x) = Σ|x - a_k|` は `x` が区間の中央値のとき最小になるので、
/// 中央値と「中央値までの要素の和」がわかればよい。
/// 値を `√m` 個ずつのバケットにまとめて持ち、
/// 出し入れを `O(1)`、中央値の探索を `O(√m)` にする。
struct AbsSum {
    /// 元の列。`a[i]` は `vals[rank[i]]` に等しい。
    a: Vec<i64>,
    /// 元の列の各要素の、`vals` での順位。
    rank: Vec<usize>,
    /// 昇順に並べた、重複を除いた値。
    vals: Vec<i64>,
    /// 順位ごとの個数。
    cnt: Vec<usize>,
    /// バケットごとの個数。
    bucket_cnt: Vec<usize>,
    /// バケットごとの値の総和。
    bucket_sum: Vec<i64>,
    /// バケット 1 個に入れる順位の数。
    width: usize,
    /// 区間の要素数。
    len: usize,
    /// 区間の値の総和。
    sum: i64,
}

impl AbsSum {
    fn add(&mut self, i: usize) {
        let (r, v) = (self.rank[i], self.a[i]);
        self.cnt[r] += 1;
        self.bucket_cnt[r / self.width] += 1;
        self.bucket_sum[r / self.width] += v;
        self.len += 1;
        self.sum += v;
    }
    fn del(&mut self, i: usize) {
        let (r, v) = (self.rank[i], self.a[i]);
        self.cnt[r] -= 1;
        self.bucket_cnt[r / self.width] -= 1;
        self.bucket_sum[r / self.width] -= v;
        self.len -= 1;
        self.sum -= v;
    }
}

impl MoSol for AbsSum {
    type Ans = i64;
    // 添字は 0..=2·10^5 の範囲に収まる。2^18 = 262144 > 2·10^5。
    const MAX_INDEX_POW2: usize = 18;
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
        // 小さい方から k 番目 (1-indexed) が中央値。
        let k = self.len.div_ceil(2);
        // 中央値を含めた、下位 k 個の値の総和。
        let mut low = 0;
        let mut rest = k;
        // バケット単位で飛ばしてから、バケットの中を 1 順位ずつ見る。
        let mut b = 0;
        while self.bucket_cnt[b] < rest {
            rest -= self.bucket_cnt[b];
            low += self.bucket_sum[b];
            b += 1;
        }
        let mut r = b * self.width;
        while self.cnt[r] < rest {
            rest -= self.cnt[r];
            low += self.vals[r] * self.cnt[r] as i64;
            r += 1;
        }
        // 同じ値が複数あるときは、下位 k 個に入る分の rest 個だけ足す。
        low += self.vals[r] * rest as i64;

        let med = self.vals[r];
        let below = med * k as i64 - low;
        let above = (self.sum - low) - med * (self.len - k) as i64;
        below + above
    }
}

fn main() {
    input! {
        n: usize,
        q: usize,
        a: [i64; n],
        queries: [(usize, usize); q],
    }

    let mut vals = a.clone();
    vals.sort_unstable();
    vals.dedup();
    let rank = a
        .iter()
        .map(|v| vals.binary_search(v).unwrap())
        .collect::<Vec<_>>();

    let m = vals.len();
    let width = m.isqrt().max(1);
    let buckets = m.div_ceil(width);
    let mut state = AbsSum {
        a,
        rank,
        vals,
        cnt: vec![0; m],
        bucket_cnt: vec![0; buckets],
        bucket_sum: vec![0; buckets],
        width,
        len: 0,
        sum: 0,
    };

    let mut mo = Mo::new();
    for (l, r) in queries {
        // 入力は 1-indexed の閉区間、Mo は 0-indexed の半開区間。
        mo.push(l - 1, r);
    }
    let ans = mo.execute(&mut state);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for x in ans.iter() {
        writeln!(out, "{x}").unwrap();
    }
}
