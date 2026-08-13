pub struct KMP<'pattern, T> {
    pattern: &'pattern [T],
    lps: Box<[usize]>, // Longest Prefix Suffix array
}
impl<'pattern, T: Eq> KMP<'pattern, T> {
    pub fn new(pattern: &'pattern [T]) -> Self {
        let lps = Self::build_lps(pattern);
        KMP { pattern, lps }
    }
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
    pub fn search<'kmp, 'src>(&'kmp self, text: &'src [T]) -> KMPIter<'kmp, 'pattern, 'src, T> {
        KMPIter {
            kmp: self,
            text,
            text_index: 0,
            pattern_index: 0,
        }
    }
}
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
