//! 空白区切りの入力を読み取るスキャナ。
//!
//! 入力全体を一度メモリに読み込んでからトークンに切り出すため、
//! 1 行ずつ読む実装よりも高速に動作する。
//!
//! ```
//! use scanner::Scanner;
//!
//! let mut sc = Scanner::new("3 1 4 1 5".as_bytes()).unwrap();
//! let n: usize = sc.read();
//! let a: Vec<i64> = sc.read_vec(n);
//! assert_eq!(a, vec![1, 4, 1]);
//! ```

use std::fmt::Debug;
use std::io::{self, Read};
use std::str::FromStr;

/// 空白区切りのトークンを順番に読み出すスキャナ。
pub struct Scanner {
    buf: String,
    pos: usize,
}

impl Scanner {
    /// `reader` の内容をすべて読み込んでスキャナを作る。
    ///
    /// # Errors
    ///
    /// `reader` からの読み込みに失敗した場合、その [`io::Error`] を返す。
    pub fn new<R: Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = String::new();
        reader.read_to_string(&mut buf)?;
        Ok(Self { buf, pos: 0 })
    }

    /// 標準入力の内容をすべて読み込んでスキャナを作る。
    ///
    /// # Errors
    ///
    /// 標準入力からの読み込みに失敗した場合、その [`io::Error`] を返す。
    pub fn from_stdin() -> io::Result<Self> {
        Self::new(io::stdin().lock())
    }

    /// 次のトークンを返す。入力が尽きている場合は [`None`] を返す。
    pub fn next_token(&mut self) -> Option<&str> {
        let bytes = self.buf.as_bytes();
        let mut start = self.pos;
        while start < bytes.len() && bytes[start].is_ascii_whitespace() {
            start += 1;
        }
        if start == bytes.len() {
            self.pos = start;
            return None;
        }
        let mut end = start;
        while end < bytes.len() && !bytes[end].is_ascii_whitespace() {
            end += 1;
        }
        self.pos = end;
        Some(&self.buf[start..end])
    }

    /// 次のトークンを `T` としてパースして返す。
    ///
    /// # Panics
    ///
    /// 入力が尽きている場合、またはパースに失敗した場合にパニックする。
    pub fn read<T>(&mut self) -> T
    where
        T: FromStr,
        T::Err: Debug,
    {
        let token = self.next_token().expect("入力が尽きています");
        token.parse().expect("トークンのパースに失敗しました")
    }

    /// 次の `n` 個のトークンを `T` としてパースして返す。
    ///
    /// # Panics
    ///
    /// 入力が尽きている場合、またはパースに失敗した場合にパニックする。
    pub fn read_vec<T>(&mut self, n: usize) -> Vec<T>
    where
        T: FromStr,
        T::Err: Debug,
    {
        (0..n).map(|_| self.read()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;

    #[test]
    fn read_mixed_types() {
        let mut sc = Scanner::new("42 -1 3.5 hello".as_bytes()).unwrap();
        assert_eq!(sc.read::<u32>(), 42);
        assert_eq!(sc.read::<i64>(), -1);
        assert!((sc.read::<f64>() - 3.5).abs() < 1e-12);
        assert_eq!(sc.read::<String>(), "hello");
    }

    #[test]
    fn skips_any_whitespace() {
        let mut sc = Scanner::new("  1\t2\r\n3\n\n 4 \n".as_bytes()).unwrap();
        assert_eq!(sc.read_vec::<i32>(4), vec![1, 2, 3, 4]);
        assert_eq!(sc.next_token(), None);
    }

    #[test]
    fn empty_input_yields_nothing() {
        let mut sc = Scanner::new("   \n ".as_bytes()).unwrap();
        assert_eq!(sc.next_token(), None);
    }
}
