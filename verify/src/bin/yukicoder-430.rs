// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/430

use proconio::{input, marker::Bytes};
use std::io::Write as _;

use kmp::KMP;

fn main() {
    input! {
        s: Bytes,
        m: usize,
        c: [Bytes; m],
    }

    // C_i ごとに検索器を作り直して S 全体を走査する。
    // |S| <= 5 * 10^4, M <= 5000 なので全体で 2.5 * 10^8 歩ほどになるが、
    // KMP は S 側の添字を巻き戻さないので 1 歩あたりの仕事は定数である。
    //
    // 重なり合う出現もそれぞれ数える。S = "AAAA", C_i = "AA" なら
    // 位置 0, 1, 2 の 3 個で、これは問題文の数え方 (サンプル 3) と一致する。
    let ans: usize = c.iter().map(|p| KMP::new(p).search(&s).count()).sum();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{ans}").unwrap();
}
