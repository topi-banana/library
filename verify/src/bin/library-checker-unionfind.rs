// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/unionfind

use std::io::{self, BufWriter, Write};

use dsu::Dsu;
use scanner::Scanner;

fn main() -> io::Result<()> {
    let mut sc = Scanner::from_stdin()?;
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());

    let n: usize = sc.read();
    let q: usize = sc.read();
    let mut dsu = Dsu::new(n);

    for _ in 0..q {
        let t: u8 = sc.read();
        let u: usize = sc.read();
        let v: usize = sc.read();
        if t == 0 {
            dsu.merge(u, v);
        } else {
            writeln!(out, "{}", u8::from(dsu.same(u, v)))?;
        }
    }

    out.flush()
}
