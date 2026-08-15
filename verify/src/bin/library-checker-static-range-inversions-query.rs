// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/static_range_inversions_query

use proconio::input;
use std::io::{BufWriter, Write};

use mo::{Mo, MoSol};

/// 個数を載せた Fenwick tree。
struct Fenwick {
    tree: Vec<u32>,
}

impl Fenwick {
    fn new(n: usize) -> Self {
        Self { tree: vec![0; n + 1] }
    }
    /// 位置 `i` の個数を 1 増やす。
    fn add(&mut self, i: usize) {
        let mut x = i + 1;
        while x < self.tree.len() {
            self.tree[x] += 1;
            x += x & x.wrapping_neg();
        }
    }
    /// 位置 `i` の個数を 1 減らす。
    fn sub(&mut self, i: usize) {
        let mut x = i + 1;
        while x < self.tree.len() {
            self.tree[x] -= 1;
            x += x & x.wrapping_neg();
        }
    }
    /// `0..i` の個数の和。
    fn sum(&self, i: usize) -> u64 {
        let mut x = i;
        let mut acc = 0;
        while x > 0 {
            acc += u64::from(self.tree[x]);
            x -= x & x.wrapping_neg();
        }
        acc
    }
}

/// 区間の転倒数。
///
/// 5 つの verify のうち、これだけが `add_l` と `add_r` で処理が変わる。
/// 左端に入る要素が作る転倒は「区間内の自分より小さい要素」との組で、
/// 右端に入る要素が作る転倒は「区間内の自分より大きい要素」との組になる。
struct Inversions {
    /// 元の列の各要素の、座標圧縮した値での順位。
    rank: Vec<usize>,
    /// 区間内の要素の個数を順位ごとに載せた Fenwick tree。
    bit: Fenwick,
    /// 区間の要素数。
    len: u64,
    /// 区間の転倒数。
    inv: u64,
}

impl MoSol for Inversions {
    type Ans = u64;
    // 添字は 0..=10^5 の範囲に収まる。2^17 = 131072 > 10^5。
    const MAX_INDEX_POW2: usize = 17;
    fn add_l(&mut self, l_idx: usize) {
        let r = self.rank[l_idx];
        // 自分より小さい要素が、すべて自分の右にある。
        self.inv += self.bit.sum(r);
        self.bit.add(r);
        self.len += 1;
    }
    fn add_r(&mut self, r_idx: usize) {
        let r = self.rank[r_idx];
        // 自分より大きい要素が、すべて自分の左にある。同じ値は転倒に数えない。
        self.inv += self.len - self.bit.sum(r + 1);
        self.bit.add(r);
        self.len += 1;
    }
    fn del_l(&mut self, l_idx: usize) {
        let r = self.rank[l_idx];
        // 先に取り除いてから、残りとの組を打ち消す。
        self.bit.sub(r);
        self.len -= 1;
        self.inv -= self.bit.sum(r);
    }
    fn del_r(&mut self, r_idx: usize) {
        let r = self.rank[r_idx];
        self.bit.sub(r);
        self.len -= 1;
        self.inv -= self.len - self.bit.sum(r + 1);
    }
    fn solve(&mut self) -> Self::Ans {
        self.inv
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
    let rank = a.iter().map(|v| vals.binary_search(v).unwrap()).collect::<Vec<_>>();

    let mut state = Inversions { rank, bit: Fenwick::new(vals.len()), len: 0, inv: 0 };

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
