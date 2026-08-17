use super::*;

/// 区間和を求める `DivConquer` を作る。
fn sum<const N: usize>(v: Vec<i64>) -> DivConquer<N, i64, i64, i64, ()> {
    DivConquer::new(v, |s| s.iter().sum(), |s, _| s.iter().sum(), |c, _| *c, |a, b| a + b)
}

/// 全区間について愚直な区間和と一致することを確かめる。
fn assert_all_ranges<const N: usize>(dc: &mut DivConquer<N, i64, i64, i64, ()>, naive: &[i64]) {
    let im = dc.into_immut();
    for l in 0..=naive.len() {
        for r in l..=naive.len() {
            assert_eq!(im.resolve(l..r, &()), naive[l..r].iter().sum::<i64>(), "range {l}..{r}");
        }
    }
}

#[test]
fn push_crosses_block_boundary() {
    let mut dc = sum::<4>(vec![]);
    let mut naive = vec![];
    for x in 1..=12 {
        dc.push(x);
        naive.push(x);
        assert_all_ranges(&mut dc, &naive);
    }
}

#[test]
fn pop_shrinks_blocks() {
    let mut naive = (1..=9).collect::<Vec<i64>>();
    let mut dc = sum::<4>(naive.clone());
    while let Some(x) = dc.pop() {
        assert_eq!(Some(x), naive.pop());
        assert_all_ranges(&mut dc, &naive);
    }
    assert!(naive.is_empty());
}

#[test]
fn set_rebuilds_partial_last_block() {
    let mut naive = vec![1; 6];
    let mut dc = sum::<4>(naive.clone());
    dc.set(5, 100);
    naive[5] = 100;
    assert_all_ranges(&mut dc, &naive);
}

#[test]
fn block_size_one() {
    let mut dc = sum::<1>(vec![]);
    let mut naive = vec![];
    for x in 1..=4 {
        dc.push(x);
        naive.push(x);
        assert_all_ranges(&mut dc, &naive);
    }
    while dc.pop().is_some() {
        naive.pop();
        assert_all_ranges(&mut dc, &naive);
    }
}
