// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/point_add_range_sum

use std::io::{self, BufWriter, Write};

use scanner::Scanner;
use segtree::{Additive, Segtree};

fn main() -> io::Result<()> {
    let mut sc = Scanner::from_stdin()?;
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let n: usize = sc.read();
    let q: usize = sc.read();
    let a: Vec<i64> = sc.read_vec(n);
    let mut seg: Segtree<Additive<i64>> = a.into();

    for _ in 0..q {
        let t: u8 = sc.read();
        if t == 0 {
            let p: usize = sc.read();
            let x: i64 = sc.read();
            seg.set(p, seg.get(p) + x);
        } else {
            let l: usize = sc.read();
            let r: usize = sc.read();
            writeln!(out, "{}", seg.prod(l..r))?;
        }
    }

    out.flush()
}
