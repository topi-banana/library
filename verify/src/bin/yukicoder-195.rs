// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/195

use proconio::input;
use std::io::Write;

use fibonacci::Fibonacci;

/// 見る必要のある項数。
///
/// `A, B >= 1` なので `F_{A,B}(k) >= F(k)` であり、`F(45) = 1134903170 > 10^9`。
/// つまり `10^9` 以下の値が入りうるのは第 44 項までで、45 項見れば足りる。
const LIMIT: usize = 45;

/// 第 `k` 項を `F_{A,B}(k) = c_k · A + d_k · B` と書いたときの係数 `(c_k, d_k)`。
///
/// `(c_1, d_1) = (1, 0)`、`(c_2, d_2) = (0, 1)` で、以降は数列と同じ漸化式で伸びる。
/// 返り値の `k` 番目 (0-indexed) が第 `k + 1` 項の係数。
///
/// 係数から組む連立方程式は行列式も分子も負になりうるので符号付きで持つ。
/// 入力が `10^9` 以下なら絶対値は高々 `10^9 · F(44) ≈ 7.0 · 10^17` で `i64` に収まる。
fn coefficients() -> Vec<(i64, i64)> {
    let mut coef = Vec::with_capacity(LIMIT);
    let (mut c, mut d) = (1, 0);
    for _ in 0..LIMIT {
        coef.push((c, d));
        (c, d) = (d, c + d);
    }
    coef
}

fn main() {
    input! { mut vals: [u64; 3] }

    // 同じ値は数列に 1 回現れれば足りるので、重複を潰しておく。
    vals.sort_unstable();
    vals.dedup();

    let coef = coefficients();

    let ans = if let [u, w, ..] = vals[..] {
        // 相異なる 2 値は必ず別の添字に入る。
        // 同じ値が 2 回現れるのは A = B のときの第 1, 2 項だけで、
        // 第 2 項以降は A >= 1 より狭義単調増加だからである。
        //
        // そこで u, w の添字を i, j (i != j) と仮定すると
        //   c_i·A + d_i·B = u
        //   c_j·A + d_j·B = w
        // という連立方程式になる。行列式は c_i·d_j - c_j·d_i = ±F(|i - j|) で、
        // i != j なら 0 にならないので (A, B) が一意に定まる。
        //
        // 条件を満たす (A, B) が存在すれば必ずどれかの (i, j) に対応するから、
        // 全通り試して残った中の最小を取れば答えになる。
        let mut best = None;
        for (i, &(ci, di)) in coef.iter().enumerate() {
            for (j, &(cj, dj)) in coef.iter().enumerate() {
                if i == j {
                    continue;
                }
                let det = ci * dj - cj * di;
                let na = u as i64 * dj - w as i64 * di;
                let nb = w as i64 * ci - u as i64 * cj;
                // 割り切れなければ A, B が整数にならない。
                // 余りが 0 でありさえすれば、符号によらず商は厳密な値になる。
                if na % det != 0 || nb % det != 0 {
                    continue;
                }
                let (a, b) = (na / det, nb / det);
                if a < 1 || b < 1 {
                    continue;
                }
                // ここまで来た (a, b) は 10^9 以下の項として現れる以上どちらも 10^9 以下で、
                // 第 45 項でも (F(43) + F(44)) · 10^9 ≈ 1.1 · 10^18 なので u64 で足りる。
                let (a, b) = (a as u64, b as u64);

                // i, j は仮定でしかないので、残りの値も含めて実際に並べて確かめる。
                // Fibonacci が最初に返すのは b (= 第 2 項) なので、a を頭に付けて添字を揃える。
                let t = std::iter::once(a).chain(Fibonacci(a, b)).take(LIMIT).collect::<Vec<_>>();
                if vals.iter().all(|v| t.contains(v)) && best.is_none_or(|cur| (a, b) < cur) {
                    best = Some((a, b));
                }
            }
        }
        best
    } else {
        // 値が 1 種類 v だけなら、(A, B) = (1, v) が必ず v を含むので A = 1 で確定。
        // あとは v = c_k + d_k·B を満たす最小の B を探せばよい。
        // d_k >= 1 かつ v > c_k なら、割り切れたときの商は自動的に 1 以上になる。
        let v = vals[0] as i64;
        coef.iter()
            .filter(|&&(c, d)| d > 0 && v > c && (v - c) % d == 0)
            .map(|&(c, d)| (v - c) / d)
            .min()
            .map(|b| (1, b as u64))
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match ans {
        Some((a, b)) => writeln!(out, "{a} {b}").unwrap(),
        None => writeln!(out, "-1").unwrap(),
    }
}
