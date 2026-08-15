use std::collections::BTreeMap;

#[derive(Clone, PartialEq, Eq)]
pub struct Ranges<T>(BTreeMap<T, T>);

impl<T> Ranges<T> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&T, &T)> + '_ {
        self.0.iter()
    }
}

impl<T: Ord + Clone> Ranges<T> {
    pub fn insert(&mut self, range: std::ops::Range<T>) {
        let std::ops::Range { mut start, mut end } = range;
        if start >= end {
            return;
        }

        if let Some((s, e)) = self.0.range(..=&start).next_back()
            && *e >= start
        {
            start = s.clone();
            if *e > end {
                end = e.clone();
            }
        }

        let absorbed: Vec<T> = self.0.range(&start..=&end).map(|(s, _)| s.clone()).collect();
        for key in absorbed {
            if let Some(e) = self.0.remove(&key)
                && e > end
            {
                end = e;
            }
        }

        self.0.insert(start, end);
    }

    pub fn remove(&mut self, range: std::ops::Range<T>) {
        let std::ops::Range { start, end } = range;
        if start >= end {
            return;
        }

        let left = self.0.range(..&start).next_back().map(|(s, e)| (s.clone(), e.clone()));
        if let Some((s, e)) = left
            && e > start
        {
            self.0.insert(s, start.clone());
            if e > end {
                self.0.insert(end.clone(), e);
            }
        }

        let hit: Vec<T> = self.0.range(&start..&end).map(|(s, _)| s.clone()).collect();
        for key in hit {
            if let Some(e) = self.0.remove(&key)
                && e > end
            {
                self.0.insert(end.clone(), e);
            }
        }
    }
}

impl<T: Ord> Ranges<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.covering(value).is_some()
    }

    pub fn covering(&self, value: &T) -> Option<(&T, &T)> {
        self.0.range(..=value).next_back().filter(|(_, end)| *end > value)
    }

    pub fn contains_range(&self, range: &std::ops::Range<T>) -> bool {
        if range.start >= range.end {
            return true;
        }
        match self.0.range(..=&range.start).next_back() {
            Some((_, end)) => *end >= range.end,
            None => false,
        }
    }

    pub fn overlaps(&self, range: &std::ops::Range<T>) -> bool {
        if range.start >= range.end {
            return false;
        }
        if let Some((_, end)) = self.0.range(..=&range.start).next_back()
            && *end > range.start
        {
            return true;
        }
        self.0.range(&range.start..&range.end).next().is_some()
    }
}

impl<T> Default for Ranges<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Clone> Extend<std::ops::Range<T>> for Ranges<T> {
    fn extend<I: IntoIterator<Item = std::ops::Range<T>>>(&mut self, iter: I) {
        for range in iter {
            self.insert(range);
        }
    }
}

impl<T: Ord + Clone> FromIterator<std::ops::Range<T>> for Ranges<T> {
    fn from_iter<I: IntoIterator<Item = std::ops::Range<T>>>(iter: I) -> Self {
        let mut ranges = Self::new();
        ranges.extend(iter);
        ranges
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Ranges<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ranges ")?;
        f.debug_set().entries(self.0.iter().map(|(s, e)| DebugRange(s, e))).finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ImmutableRanges<T>(Vec<(T, T)>);

impl<T> ImmutableRanges<T> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_slice(&self) -> &[(T, T)] {
        &self.0
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (T, T)> {
        self.0.iter()
    }
}

impl<T: Ord> ImmutableRanges<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.covering_index(value).is_some()
    }

    pub fn covering(&self, value: &T) -> Option<&(T, T)> {
        self.covering_index(value).map(|i| &self.0[i])
    }

    pub fn covering_index(&self, value: &T) -> Option<usize> {
        let idx = self.0.partition_point(|(start, _)| start <= value);
        let i = idx.checked_sub(1)?;
        if &self.0[i].1 > value { Some(i) } else { None }
    }

    pub fn contains_range(&self, range: &std::ops::Range<T>) -> bool {
        if range.start >= range.end {
            return true;
        }
        match self.covering_index(&range.start) {
            Some(i) => self.0[i].1 >= range.end,
            None => false,
        }
    }

    pub fn overlaps(&self, range: &std::ops::Range<T>) -> bool {
        if range.start >= range.end {
            return false;
        }
        let idx = self.0.partition_point(|(_, end)| end <= &range.start);
        match self.0.get(idx) {
            Some((start, _)) => *start < range.end,
            None => false,
        }
    }
}

impl<T> Default for ImmutableRanges<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> From<Ranges<T>> for ImmutableRanges<T> {
    fn from(ranges: Ranges<T>) -> Self {
        ImmutableRanges(ranges.0.into_iter().collect())
    }
}

impl<T: Ord> From<ImmutableRanges<T>> for Ranges<T> {
    fn from(ranges: ImmutableRanges<T>) -> Self {
        Ranges(ranges.0.into_iter().collect())
    }
}

impl<T: Ord> FromIterator<std::ops::Range<T>> for ImmutableRanges<T> {
    fn from_iter<I: IntoIterator<Item = std::ops::Range<T>>>(iter: I) -> Self {
        let mut items: Vec<(T, T)> =
            iter.into_iter().filter(|r| r.start < r.end).map(|r| (r.start, r.end)).collect();
        items.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        items.dedup_by(|next, prev| {
            if prev.1 >= next.0 {
                if next.1 > prev.1 {
                    std::mem::swap(&mut prev.1, &mut next.1);
                }
                true
            } else {
                false
            }
        });

        Self(items)
    }
}

impl<T> IntoIterator for ImmutableRanges<T> {
    type Item = (T, T);
    type IntoIter = std::vec::IntoIter<(T, T)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for ImmutableRanges<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ImmutableRanges ")?;
        f.debug_set().entries(self.0.iter().map(|(s, e)| DebugRange(s, e))).finish()
    }
}

struct DebugRange<'a, T>(&'a T, &'a T);

impl<T: std::fmt::Debug> std::fmt::Debug for DebugRange<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}..{:?}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vec_of(ranges: &Ranges<i32>) -> Vec<(i32, i32)> {
        ranges.iter().map(|(s, e)| (*s, *e)).collect()
    }

    #[test]
    fn merge_on_insert() {
        let mut r = Ranges::new();
        r.insert(5..10);
        r.insert(1..3);
        assert_eq!(vec_of(&r), vec![(1, 3), (5, 10)]);

        r.insert(3..5); // 両隣に接するので 1 本になる
        assert_eq!(vec_of(&r), vec![(1, 10)]);

        r.insert(20..30);
        r.insert(0..25); // 既存を跨いで飲み込む
        assert_eq!(vec_of(&r), vec![(0, 30)]);

        r.insert(7..8); // 完全に内包される
        assert_eq!(vec_of(&r), vec![(0, 30)]);

        r.insert(40..40); // 空区間は無視
        assert_eq!(vec_of(&r), vec![(0, 30)]);
    }

    #[test]
    fn no_merge_for_disjoint_integers() {
        let r: Ranges<i32> = [1..2, 3..4].into_iter().collect();
        assert_eq!(vec_of(&r), vec![(1, 2), (3, 4)]);
    }

    #[test]
    // 1 本だけの Range 配列は「範囲そのものの Vec の書き間違い」と見なされるが、
    // ここは意図して 1 区間だけを流し込んでいる。
    #[allow(clippy::single_range_in_vec_init)]
    fn remove_splits_and_truncates() {
        let mut r: Ranges<i32> = [1..10].into_iter().collect();
        r.remove(3..5);
        assert_eq!(vec_of(&r), vec![(1, 3), (5, 10)]);

        r.remove(2..7); // 左を切り詰め、右の頭を削る
        assert_eq!(vec_of(&r), vec![(1, 2), (7, 10)]);

        r.remove(0..100);
        assert!(r.is_empty());
    }

    #[test]
    fn remove_across_multiple() {
        let mut r: Ranges<i32> = [0..5, 10..15, 20..25].into_iter().collect();
        r.remove(3..22);
        assert_eq!(vec_of(&r), vec![(0, 3), (22, 25)]);
    }

    #[test]
    fn queries() {
        let r: Ranges<i32> = [1..5, 10..20].into_iter().collect();
        assert!(r.contains(&1));
        assert!(r.contains(&4));
        assert!(!r.contains(&5)); // 半開区間なので end は含まない
        assert!(!r.contains(&9));
        assert_eq!(r.covering(&12), Some((&10, &20)));

        assert!(r.contains_range(&(10..20)));
        assert!(!r.contains_range(&(4..11)));
        assert!(r.overlaps(&(4..11)));
        assert!(!r.overlaps(&(5..10)));
    }

    #[test]
    fn immutable_view() {
        let r: Ranges<i32> = [10..20, 1..5, 3..7].into_iter().collect();
        let frozen: ImmutableRanges<_> = r.into();
        assert_eq!(frozen.as_slice(), &[(1, 7), (10, 20)]);

        assert!(!frozen.contains(&0));
        assert!(frozen.contains(&1));
        assert!(frozen.contains(&6));
        assert!(!frozen.contains(&7));
        assert!(frozen.contains(&19));
        assert!(!frozen.contains(&20));
        assert_eq!(frozen.covering(&15), Some(&(10, 20)));
        assert_eq!(frozen.covering_index(&3), Some(0));

        assert!(frozen.overlaps(&(6..11)));
        assert!(!frozen.overlaps(&(7..10)));
        assert!(frozen.contains_range(&(2..7)));
        assert!(!frozen.contains_range(&(2..8)));
    }

    #[test]
    fn immutable_from_iter() {
        // 未ソート/重なり/接触/内包/空区間/同一 start が混ざった入力
        let input = [10..20, 3..7, 1..5, 5..5, 7..10, 3..4, 30..31];
        let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
        assert_eq!(frozen.as_slice(), &[(1, 20), (30, 31)]);

        // Ranges 経由と同じ結果になること
        assert_eq!(frozen, Ranges::from_iter(input).into());

        let (lo, hi) = (3, 1); // 逆転区間。リテラルだと clippy に怒られるので変数経由
        let empty: ImmutableRanges<i32> = [5..5, lo..hi].into_iter().collect();
        assert!(empty.is_empty());
    }

    #[test]
    fn immutable_from_iter_touching_boundary() {
        let touching = [1..3, 3..5];
        let frozen: ImmutableRanges<i32> = touching.clone().into_iter().collect();
        assert_eq!(frozen.as_slice(), &[(1, 5)]);
        assert_eq!(frozen, Ranges::from_iter(touching).into());

        let disjoint = [1..2, 3..4];
        let frozen: ImmutableRanges<i32> = disjoint.clone().into_iter().collect();
        assert_eq!(frozen.as_slice(), &[(1, 2), (3, 4)]);
        assert_eq!(frozen, Ranges::from_iter(disjoint).into());
    }

    #[test]
    fn immutable_from_iter_swallows_chain() {
        // 大区間が後続を次々飲み込む。畳み込みが「残った区間」と比較していないと分裂する
        let input = [0..100, 10..20, 30..40, 50..60];
        let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
        assert_eq!(frozen.as_slice(), &[(0, 100)]);
        assert_eq!(frozen, Ranges::from_iter(input).into());

        // 途中で end が伸びていく鎖
        let input = [0..10, 5..20, 15..30, 100..101];
        let frozen: ImmutableRanges<i32> = input.clone().into_iter().collect();
        assert_eq!(frozen.as_slice(), &[(0, 30), (100, 101)]);
        assert_eq!(frozen, Ranges::from_iter(input).into());
    }

    #[test]
    fn immutable_from_iter_without_clone() {
        // Clone を要求しないことの確認 (Ranges 経由では通らない)
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        struct NoClone(i32);

        let r: ImmutableRanges<NoClone> =
            [NoClone(1)..NoClone(3), NoClone(3)..NoClone(5)].into_iter().collect();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn works_with_non_integer_keys() {
        let mut r = Ranges::new();
        r.insert("a".to_string().."m".to_string());
        r.insert("m".to_string().."z".to_string());
        assert_eq!(r.len(), 1);
        assert!(r.contains(&"q".to_string()));
    }
}
