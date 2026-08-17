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
    let mut fib = Fibonacci(0, 1);
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
    let fib = Fibonacci(0, 1);
    for (i, f) in fib.take(185).enumerate() {
        assert_eq!(f, fibonacci_matrix_pow(i + 1));
    }
}
