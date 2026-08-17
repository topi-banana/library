pub struct Array2d<T> {
    w: usize,
    body: Vec<T>,
}

impl<T> Array2d<T> {
    pub fn with_capacity(h: usize, w: usize) -> Self {
        Self { w, body: Vec::with_capacity(h * w) }
    }

    pub fn width(&self) -> usize {
        self.w
    }

    pub fn height(&self) -> usize {
        self.body.len().checked_div(self.w).unwrap_or(0)
    }

    fn row_start(&self, row: usize) -> usize {
        let h = self.height();
        assert!(row < h, "index out of bounds: the height is {h} but the row index is {row}");
        row * self.w
    }

    fn row_range(&self, start: usize, end: usize) -> std::ops::Range<usize> {
        let h = self.height();
        assert!(start <= end, "slice index starts at row {start} but ends at row {end}");
        assert!(end <= h, "range end index {end} out of range for Array2d of height {h}");
        start * self.w..end * self.w
    }

    fn row_slice(&self, start: usize, end: usize) -> &[T] {
        &self.body[self.row_range(start, end)]
    }

    fn row_slice_mut(&mut self, start: usize, end: usize) -> &mut [T] {
        let range = self.row_range(start, end);
        &mut self.body[range]
    }
}

fn inclusive_end(end: usize) -> usize {
    end.checked_add(1).expect("range end index is out of range for Array2d")
}

impl<T> std::ops::Index<usize> for Array2d<T> {
    type Output = [T];

    fn index(&self, row: usize) -> &Self::Output {
        let start = self.row_start(row);
        &self.body[start..][..self.w]
    }
}

impl<T> std::ops::IndexMut<usize> for Array2d<T> {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        let (start, w) = (self.row_start(row), self.w);
        &mut self.body[start..][..w]
    }
}

macro_rules! impl_row_range_index {
    ($($ty:ty => |$r:ident, $h:ident| $bounds:expr),* $(,)?) => {$(
        impl<T> std::ops::Index<$ty> for Array2d<T> {
            type Output = [T];

            fn index(&self, $r: $ty) -> &Self::Output {
                let $h = self.height();
                let (start, end) = $bounds;
                self.row_slice(start, end)
            }
        }

        impl<T> std::ops::IndexMut<$ty> for Array2d<T> {
            fn index_mut(&mut self, $r: $ty) -> &mut Self::Output {
                let $h = self.height();
                let (start, end) = $bounds;
                self.row_slice_mut(start, end)
            }
        }
    )*};
}

impl_row_range_index! {
    std::ops::Range<usize> => |r, _h| (r.start, r.end),
    std::ops::RangeTo<usize> => |r, _h| (0, r.end),
    std::ops::RangeFrom<usize> => |r, h| (r.start, h),
    std::ops::RangeFull => |_r, h| (0, h),
    std::ops::RangeInclusive<usize> => |r, _h| (*r.start(), inclusive_end(*r.end())),
    std::ops::RangeToInclusive<usize> => |r, _h| (0, inclusive_end(r.end)),
}

#[cfg(test)]
mod tests;
