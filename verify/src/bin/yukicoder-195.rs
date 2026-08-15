// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/195

use proconio::input;
use std::io::Write;

use fibonacci::Fibonacci;

/// 見る必要のある項数。
///
/// `A, B >= 1` なので `F_{A,B}(k) >= F(k)` であり、`F(45) = 1134903170 > 10^9`。
/// つまり `10^9` 以下の値が入りうるのは第 44 項までで、45 項見れば足りる。
const LIMIT: usize = 45;

/// `(A, B)` フィボナッチ数列の第 1 項から第 `LIMIT` 項まで。
fn terms(a: u128, b: u128) -> Vec<u128> {
    // Fibonacci が最初に返すのは b (= F(2)) なので、a を頭に付けて添字を揃える。
    std::iter::once(a).chain(Fibonacci { a, b }).take(LIMIT).collect()
}

/// 第 `k` 項を `F_{A,B}(k) = c_k · A + d_k · B` と書いたときの係数 `(c_k, d_k)`。
///
/// `(c_1, d_1) = (1, 0)`、`(c_2, d_2) = (0, 1)` で、以降は数列と同じ漸化式で伸びる。
/// 返り値の `k` 番目 (0-indexed) が第 `k + 1` 項の係数。
fn coefficients() -> Vec<(i128, i128)> {
    let mut coef = Vec::with_capacity(LIMIT);
    let (mut c, mut d) = (1, 0);
    for _ in 0..LIMIT {
        coef.push((c, d));
        (c, d) = (d, c + d);
    }
    coef
}

fn main() {
    input! {
        x: u64,
        y: u64,
        z: u64,
    }

    // 同じ値は数列に 1 回現れれば足りるので、重複を潰しておく。
    let mut vals = [x, y, z].map(u128::from).to_vec();
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
        let mut best: Option<(u128, u128)> = None;
        for (i, &(ci, di)) in coef.iter().enumerate() {
            for (j, &(cj, dj)) in coef.iter().enumerate() {
                if i == j {
                    continue;
                }
                let det = ci * dj - cj * di;
                let na = u as i128 * dj - w as i128 * di;
                let nb = w as i128 * ci - u as i128 * cj;
                // 割り切れなければ A, B が整数にならない。
                // 余りが 0 でありさえすれば、符号によらず商は厳密な値になる。
                if na % det != 0 || nb % det != 0 {
                    continue;
                }
                let (a, b) = (na / det, nb / det);
                if a < 1 || b < 1 {
                    continue;
                }
                let (a, b) = (a as u128, b as u128);

                // i, j は仮定でしかないので、残りの値も含めて実際に並べて確かめる。
                let t = terms(a, b);
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
        let v = vals[0] as i128;
        coef.iter()
            .filter(|&&(c, d)| d > 0 && v > c && (v - c) % d == 0)
            .map(|&(c, d)| (v - c) / d)
            .min()
            .map(|b| (1, b as u128))
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    match ans {
        Some((a, b)) => writeln!(out, "{a} {b}").unwrap(),
        None => writeln!(out, "-1").unwrap(),
    }
}
