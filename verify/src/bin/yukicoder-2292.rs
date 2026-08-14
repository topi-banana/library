// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/2292

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use ranges::Ranges;

fn main() {
    input! {
        _n: u32,
        q: usize,
    }

    // 頂点 v と v+1 を結ぶ辺に番号 v を振り、存在する辺の集合を半開区間で持つ。
    // 辺 start..end が 1 本の区間になっていれば、頂点 start..=end が 1 つの連結成分である。
    //
    // 頂点側を区間にしないのは、隣り合う別々の成分 (例えば {1,2,3} と {4,5}) が
    // 半開区間 1..4 と 4..6 になって接してしまい、insert が統合してしまうからである。
    // 辺側なら、2 つの区間が接するにはその頂点を両方の成分が共有していなければならず、
    // そのようなことは起こらない。
    let mut edges: Ranges<u32> = Ranges::new();

    let mut out = String::new();
    for _ in 0..q {
        input! { kind: u8 }
        match kind {
            // L <= u < v <= R を全て結ぶ = 辺 L, L+1, ..., R-1 を張る。
            1 => {
                input! { l: u32, r: u32 }
                edges.insert(l..r);
            }
            // 残る辺は両端が [1, R] に収まるものと [L, N] に収まるものだけになる。
            // すなわち辺 L, L+1, ..., R-1 を消すことと同じで、
            // 跨いでいた成分は remove によって 2 つに割れる。
            2 => {
                input! { l: u32, r: u32 }
                edges.remove(l..r);
            }
            // u と v が連結 <=> その間の辺が 1 本残らず繋がっている。
            // u, v の大小は保証されないので並べ替える。u == v は空区間になり、
            // contains_range が true を返すので連結として扱われる。
            3 => {
                input! { u: u32, v: u32 }
                let (lo, hi) = (u.min(v), u.max(v));
                let connected = u8::from(edges.contains_range(&(lo..hi)));
                writeln!(out, "{connected}").unwrap();
            }
            // 辺区間 start..end は頂点 start..=end に対応するので、サイズは end - start + 1。
            // v が成分の右端のときは辺 v が無く covering(&v) が None になるため、
            // 左隣の辺 v-1 も見る。どちらも当たる場合は同じ区間なので結果は変わらない。
            _ => {
                input! { v: u32 }
                let size = match edges.covering(&v).or_else(|| edges.covering(&(v - 1))) {
                    Some((start, end)) => u64::from(end - start) + 1,
                    None => 1,
                };
                writeln!(out, "{size}").unwrap();
            }
        }
    }

    let stdout = std::io::stdout();
    let mut o = stdout.lock();
    o.write_all(out.as_bytes()).unwrap();
}
