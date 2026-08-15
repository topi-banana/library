// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1332

use proconio::input;
use std::io::{BufWriter, Write};

use div_conquer::DivConquer;

/// ブロック長。
///
/// 1 クエリの計算量は「完全なブロック `n / B` 個の二分探索」と
/// 「両端の半端な部分 高々 `2B` 要素の走査」の和になる。
/// 走査は 1 要素あたり減算と比較だけで SIMD 化も効くのに対し、
/// 二分探索は 1 段ごとにキャッシュミスしうる依存した読み出しなので、
/// 定数倍は走査の方がずっと軽い。そのぶん `√n` より大きめに取る。
const B: usize = 4096;

/// 座標との距離の最小値。
///
/// `DivConquer` は部分結果を `Default` から畳み込むので、
/// 単位元が `0` になってしまう `i32` はそのままでは使えない。
/// 最小値の単位元は `i32::MAX` なので、新しい型に包んで `Default` を与える。
#[derive(Clone, Copy)]
struct Dist(i32);

impl Default for Dist {
    fn default() -> Self {
        Self(i32::MAX)
    }
}

/// ブロックの座標を昇順に並べたもの。
type Cache = Vec<i32>;

fn cacher(block: &[i32]) -> Cache {
    let mut sorted = block.to_vec();
    sorted.sort_unstable();
    sorted
}

fn resolver(block: &[i32], &x: &i32) -> Dist {
    let min = block.iter().map(|&v| (v - x).abs()).min();
    // 空の区間ではマージの単位元をそのまま返す。
    min.map(Dist).unwrap_or_default()
}

fn cache_resolver(sorted: &Cache, &x: &i32) -> Dist {
    // 整列してあれば、`x` に最も近いのは `x` 未満の最大の座標か、
    // `x` 以上の最小の座標のどちらかに限られる。
    let k = sorted.partition_point(|&v| v < x);
    let left = if k > 0 { x - sorted[k - 1] } else { i32::MAX };
    let right = if k < sorted.len() { sorted[k] - x } else { i32::MAX };
    Dist(left.min(right))
}

fn merger(a: Dist, b: Dist) -> Dist {
    Dist(a.0.min(b.0))
}

fn main() {
    input! {
        n: usize,
        // 座標は 10^9 以下なので、差も含めて i32 に収まる。
        // ブロックごとの整列済み配列を小さく保つため、i64 にはしない。
        x: [i32; n],
        q: usize,
        queries: [(usize, usize, i32); q],
    }

    let dc = DivConquer::<B, _, _, _, _>::new(x, cacher, resolver, cache_resolver, merger);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for (l, r, x) in queries {
        // 入力は 1-indexed の閉区間、`resolve` は 0-indexed の半開区間。
        writeln!(out, "{}", dc.resolve(l - 1..r, &x).0).unwrap();
    }
}
