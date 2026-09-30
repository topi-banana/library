// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/649

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use avl_tree::{Indexed, Map, SimpleElement};

fn main() {
    input! {
        q: usize,
        k: usize,
    }

    // 重複を許す集合なので、キーは (値, 通し番号) の組にして一意にする。
    let mut set: Map<Indexed<SimpleElement<(i64, usize), i64>>> = Map::new();
    let mut next_id = 0usize;
    let mut out = String::new();

    for _ in 0..q {
        input! { t: u8 }
        if t == 1 {
            input! { v: i64 }
            set.put((v, next_id), v);
            next_id += 1;
        } else if set.len() < k {
            writeln!(out, "-1").unwrap();
        } else {
            // K 番目に小さい要素は in-order で index K-1。
            let slot = set.slot_by_index(k - 1).unwrap();
            let value = *unsafe { set.slot_ref(slot) }.inner().value();
            unsafe { set.slot_remove(slot) };
            writeln!(out, "{value}").unwrap();
        }
    }

    let stdout = std::io::stdout();
    stdout.lock().write_all(out.as_bytes()).unwrap();
}
