// competitive-verifier: PROBLEM https://judge.yosupo.jp/problem/range_affine_range_sum

use proconio::input;
use std::io::{BufWriter, Write};

use avl_tree::{Action, LazySegmentTree, Map, Monoid};

const MOD: u64 = 998_244_353;

/// 区間アフィン・区間和
enum AffineSum {}

impl Monoid for AffineSum {
    /// (和, 要素数)
    type S = (u64, u64);

    fn op(a: &(u64, u64), b: &(u64, u64)) -> (u64, u64) {
        ((a.0 + b.0) % MOD, a.1 + b.1)
    }

    fn identity() -> (u64, u64) {
        (0, 0)
    }
}

impl Action for AffineSum {
    /// x -> a * x + b
    type F = (u64, u64);

    fn mapping(f: &(u64, u64), s: &(u64, u64)) -> (u64, u64) {
        ((f.0 * s.0 + f.1 * s.1) % MOD, s.1)
    }

    fn composition(f: &(u64, u64), g: &(u64, u64)) -> (u64, u64) {
        ((f.0 * g.0) % MOD, (f.0 * g.1 + f.1) % MOD)
    }

    fn id() -> (u64, u64) {
        (1, 0)
    }
}

fn main() {
    input! {
        n: usize,
        q: usize,
        a: [u64; n],
    }

    // キーを index にした遅延セグメント木。
    let mut seg: Map<LazySegmentTree<usize, AffineSum>> = Map::new();
    for (i, &value) in a.iter().enumerate() {
        seg.put(i, (value % MOD, 1));
    }

    let stdout = std::io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for _ in 0..q {
        input! { t: u8 }
        if t == 0 {
            input! { l: usize, r: usize, b: u64, c: u64 }
            seg.apply(l..r, (b % MOD, c % MOD));
        } else {
            input! { l: usize, r: usize }
            writeln!(out, "{}", seg.prod(l..r).0).unwrap();
        }
    }
}
