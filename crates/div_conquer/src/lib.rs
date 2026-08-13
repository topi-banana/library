struct DivConquer<const N: usize, Element, Cache, Result, Arg> {
    slice: Vec<Element>,
    cache: Vec<Cache>,
    cacher: fn(&[Element]) -> Cache,
    resolver: fn(&[Element], &Arg) -> Result,
    cache_resolver: fn(&Cache, &Arg) -> Result,
    merger: fn(Result, Result) -> Result,
}

impl<const N: usize, E, C, R, A> DivConquer<N, E, C, R, A> {
    fn new(
        slice: Vec<E>,
        cacher: fn(&[E]) -> C,
        resolver: fn(&[E], &A) -> R,
        cache_resolver: fn(&C, &A) -> R,
        merger: fn(R, R) -> R,
    ) -> Self {
        let cache = slice.chunks(N).map(&cacher).collect::<Vec<_>>();
        Self { slice, cache, cacher, resolver, cache_resolver, merger }
    }
    fn resolve(&self, range: impl std::ops::RangeBounds<usize>, arg: &A) -> R
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
