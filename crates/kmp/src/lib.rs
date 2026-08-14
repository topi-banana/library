//! Knuth-Morris-Pratt 法による列の検索。
//!
//! パターンがテキストのどこに現れるかを、すべて列挙する。
//! パターンの長さを `m`、テキストの長さを `n` として、
//! 前処理 `O(m)` 時間・メモリ、検索は全体を列挙して `O(n)` 時間。
//!
//! 要素の型に必要な境界は [`Eq`] だけなので、文字列に限らず任意の列に使える。
//!
//! ```
//! use kmp::KMP;
//!
//! let kmp = KMP::new(b"aba");
//! let hits: Vec<_> = kmp.search(b"abababa").collect();
//! // 重なり合う出現もそれぞれ数える。
//! assert_eq!(hits, vec![0, 2, 4]);
//! ```

/// パターンを前処理して保持する検索器。
///
/// [`KMP::new`] で作り、[`KMP::search`] でテキストごとに検索する。
/// 前処理はパターンだけに依存するので、1 つの `KMP` を複数のテキストに使い回せる。
///
/// パターンは借用したまま持つため、`KMP` はパターンより長生きできない。
pub struct KMP<'pattern, T> {
    pattern: &'pattern [T],
    lps: Box<[usize]>, // Longest Prefix Suffix array
}
impl<'pattern, T: Eq> KMP<'pattern, T> {
    /// パターンを前処理して検索器を作る。
    ///
    /// パターンの長さを `m` として `O(m)` 時間・メモリ。
    /// 空のパターンも受け付けるが、[`KMP::search`] は 1 つも位置を返さない。
    ///
    /// ```
    /// use kmp::KMP;
    ///
    /// let pattern = [1, 2, 1];
    /// let kmp = KMP::new(&pattern);
    /// assert_eq!(kmp.search(&[1, 2, 1, 2, 1]).collect::<Vec<_>>(), vec![0, 2]);
    /// ```
    pub fn new(pattern: &'pattern [T]) -> Self {
        let lps = Self::build_lps(pattern);
        KMP { pattern, lps }
    }
    /// `lps[i]` = `pattern[..=i]` の接頭辞と接尾辞が一致する最大の長さ (ただし全体を除く)。
    ///
    /// 照合が途中で失敗したとき、パターン側の添字をこの長さまで戻せば、
    /// テキスト側を巻き戻さずに照合を続けられる。
    fn build_lps(pattern: &[T]) -> Box<[usize]> {
        let mut lps = vec![0; pattern.len()];
        let mut length = 0;
        let mut i = 1;
        while i < pattern.len() {
            if pattern[i] == pattern[length] {
                length += 1;
                lps[i] = length;
                i += 1;
            } else if length != 0 {
                length = lps[length - 1];
            } else {
                lps[i] = 0;
                i += 1;
            }
        }
        lps.into_boxed_slice()
    }
    /// `text` の中でパターンが現れる開始位置を、昇順に列挙する。
    ///
    /// テキストの長さを `n` として、最後まで列挙して `O(n)` 時間。
    /// 重なり合う出現もそれぞれ数える。
    /// パターンが空のとき、およびパターンがテキストより長いときは 1 つも返さない。
    ///
    /// ```
    /// use kmp::KMP;
    ///
    /// let kmp = KMP::new(b"aa");
    /// assert_eq!(kmp.search(b"aaaa").collect::<Vec<_>>(), vec![0, 1, 2]);
    /// assert_eq!(kmp.search(b"abab").collect::<Vec<_>>(), vec![]);
    ///
    /// // 空のパターンは「どこにでも現れる」とは扱わない。
    /// let empty = KMP::new(b"");
    /// assert_eq!(empty.search(b"abc").count(), 0);
    /// ```
    pub fn search<'kmp, 'src>(&'kmp self, text: &'src [T]) -> KMPIter<'kmp, 'pattern, 'src, T> {
        KMPIter {
            kmp: self,
            text,
            text_index: 0,
            pattern_index: 0,
        }
    }
}
/// [`KMP::search`] が返すイテレータ。
///
/// 次の出現位置が確定するまでしかテキストを走査しないので、
/// 「最初の 1 つだけ欲しい」用途ではテキスト全体を見ずに済む。
///
/// ```
/// use kmp::KMP;
///
/// let kmp = KMP::new(b"ab");
/// assert_eq!(kmp.search(b"xxabxxab").next(), Some(2));
/// ```
pub struct KMPIter<'kmp, 'pattern, 'src, T> {
    kmp: &'kmp KMP<'pattern, T>,
    text: &'src [T],
    text_index: usize,
    pattern_index: usize,
}
impl<'kmp, 'pattern, 'src, T: PartialEq> Iterator for KMPIter<'kmp, 'pattern, 'src, T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        if self.text.len() < self.kmp.pattern.len() || self.kmp.pattern.is_empty() {
            return None;
        }
        while self.text_index < self.text.len() {
            if self.kmp.pattern[self.pattern_index] == self.text[self.text_index] {
                self.pattern_index += 1;
                self.text_index += 1;
            }
            if self.pattern_index == self.kmp.pattern.len() {
                let res = self.text_index - self.pattern_index;
                self.pattern_index = self.kmp.lps[self.pattern_index - 1];
                return Some(res);
            } else if self.text_index < self.text.len()
                && self.kmp.pattern[self.pattern_index] != self.text[self.text_index]
            {
                if self.pattern_index != 0 {
                    self.pattern_index = self.kmp.lps[self.pattern_index - 1];
                } else {
                    self.text_index += 1;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kmp_search() {
        let pattern = vec!['a', 'b', 'c'];
        let kmp = KMP::new(&pattern);
        let text = vec!['a', 'b', 'a', 'b', 'c', 'a', 'b', 'c', 'a'];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![2, 5]);
    }

    #[test]
    fn test_no_match() {
        let pattern = vec!['x', 'y', 'z'];
        let kmp = KMP::new(&pattern);
        let text = vec!['a', 'b', 'c', 'd', 'e'];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_empty_pattern() {
        let pattern: Vec<char> = vec![];
        let kmp = KMP::new(&pattern);
        let text = vec!['a', 'b', 'c'];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_empty_text() {
        let pattern = vec!['a'];
        let kmp = KMP::new(&pattern);
        let text: Vec<char> = vec![];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_empty_empty() {
        let pattern = vec![];
        let kmp = KMP::new(&pattern);
        let text: Vec<char> = vec![];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_bigger_pattern() {
        let pattern = vec!['a', 'b', 'c', 'd', 'e'];
        let kmp = KMP::new(&pattern);
        let text: Vec<char> = vec!['a', 'b'];
        let result: Vec<_> = kmp.search(&text).collect();
        assert_eq!(result, vec![]);
    }
}
