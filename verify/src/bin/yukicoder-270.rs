// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/270

use proconio::input;
use std::io::Write;

use permutation::next_permutation;

fn dist(a: &[u64], b: &[u64]) -> u64 {
    a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).sum()
}

fn main() {
    input! {
        n: usize,
        k: usize,
        p: [u64; n],
        b: [u64; n],
    }

    // N K = 10^5 なので dist を毎回計算し直すと間に合わない。
    // next_permutation が書き換えるのは pivot 以降だけなので、その範囲だけ足し引きする。
    // 書き換わる長さは全体で均すと 1 回あたり定数なので、全体で O(N + K) になる。
    let mut a = p;
    let mut cur = dist(&a, &b);
    let mut ans = 0u64;
    for _ in 0..k {
        ans += cur;

        // next_permutation が書き換える範囲の先頭。最後の置換なら見つからず 0 になる。
        let start = a.windows(2).rposition(|w| w[0] < w[1]).unwrap_or(0);
        cur -= dist(&a[start..], &b[start..]);
        if !next_permutation(&mut a) {
            // 最後の置換 (降順) なので、反転して最初の置換 (昇順) へ折り返す。
            a.reverse();
        }
        cur += dist(&a[start..], &b[start..]);
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{ans}").unwrap();
}
