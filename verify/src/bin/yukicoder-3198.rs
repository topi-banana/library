// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/3198

use proconio::input;
use std::io::{BufWriter, Write};

use div_conquer::DivConquer;

/// ブロック長。
///
/// 1 クエリの計算量は「完全なブロック `len / B` 個の最大値」「両端の半端な部分
/// 高々 `2B` 要素の走査」「末尾に追加して汚れたブロック 1 個の作り直し `B` 要素の走査」の和になる。
/// どれも整数の最大値を取るだけで定数倍に差がないので、`len / B + 3B` を最小にする
/// `√(len / 3) ≒ 258` に近い 2 冪を選ぶ。
const B: usize = 256;

/// ブロックに含まれる最大値。
fn cacher(block: &[u32]) -> u32 {
    resolver(block, &())
}

/// 区間の最大値。
///
/// `DivConquer::resolve` は `Default` から畳み込みを始めるので、
/// 空の区間では最大値の単位元を返さなければならない。
/// $1 \le x_i$ より `0` はどの要素より小さく、単位元として使える。
fn resolver(block: &[u32], _: &()) -> u32 {
    block.iter().copied().max().unwrap_or_default()
}

fn cache_resolver(cache: &u32, _: &()) -> u32 {
    *cache
}

fn merger(a: u32, b: u32) -> u32 {
    a.max(b)
}

fn main() {
    input! { q: usize }

    let mut dc =
        DivConquer::<B, _, _, _, _>::new(Vec::new(), cacher, resolver, cache_resolver, merger);
    // `DivConquer` は長さを教えてくれないので、末尾からの区間を作るために自分で数える。
    let mut len = 0;

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for _ in 0..q {
        input! { kind: u8 }
        match kind {
            1 => {
                input! { x: u32 }
                dc.push(x);
                len += 1;
            }
            _ => {
                input! { k: usize }
                writeln!(out, "{}", dc.into_immut().resolve(len - k.., &())).unwrap();
            }
        }
    }
}
