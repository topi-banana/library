// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/static_range_mode_query

use proconio::input;
use std::io::{BufWriter, Write};

use mo::{Mo, MoSol};

/// 区間の最頻値とその出現回数。
///
/// 順位を個数の昇順に並べた配列 `order` を保ち、その末尾を最頻値とする。
/// 個数が同じ順位は `order` 上で連続した 1 ブロックを占めるので、
/// 「ブロックの端と入れ替えてから境界を 1 ずらす」だけで昇順を保てる。
/// 個数の最大値だけを追いかける方法だと、
/// 最大値を持つ値が複数あるときに「どれが最頻値か」を見失う。
struct Mode {
    /// 昇順に並べた、重複を除いた値。
    vals: Vec<i64>,
    /// 元の列の各要素の、`vals` での順位。
    rank: Vec<usize>,
    /// 順位ごとの個数。
    cnt: Vec<usize>,
    /// 個数の昇順に並べた順位。
    order: Vec<usize>,
    /// `order` 内での順位の位置。`order[pos[r]] == r`。
    pos: Vec<usize>,
    /// 個数 `c` の順位が `order` 上で占めるブロックの先頭。
    start: Vec<usize>,
}

impl Mode {
    fn add(&mut self, i: usize) {
        let r = self.rank[i];
        let c = self.cnt[r];
        // 個数 c のブロックの末尾へ移してから、そこを個数 c + 1 のブロックに取り込む。
        let tail = self.start[c + 1] - 1;
        self.move_to(r, tail);
        self.start[c + 1] = tail;
        self.cnt[r] = c + 1;
    }
    fn del(&mut self, i: usize) {
        let r = self.rank[i];
        let c = self.cnt[r];
        // 個数 c のブロックの先頭へ移してから、そこを個数 c - 1 のブロックへ渡す。
        let head = self.start[c];
        self.move_to(r, head);
        self.start[c] = head + 1;
        self.cnt[r] = c - 1;
    }
    /// `order` 上で順位 `r` を位置 `p` へ動かす。今 `p` にいる順位とは入れ替える。
    fn move_to(&mut self, r: usize, p: usize) {
        let q = self.pos[r];
        let s = self.order[p];
        self.order.swap(p, q);
        self.pos[r] = p;
        self.pos[s] = q;
    }
}

impl MoSol for Mode {
    /// 最頻値とその出現回数。
    type Ans = (i64, usize);
    // 添字は 0..=10^5 の範囲に収まる。2^17 = 131072 > 10^5。
    const MAX_INDEX_POW2: usize = 17;
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
        let r = self.order[self.order.len() - 1];
        (self.vals[r], self.cnt[r])
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

    let m = vals.len();
    // はじめはどの値も 0 個なので、全体が個数 0 のブロック 1 つになる。
    let mut start = vec![m; n + 1];
    start[0] = 0;
    let mut state =
        Mode { vals, rank, cnt: vec![0; m], order: (0..m).collect(), pos: (0..m).collect(), start };

    let mut mo = Mo::new();
    for (l, r) in queries {
        // 入力は 0-indexed の半開区間なので、そのまま渡せる。
        mo.push(l, r);
    }
    let ans = mo.execute(&mut state);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for &(v, c) in ans.iter() {
        writeln!(out, "{v} {c}").unwrap();
    }
}
