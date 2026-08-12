// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1469

use proconio::{input, marker::Bytes};
use std::io::Write;

use rle::rle;

fn main() {
    input! { s:Bytes }
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for (b, _) in rle(&mut s.into_iter()) {
        out.write(&[b]).unwrap();
    }
    out.write(b"\n").unwrap();
}
