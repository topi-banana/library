// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/927

use proconio::{input, marker::Bytes};
use std::io::Write;

use permutation::prev_permutation;

fn main() {
    input! { x: Bytes }

    // 桁を降順に並べたものが最大の Y。そこから 1 つ戻せば 2 番目に大きい並びになる。
    let mut d = x;
    d.sort_unstable();
    d.reverse();

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // 戻せないのは全ての桁が同じとき。先頭が '0' になるのは桁が {c, 0, 0, ..., 0} の
    // ときだけで、このとき先頭が '0' でない並びは最大の 1 通りしかない。
    // どちらも 2 番目に大きい Y が存在しないので -1 を返す。
    if !prev_permutation(&mut d) || d[0] == b'0' {
        out.write_all(b"-1\n").unwrap();
    } else {
        d.push(b'\n');
        out.write_all(&d).unwrap();
    }
}
