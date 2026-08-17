pub fn next_permutation<T: Ord>(a: &mut [T]) -> bool {
    let Some(i) = a.windows(2).rposition(|w| unsafe { w.get_unchecked(0) < w.get_unchecked(1) })
    else {
        return false;
    };
    let j = a.iter().rposition(|x| x > unsafe { a.get_unchecked(i) }).unwrap();
    a.swap(i, j);
    unsafe { a.get_unchecked_mut(i + 1..) }.reverse();
    true
}

pub fn prev_permutation<T: Ord>(a: &mut [T]) -> bool {
    let Some(i) = a.windows(2).rposition(|w| unsafe { w.get_unchecked(0) > w.get_unchecked(1) })
    else {
        return false;
    };
    let j = a.iter().rposition(|x| x < unsafe { a.get_unchecked(i) }).unwrap();
    a.swap(i, j);
    unsafe { a.get_unchecked_mut(i + 1..) }.reverse();
    true
}

pub fn permutation<T: Ord + Clone>(counts: &mut [(T, usize)], len: usize) -> Vec<Vec<T>> {
    fn dfs<T: Ord + Clone>(
        counts: &mut [(T, usize)],
        indices: &mut Vec<usize>,
        result: &mut Vec<Vec<T>>,
        len: usize,
    ) {
        if indices.len() == len {
            let mut c = Vec::with_capacity(len);
            for &i in &*indices {
                c.push(unsafe { counts.get_unchecked(i) }.0.clone());
            }
            result.push(c);
            return;
        }

        for i in 0..counts.len() {
            if unsafe { counts.get_unchecked(i) }.1 > 0 {
                indices.push(i);
                unsafe { counts.get_unchecked_mut(i) }.1 -= 1;
                dfs(counts, indices, result, len);
                unsafe { counts.get_unchecked_mut(i) }.1 += 1;
                indices.pop();
            }
        }
    }

    let mut result = vec![];
    dfs(counts, &mut Vec::with_capacity(len), &mut result, len);
    result
}

#[cfg(test)]
mod tests;
