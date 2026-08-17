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

pub struct Fibonacci<T>(pub T, pub T);

impl<T: std::ops::AddAssign + Clone> Iterator for Fibonacci<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0 += self.1.clone();
        std::mem::swap(&mut self.0, &mut self.1);
        Some(self.0.clone())
    }
}

#[cfg(test)]
mod tests;
