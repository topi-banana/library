use std::panic;

pub struct DivConquer<const N: usize, Element, Cache, Result, Arg> {
    slice: Vec<Element>,
    dirty: Vec<bool>,
    cache: Vec<std::mem::MaybeUninit<Cache>>,
    cacher: fn(&[Element]) -> Cache,
    resolver: fn(&[Element], &Arg) -> Result,
    cache_resolver: fn(&Cache, &Arg) -> Result,
    merger: fn(Result, Result) -> Result,
}

impl<const N: usize, E, C, R, A> DivConquer<N, E, C, R, A> {
    pub fn new(
        slice: Vec<E>,
        cacher: fn(&[E]) -> C,
        resolver: fn(&[E], &A) -> R,
        cache_resolver: fn(&C, &A) -> R,
        merger: fn(R, R) -> R,
    ) -> Self {
        let cache =
            slice.chunks(N).map(&cacher).map(std::mem::MaybeUninit::new).collect::<Vec<_>>();
        let dirty = vec![false; cache.len()];
        Self { slice, dirty, cache, cacher, resolver, cache_resolver, merger }
    }
    pub fn push(&mut self, element: E) {
        self.slice.push(element);
        let block = (self.slice.len() - 1) / N;
        if block == self.cache.len() {
            self.cache.push(std::mem::MaybeUninit::uninit());
            self.dirty.push(true);
        } else {
            self.dirty[block] = true;
        }
    }
    pub fn pop(&mut self) -> Option<E> {
        let res = self.slice.pop();
        if res.is_some() {
            if self.cache.len() > self.slice.len().div_ceil(N) {
                self.dirty.pop();
                self.cache.pop();
            } else {
                self.dirty[self.slice.len() / N] = true;
            }
        }
        res
    }
    pub fn set(&mut self, index: usize, element: E) {
        self.slice[index] = element;
        self.dirty[index / N] = true;
    }
    pub fn into_immut<'a>(&'a mut self) -> ImmutableDivConquer<'a, N, E, C, R, A> {
        for i in 0..self.dirty.len() {
            if self.dirty[i] {
                self.dirty[i] = false;
                let end = ((i + 1) * N).min(self.slice.len());
                self.cache[i].write((self.cacher)(&self.slice[i * N..end]));
            }
        }
        ImmutableDivConquer {
            slice: &self.slice,
            cache: unsafe { self.cache.assume_init_ref() },
            resolver: self.resolver,
            cache_resolver: self.cache_resolver,
            merger: self.merger,
        }
    }
}

pub struct ImmutableDivConquer<'a, const N: usize, Element, Cache, Result, Arg> {
    slice: &'a [Element],
    cache: &'a [Cache],
    resolver: fn(&[Element], &Arg) -> Result,
    cache_resolver: fn(&Cache, &Arg) -> Result,
    merger: fn(Result, Result) -> Result,
}

impl<'a, const N: usize, E, C, R, A> ImmutableDivConquer<'a, N, E, C, R, A> {
    pub fn new(
        slice: &'a [E],
        cache: &'a mut Vec<C>,
        cacher: fn(&[E]) -> C,
        resolver: fn(&[E], &A) -> R,
        cache_resolver: fn(&C, &A) -> R,
        merger: fn(R, R) -> R,
    ) -> Self {
        if !cache.is_empty() {
            panic!()
        }
        cache.extend(slice.chunks(N).map(&cacher));
        Self { slice, cache, resolver, cache_resolver, merger }
    }
    pub fn resolve(&self, range: impl std::ops::RangeBounds<usize>, arg: &A) -> R
    where
        R: Default + Clone,
    {
        use std::ops::Bound;

        let l = match range.start_bound() {
            Bound::Included(&s) => s,
            Bound::Excluded(&s) => s + 1,
            Bound::Unbounded => 0,
        };
        let r = match range.end_bound() {
            Bound::Included(&e) => e + 1,
            Bound::Excluded(&e) => e,
            Bound::Unbounded => self.slice.len(),
        };
        assert!(l <= r && r <= self.slice.len(), "range out of bounds");

        let (lb, rb) = (l / N, r / N);

        if lb == rb {
            return (self.resolver)(&self.slice[l..r], arg);
        }

        let mut acc = R::default();

        let head = if l % N == 0 {
            lb
        } else {
            acc = (self.merger)(acc, (self.resolver)(&self.slice[l..(lb + 1) * N], arg));
            lb + 1
        };

        for b in head..rb {
            acc = (self.merger)(acc, (self.cache_resolver)(&self.cache[b], arg));
        }

        if r % N != 0 {
            acc = (self.merger)(acc, (self.resolver)(&self.slice[rb * N..r], arg));
        }

        acc
    }
}

#[cfg(test)]
mod tests;
