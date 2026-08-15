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

pub struct Fibonacci {
    pub a: u128,
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
