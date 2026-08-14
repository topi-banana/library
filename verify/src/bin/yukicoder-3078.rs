// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/3078

use proconio::input;
use std::io::{BufWriter, Write};

use div_conquer::DivConquer;

/// ブロック長。
///
/// 1 クエリの計算量は「完全なブロック `n / B` 個の二分探索」と
/// 「両端の半端な部分 高々 `2B` 要素の走査」の和になる。
/// 走査は 1 要素あたり減算と加算だけで SIMD 化も効くのに対し、
/// 二分探索は 1 段ごとにキャッシュミスしうる依存した読み出しなので、
/// 定数倍は走査の方がずっと軽い。そのぶん `√n` より大きめに取る。
const B: usize = 2048;

/// ブロックの要素を昇順に並べたものと、その累積和。
///
/// 累積和は先頭に `0` を置いた長さ `len + 1` の列で、
/// `sum[k]` が小さい方から `k` 個の和になる。
type Cache = (Vec<i64>, Vec<i64>);

fn cacher(block: &[i64]) -> Cache {
    let mut sorted = block.to_vec();
    sorted.sort_unstable();

    let mut sum = Vec::with_capacity(sorted.len() + 1);
    let mut acc = 0;
    sum.push(acc);
    for &a in &sorted {
        acc += a;
        sum.push(acc);
    }

    (sorted, sum)
}

fn resolver(block: &[i64], &x: &i64) -> i64 {
    block.iter().map(|&a| (a - x).abs()).sum()
}

fn cache_resolver((sorted, sum): &Cache, &x: &i64) -> i64 {
    // `x` 未満の要素は `x - a` を、`x` 以上の要素は `a - x` を足す。
    // 整列してあれば境界は 1 回の二分探索で決まり、
    // それぞれの和は累積和から求まる。
    let k = sorted.partition_point(|&a| a < x);
    let (lo, hi) = (sum[k], sum[sorted.len()] - sum[k]);
    (x * k as i64 - lo) + (hi - x * (sorted.len() - k) as i64)
}

fn merger(a: i64, b: i64) -> i64 {
    a + b
}

fn main() {
    input! {
        n: usize,
        q: usize,
        a: [i64; n],
        queries: [(usize, usize, i64); q],
    }

    let dc = DivConquer::<B, _, _, _, _>::new(a, cacher, resolver, cache_resolver, merger);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for (l, r, x) in queries {
        // 入力は 1-indexed の閉区間、`resolve` は 0-indexed の半開区間。
        writeln!(out, "{}", dc.resolve(l - 1..r, &x)).unwrap();
    }
}
