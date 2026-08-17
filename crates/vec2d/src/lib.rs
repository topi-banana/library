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
mod tests {
    use super::Array2d;

    // 外から要素を詰める手段がまだないので、テストでは直接組み立てる。
    fn seq(h: usize, w: usize) -> Array2d<usize> {
        Array2d { w, body: (0..h * w).collect() }
    }

    #[test]
    fn shape() {
        let a = seq(3, 4);
        assert_eq!(a.height(), 3);
        assert_eq!(a.width(), 4);
        // 容量を確保しただけでは行はまだ存在しない。
        assert_eq!(Array2d::<usize>::with_capacity(3, 4).height(), 0);
    }

    #[test]
    fn row_index_returns_one_row() {
        let a = seq(3, 4);
        assert_eq!(a[0], [0, 1, 2, 3]);
        assert_eq!(a[1], [4, 5, 6, 7]);
        assert_eq!(a[2], [8, 9, 10, 11]);
    }

    #[test]
    fn element_index_like_nested_array() {
        let a = seq(3, 4);
        assert_eq!(a[0][0], 0);
        assert_eq!(a[1][2], 6);
        assert_eq!(a[2][3], 11);
    }

    #[test]
    fn row_ranges() {
        let a = seq(3, 4);
        let all: Vec<usize> = (0..12).collect();
        assert_eq!(a[1..2], [4, 5, 6, 7]);
        assert_eq!(a[..2], [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(a[1..], [4, 5, 6, 7, 8, 9, 10, 11]);
        assert_eq!(a[..], all[..]);
        assert_eq!(a[0..=1], [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(a[..=0], [0, 1, 2, 3]);
    }

    #[test]
    fn range_slices_split_back_into_rows() {
        let a = seq(3, 4);
        let rows: Vec<&[usize]> = a[1..].chunks(a.width()).collect();
        assert_eq!(rows, vec![&a[1], &a[2]]);
    }

    #[test]
    fn empty_ranges() {
        let a = seq(3, 4);
        assert!(a[2..2].is_empty());
        assert!(a[..0].is_empty());
        assert!(a[3..].is_empty());
        // 幅が 0 なら全体を取っても空。
        assert!(Array2d::<usize> { w: 0, body: Vec::new() }[..].is_empty());
    }

    #[test]
    fn index_mut_writes_through() {
        let mut a = seq(2, 3);
        a[0][1] = 100;
        a[1..][2] = 200;
        assert_eq!(a[0], [0, 100, 2]);
        assert_eq!(a[1], [3, 4, 200]);

        a[..].fill(0);
        assert_eq!(a[..], [0; 6]);
    }

    #[test]
    #[should_panic(expected = "the height is 3 but the row index is 3")]
    fn row_index_out_of_bounds() {
        let a = seq(3, 4);
        let _ = &a[3];
    }

    #[test]
    #[should_panic(expected = "range end index 4 out of range for Array2d of height 3")]
    fn range_end_out_of_bounds() {
        let a = seq(3, 4);
        let _ = &a[..4];
    }

    #[test]
    // 開始行が行数を超える場合。end は行数まで詰められるので、逆転として弾かれる。
    #[should_panic(expected = "slice index starts at row 4 but ends at row 3")]
    fn range_start_out_of_bounds() {
        let a = seq(3, 4);
        let _ = &a[4..];
    }

    #[test]
    #[should_panic(expected = "slice index starts at row 2 but ends at row 1")]
    fn reversed_range() {
        let a = seq(3, 4);
        let (lo, hi) = (2, 1); // リテラルで書くと clippy に空範囲だと怒られる
        let _ = &a[lo..hi];
    }

    #[test]
    #[should_panic(expected = "range end index is out of range for Array2d")]
    fn inclusive_range_at_max() {
        let a = seq(3, 4);
        let _ = &a[..=usize::MAX];
    }

    #[test]
    #[should_panic(expected = "the height is 0 but the row index is 0")]
    fn zero_width_has_no_row() {
        let a = Array2d::<usize> { w: 0, body: Vec::new() };
        let _ = &a[0];
    }
}
