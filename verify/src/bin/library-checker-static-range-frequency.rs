// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/static_range_frequency

use proconio::input;
use std::io::{BufWriter, Write};

use div_conquer::DivConquer;

/// ブロック長。
///
/// 1 クエリの計算量は「完全なブロック `n / B` 個の二分探索」と
/// 「両端の半端な部分 高々 `2B` 要素の走査」の和になる。
/// 走査は 1 要素あたり比較 1 回で SIMD 化も効くのに対し、
/// 二分探索は 1 段ごとにキャッシュミスしうる依存した読み出しなので、
/// 定数倍は走査の方が 1 桁以上軽い。そのぶんブロックを大きめに取る。
const B: usize = 8192;

/// ブロックに現れる値の昇順の列と、その累積個数。
///
/// `uniq[k]` の個数は `cum[k + 1] - cum[k]` で、`cum` の長さは `uniq` より 1 大きい。
/// 単に整列した列を持って `x` の下界と上界を二分探索してもよいが、
/// 重複を潰しておけば二分探索は 1 回で済む。
type Cache = (Vec<u32>, Vec<u32>);

fn cacher(block: &[u32]) -> Cache {
    let mut sorted = block.to_vec();
    sorted.sort_unstable();

    let mut uniq = Vec::new();
    let mut cum = Vec::new();
    for (i, &v) in sorted.iter().enumerate() {
        if uniq.last() != Some(&v) {
            uniq.push(v);
            cum.push(i as u32);
        }
    }
    cum.push(sorted.len() as u32);

    (uniq, cum)
}

fn resolver(block: &[u32], &x: &u32) -> usize {
    block.iter().filter(|&&v| v == x).count()
}

fn cache_resolver((uniq, cum): &Cache, &x: &u32) -> usize {
    let k = uniq.partition_point(|&v| v < x);
    if uniq.get(k) == Some(&x) {
        (cum[k + 1] - cum[k]) as usize
    } else {
        0
    }
}

fn merger(a: usize, b: usize) -> usize {
    a + b
}

fn main() {
    input! {
        n: usize,
        q: usize,
        a: [u32; n],
        queries: [(usize, usize, u32); q],
    }

    let dc = DivConquer::<B, _, _, _, _>::new(a, cacher, resolver, cache_resolver, merger);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for (l, r, x) in queries {
        // 入力は 0-indexed の半開区間なので、そのまま渡せる。
        writeln!(out, "{}", dc.resolve(l..r, &x)).unwrap();
    }
}
