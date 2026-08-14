// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/786
// この問題の配布テストケースは想定出力に末尾改行がないため、既定の完全一致比較では
// WA になる。ERROR を指定すると比較がトークン単位になり、改行の差が無視される。
// 値どうしはまず文字列一致で判定されるので、0 を渡せば比較は厳密なまま。
// competitive-verifier: ERROR 0

use proconio::input;
use std::io::Write;

use fibonacci::fibonacci_matrix_pow;

fn main() {
    input! {
        n: usize,
    }

    // n 段目に来る直前は n-1 段目か n-2 段目なので、昇り方の総数 f(n) は
    // f(n) = f(n-1) + f(n-2) を満たす。f(1) = 1、f(2) = 2 なので f(n) = F(n + 1)。
    // 必要なのは 1 項だけなので、O(log n) の行列累乗で求める。
    //
    // N <= 50 で F(51) = 20365011074 なので、32 bit 整数には収まらない。
    let ans = fibonacci_matrix_pow(n + 1);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "{ans}").unwrap();
}
