// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1471

use proconio::{input, marker::Bytes};
use std::io::{BufWriter, Write};

use mo::{Mo, MoSol};

/// 区間に含まれる英小文字それぞれの個数。
struct LetterCount {
    /// `b'a'` を `0` とした文字列。
    s: Vec<usize>,
    cnt: [usize; 26],
}

impl LetterCount {
    fn add(&mut self, i: usize) {
        self.cnt[self.s[i]] += 1;
    }
    fn del(&mut self, i: usize) {
        self.cnt[self.s[i]] -= 1;
    }
}

impl MoSol for LetterCount {
    /// 区間内の文字の個数をそのまま返す。
    ///
    /// 部分文字列を並べ替えた辞書順最小は「a から順に個数だけ並べた文字列」なので、
    /// その `x` 文字目は個数の表があれば決まる。
    /// `solve` にクエリごとの `x` を渡す方法はないため、答えを受け取ってから使う。
    type Ans = [usize; 26];
    // 添字は 0..=10^4 の範囲に収まる。2^14 = 16384 > 10^4。
    const MAX_INDEX_POW2: usize = 14;
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
        self.cnt
    }
}

fn main() {
    input! {
        _n: usize,
        q: usize,
        s: Bytes,
        queries: [(usize, usize, usize); q],
    }

    let mut mo = Mo::new();
    for &(l, r, _) in &queries {
        // 入力は 1-indexed の閉区間、Mo は 0-indexed の半開区間。
        mo.push(l - 1, r);
    }
    let mut state = LetterCount {
        s: s.iter().map(|&b| usize::from(b - b'a')).collect(),
        cnt: [0; 26],
    };
    let ans = mo.execute(&mut state);

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for (&(_, _, x), cnt) in queries.iter().zip(ans.iter()) {
        // 小さい文字から個数を削っていき、x 文字目に来る文字を探す。
        let mut rest = x;
        for (c, &k) in cnt.iter().enumerate() {
            if rest <= k {
                out.write_all(&[b'a' + c as u8]).unwrap();
                break;
            }
            rest -= k;
        }
        out.write_all(b"\n").unwrap();
    }
}
