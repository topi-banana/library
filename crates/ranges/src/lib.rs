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
mod tests;
