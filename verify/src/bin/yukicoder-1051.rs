// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1051

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use permutation::next_permutation;

/// `[t, s から t を除いた残りを降順]` を作る。
///
/// これに `next_permutation` を掛けると
/// `[s の中で t より大きい最小の値, 残りを昇順]` になる。
/// 先頭を除いた部分が降順なので pivot は必ず先頭で、
/// 交換相手は「t より大きい最後の要素」= t より大きい最小の値だからである。
fn descending_from(s: &[usize], t: usize) -> Vec<usize> {
    let mut rest: Vec<usize> = s.iter().copied().filter(|&x| x != t).collect();
    rest.sort_unstable();
    rest.reverse();

    let mut v = Vec::with_capacity(s.len());
    v.push(t);
    v.append(&mut rest);
    v
}

fn main() {
    input! {
        n: usize,
        p: usize,
        q: usize,
        a: [usize; n],
    }

    let pos_p = a.iter().position(|&x| x == p).unwrap();
    let pos_q = a.iter().position(|&x| x == q).unwrap();

    // suf_max[i]       = max(a[i..])
    // suf_max_not_q[i] = max{ a[j] | j >= i, a[j] != q }
    // どちらも要素が無ければ 0。値は 1 以上なので 0 は「無し」を表せる。
    let mut suf_max = vec![0usize; n + 1];
    let mut suf_max_not_q = vec![0usize; n + 1];
    for (i, &x) in a.iter().enumerate().rev() {
        suf_max[i] = suf_max[i + 1].max(x);
        suf_max_not_q[i] = if x == q { suf_max_not_q[i + 1] } else { suf_max_not_q[i + 1].max(x) };
    }

    // b は a と先頭 i 要素を共有し b[i] > a[i] となる形しかない。
    // i が大きいほど b は小さくなるので、条件を満たす最大の i を探す。
    // i を決めたときの p, q の位置関係は次の 4 通り。
    //
    //   i > max(pos_p, pos_q)     p, q とも接頭辞にあり順序は a のまま  -> pos_p < pos_q なら可
    //   pos_p < i <= pos_q        p だけ接頭辞にある                    -> 可
    //   pos_q < i <= pos_p        q だけ接頭辞にある (p は必ず後ろ)     -> 不可
    //   i <= min(pos_p, pos_q)    どちらも接尾辞にある                  -> b[i] != q なら可
    //
    // 最後の場合に b[i] = q を許さないのは、p が i より後ろに来てしまうからである。
    let mut found = None;
    if pos_p < pos_q {
        // i > pos_p では q の制約が付かないので、a[i] より大きい値が後ろにある最大の i でよい。
        found = (pos_p + 1..n).rev().find(|&i| a[i] < suf_max[i + 1]).map(|i| (i, false));
    }
    if found.is_none() {
        // 残るのは i <= min(pos_p, pos_q)。b[i] に q は置けないので、
        // a[i] より大きい q 以外の値が後ろにあることを条件にする。
        found =
            (0..=pos_p.min(pos_q)).rev().find(|&i| a[i] < suf_max_not_q[i + 1]).map(|i| (i, true));
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let Some((i, forbid_q)) = found else {
        out.write_all(b"-1\n").unwrap();
        return;
    };

    // tail = [a[i] より大きい最小の値, 残りを昇順]。
    let mut tail = descending_from(&a[i..], a[i]);
    assert!(next_permutation(&mut tail));
    if forbid_q && tail[0] == q {
        // 先頭に q は置けない。閾値を q へ上げれば、a[i] より大きい q 以外の
        // 最小の値 (= q より大きい最小の値) が先頭に来る。
        tail = descending_from(&a[i..], q);
        assert!(next_permutation(&mut tail));
    }

    // p, q が両方とも tail[1..] (昇順部分) にあり p > q なら、このままでは q が先に来る。
    // 先頭から順に「置ける中で最小の値」を選ぶと、q は p を置いた直後に初めて置けるので、
    // q を p の直後まで後ろへずらしたものが条件を満たす最小の並びになる。
    if forbid_q && tail[0] != p && p > q {
        let asc = &mut tail[1..];
        let ip = asc.binary_search(&p).unwrap();
        let iq = asc.binary_search(&q).unwrap();
        asc[iq..=ip].rotate_left(1);
    }

    let mut s = String::with_capacity(n * 7);
    for (idx, x) in a[..i].iter().chain(&tail).enumerate() {
        if idx > 0 {
            s.push(' ');
        }
        write!(s, "{x}").unwrap();
    }
    s.push('\n');
    out.write_all(s.as_bytes()).unwrap();
}
