//! フィボナッチ数列。
//!
//! `F(0) = 0`、`F(1) = 1`、`F(n) = F(n - 1) + F(n - 2)` で定まる数列を [`u128`] で扱う。
//!
//! 第 `n` 項だけを `O(log n)` で求める [`fibonacci_matrix_pow`] と、
//! 先頭から順に列挙するイテレータ [`Fibonacci`] がある。
//!
//! ```
//! use fibonacci::{Fibonacci, fibonacci_matrix_pow};
//!
//! assert_eq!(fibonacci_matrix_pow(0), 0);
//! assert_eq!(fibonacci_matrix_pow(10), 55);
//!
//! let fib = Fibonacci { a: 0, b: 1 };
//! assert_eq!(fib.take(5).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5]);
//! ```
//!
//! いずれも [`u128`] の範囲を超えたときの扱いはビルドの
//! オーバーフロー検査に従う。詳細は各アイテムの `# Panics` を参照。

/// 行列累乗法で第 `n` 項 `F(n)` を求める。
///
/// `[[1, 1], [1, 0]]` の `n - 1` 乗の左上成分が `F(n)` に等しいことを使い、
/// 繰り返し二乗法で `O(log n)` 回の行列積に落とす。
/// `F(0) = 0`、`F(1) = F(2) = 1` の添字で、`fibonacci_matrix_pow(0)` は `0` を返す。
///
/// 先頭から順に列挙したいときは [`Fibonacci`] を使う。
/// 第 `n` 項までまとめて必要なら、そちらのほうが全体で `O(n)` と速い。
///
/// # Panics
///
/// 途中の計算が [`u128`] に収まらないとき、オーバーフロー検査が有効なビルド
/// (debug ビルド) では panic する。あふれるのは `F(n)` 自体が収まらないときだけなので、
/// `n <= 186` なら panic しない。
///
/// 検査が無効なビルド (release ビルド) では `2^128` を法とした値になる。
///
/// # Examples
///
/// ```
/// use fibonacci::fibonacci_matrix_pow;
///
/// assert_eq!(fibonacci_matrix_pow(0), 0);
/// assert_eq!(fibonacci_matrix_pow(1), 1);
/// assert_eq!(fibonacci_matrix_pow(2), 1);
/// assert_eq!(fibonacci_matrix_pow(50), 12586269025);
/// ```
pub fn fibonacci_matrix_pow(n: usize) -> u128 {
    fn matrix_multiply(a: [[u128; 2]; 2], b: [[u128; 2]; 2]) -> [[u128; 2]; 2] {
        [
            [a[0][0] * b[0][0] + a[0][1] * b[1][0], a[0][0] * b[0][1] + a[0][1] * b[1][1]],
            [a[1][0] * b[0][0] + a[1][1] * b[1][0], a[1][0] * b[0][1] + a[1][1] * b[1][1]],
        ]
    }
    fn matrix_power(matrix: [[u128; 2]; 2], mut n: usize) -> [[u128; 2]; 2] {
        let mut result = [[1, 0], [0, 1]];
        let mut base = matrix;
        while n > 0 {
            if n % 2 == 1 {
                result = matrix_multiply(result, base);
            }
            n /= 2;
            // 最後の 1 周で二乗しても結果には使われず、あふれるだけなので飛ばす。
            if n > 0 {
                base = matrix_multiply(base, base);
            }
        }
        result
    }
    if n == 0 {
        return 0;
    }
    let base = [[1, 1], [1, 0]];
    let result = matrix_power(base, n - 1);
    result[0][0]
}

/// フィボナッチ数列を第 1 項から順に返す無限イテレータ。
///
/// 直前の 2 項だけを持ち、[`next`](Iterator::next) 1 回あたり `O(1)` で進む。
/// 第 `n` 項まで取り出すと全体で `O(n)`。
/// [`next`](Iterator::next) が [`None`] を返すことはないので、
/// [`take`](Iterator::take) などで打ち切って使う。
///
/// 直前に返した項を `F(k)` として、`a == F(k)`、`b == F(k + 1)` が保たれる。
/// 初期値 `Fibonacci { a: 0, b: 1 }` は `k = 0` にあたり、
/// 最初の [`next`](Iterator::next) は `F(1) = 1` を返す。
/// 途中の項から始めたいときは、この不変条件を満たす値を直接入れればよい。
///
/// # Panics
///
/// [`next`](Iterator::next) は返す項の 1 つ先まで計算するため、
/// オーバーフロー検査が有効なビルド (debug ビルド) では
/// `F(187)` を作る `F(186)` の生成時に panic する。取り出せるのは `F(185)` まで。
///
/// 検査が無効なビルド (release ビルド) では `2^128` を法とした値になり、
/// `F(186)` までは正しい値が返る。
///
/// # Examples
///
/// ```
/// use fibonacci::Fibonacci;
///
/// let fib = Fibonacci { a: 0, b: 1 };
/// assert_eq!(fib.take(10).collect::<Vec<_>>(), vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]);
///
/// // 100 を超える最初の項
/// let fib = Fibonacci { a: 0, b: 1 };
/// assert_eq!(fib.take_while(|&f| f <= 100).last(), Some(89));
///
/// // F(10) = 55、F(11) = 89 から再開する
/// let fib = Fibonacci { a: 55, b: 89 };
/// assert_eq!(fib.take(3).collect::<Vec<_>>(), vec![89, 144, 233]);
/// ```
pub struct Fibonacci {
    /// 直前に返した項 `F(k)`。
    pub a: u128,
    /// 次に返す項 `F(k + 1)`。
    pub b: u128,
}
impl Iterator for Fibonacci {
    type Item = u128;

    fn next(&mut self) -> Option<Self::Item> {
        let na = self.b;
        let nb = self.a + self.b;
        self.a = na;
        self.b = nb;
        Some(self.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_case_set() -> Vec<usize> {
        vec![1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
    }

    #[test]
    fn test_fibonacci_matrix_pow() {
        for (i, n) in test_case_set().into_iter().enumerate() {
            assert_eq!(fibonacci_matrix_pow(i + 1), n as u128);
        }
    }

    #[test]
    fn test_fibonacci_iter() {
        let mut fib = Fibonacci { a: 0, b: 1 };
        for n in test_case_set() {
            assert_eq!(fib.next().unwrap(), n as u128);
        }
    }

    // u128 に収まる最大の項。行列を余分に二乗していると、ここに届く前に panic する。
    #[test]
    fn test_fibonacci_matrix_pow_upper_bound() {
        assert_eq!(fibonacci_matrix_pow(186), 332825110087067562321196029789634457848);
    }

    // イテレータが panic せずに返せる F(185) までを突き合わせる。
    #[test]
    fn test_fibonacci_iter_matches_matrix_pow() {
        let fib = Fibonacci { a: 0, b: 1 };
        for (i, f) in fib.take(185).enumerate() {
            assert_eq!(f, fibonacci_matrix_pow(i + 1));
        }
    }
}
