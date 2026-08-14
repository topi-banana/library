// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/2298

use proconio::{input, marker::Bytes};
use std::io::Write as _;

use kmp::KMP;

/// 探す文字列。9 文字すべてが相異なる。
const PATTERN: &[u8] = b"yukicoder";

fn main() {
    input! { s: Bytes }

    // PATTERN は全文字が相異なるので LPS は全て 0 になり、出現同士が重なることはない。
    // よって隣り合う出現位置の差は必ず 9 以上で、差がちょうど 9 のときだけ
    // 2 つの出現が隙間なく繋がっている。
    // 求める K は「差が 9 で繋がった出現の並び」の最長の長さである。
    //
    // 1 つも出現しなければ K = 0。空文字列 (K = 0 回の繰り返し) は
    // 任意の S の部分文字列なので、これがそのまま答えになる (サンプル 2)。
    let kmp = KMP::new(PATTERN);
    let mut ans = 0;
    let mut run = 0;
    let mut prev = None;
    for i in kmp.search(&s) {
        // search は位置を昇順に返すので prev < i であり、引き算は溢れない。
        run = match prev {
            Some(p) if i - p == PATTERN.len() => run + 1,
            _ => 1,
        };
        ans = ans.max(run);
        prev = Some(i);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{ans}").unwrap();
}
