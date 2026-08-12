// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1469

use proconio::{input, marker::Bytes};
use std::io::Write;

use rle::Rle;

fn main() {
    input! { s:Bytes }
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (&b, _) in s.iter().rle() {
        out.write_all(&[b]).unwrap();
    }
    out.write_all(b"\n").unwrap();
}
