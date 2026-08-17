pub fn rle<T: Iterator<Item = I>, I: PartialEq>(iter: &mut T) -> Vec<(I, usize)> {
    let mut res = Vec::new();
    let Some(mut pre) = iter.next() else {
        return res;
    };
    let mut cnt = 1usize;
    for now in iter {
        if now != pre {
            res.push((pre, std::mem::take(&mut cnt)));
        }
        pre = now;
        cnt += 1;
    }
    res.push((pre, cnt));
    res
}

#[derive(Debug, Clone)]
pub struct RleIter<T, I> {
    iter: T,
    pre: Option<I>,
    cnt: usize,
}

impl<T: Iterator<Item = I>, I: PartialEq> Iterator for RleIter<T, I> {
    type Item = (I, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let pre = self.pre.take()?;
        for now in self.iter.by_ref() {
            if now != pre {
                let res = (pre, self.cnt);
                self.pre = Some(now);
                self.cnt = 1;
                return Some(res);
            }
            self.cnt += 1;
        }
        Some((pre, std::mem::take(&mut self.cnt)))
    }
}

pub trait Rle: Iterator + Sized {
    fn rle(mut self) -> RleIter<Self, Self::Item>
    where
        Self::Item: PartialEq,
    {
        let pre = self.next();
        let cnt = usize::from(pre.is_some());
        RleIter { iter: self, pre, cnt }
    }
}

impl<T: Iterator> Rle for T {}

#[cfg(test)]
mod tests;
