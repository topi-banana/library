pub trait MoSol {
    type Ans;
    const MAX_INDEX_POW2: usize;

    fn add_l(&mut self, l_idx: usize);
    fn add_r(&mut self, r_idx: usize);
    fn del_l(&mut self, l_idx: usize);
    fn del_r(&mut self, r_idx: usize);
    fn solve(&mut self) -> Self::Ans;
}

#[derive(Debug, Clone, Default)]
pub struct Mo {
    queries: Vec<(usize, usize, usize)>,
}
impl Mo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, l: usize, r: usize) {
        self.queries.push((l, r, self.queries.len()));
    }

    pub fn execute<S: MoSol>(mut self, state: &mut S) -> Box<[S::Ans]>
    where
        S::Ans: Default + Clone,
    {
        fn hilbert_order(x: usize, y: usize, pow: usize, rotate: usize) -> usize {
            if pow == 0 {
                return 0;
            }
            let hpow = 1 << (pow - 1);
            let seg = match (x < hpow, y < hpow) {
                (true, true) => 0,
                (true, false) => 3,
                (false, true) => 1,
                (false, false) => 2,
            };
            let seg = (seg + rotate) & 3;
            let (nx, ny) = (x & (x ^ hpow), y & (y ^ hpow));
            let nrot = (rotate + [3, 0, 0, 1][seg]) & 3;
            let sub_square_size = 1usize << ((pow << 1) - 2);
            let ans = seg * sub_square_size;
            let add = hilbert_order(nx, ny, pow - 1, nrot);
            if seg == 1 || seg == 2 { ans + add } else { ans + sub_square_size - add - 1 }
        }
        self.queries.sort_by_cached_key(|&(l, r, _)| hilbert_order(l, r, S::MAX_INDEX_POW2, 0));

        let mut ans = vec![S::Ans::default(); self.queries.len()].into_boxed_slice();
        let (mut nl, mut nr) = (0, 0);
        for (l, r, i) in self.queries {
            while nl > l {
                nl -= 1;
                state.add_l(nl);
            }
            while nr < r {
                state.add_r(nr);
                nr += 1
            }
            while nl < l {
                state.del_l(nl);
                nl += 1
            }
            while nr > r {
                nr -= 1;
                state.del_r(nr);
            }
            ans[i] = state.solve();
        }
        ans
    }
}
