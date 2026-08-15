// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/static_range_count_distinct

use proconio::input;
use std::io::{BufWriter, Write};

use mo::{Mo, MoSol};

/// 区間に含まれる相異なる値の個数。
///
/// `N, Q <= 5 * 10^5` と大きく、端の移動は 10^8 のオーダーで起こる。
/// 内側のループが触る配列を小さく保つため、順位と個数は `u32` で持つ。
struct Distinct {
    /// 元の列の各要素の、座標圧縮した値での順位。
    rank: Vec<u32>,
    /// 順位ごとの個数。
    cnt: Vec<u32>,
    /// 区間に現れる値の種類数。
    distinct: usize,
}

impl Distinct {
    fn add(&mut self, i: usize) {
        let r = self.rank[i] as usize;
        if self.cnt[r] == 0 {
            self.distinct += 1;
        }
        self.cnt[r] += 1;
    }
    fn del(&mut self, i: usize) {
        let r = self.rank[i] as usize;
        self.cnt[r] -= 1;
        if self.cnt[r] == 0 {
            self.distinct -= 1;
        }
    }
}

impl MoSol for Distinct {
    type Ans = usize;
    // 添字は 0..=5·10^5 の範囲に収まる。2^19 = 524288 > 5·10^5。
    const MAX_INDEX_POW2: usize = 19;
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

fn main() {
    input! {
        n: usize,
        q: usize,
        a: [i64; n],
        queries: [(usize, usize); q],
    }

    // 値そのものは使わないので、順位だけあればよい。
    let mut vals = a.clone();
    vals.sort_unstable();
    vals.dedup();
    let rank = a.iter().map(|v| vals.binary_search(v).unwrap() as u32).collect::<Vec<_>>();

    let mut state = Distinct { rank, cnt: vec![0; vals.len()], distinct: 0 };

    let mut mo = Mo::new();
    for (l, r) in queries {
        // 入力は 0-indexed の半開区間なので、そのまま渡せる。
        mo.push(l, r);
    }
    let ans = mo.execute(&mut state);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for x in ans.iter() {
        writeln!(out, "{x}").unwrap();
    }
}
