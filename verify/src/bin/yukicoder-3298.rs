// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/3298

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use avl_tree::{Indexed, Map, SimpleElement};

fn main() {
    input! {
        n: usize,
        k: usize,
        q: usize,
        a: [i64; n],
    }

    // 大きさが同じスライムを複数持てるように、キーは (値, 通し番号) の組にする。
    let mut slimes: Map<Indexed<SimpleElement<(i64, usize), i64>>> = Map::new();
    let mut next_id = 0usize;
    for value in a {
        slimes.put((value, next_id), value);
        next_id += 1;
    }

    let mut out = String::new();
    for _ in 0..q {
        input! { t: u8 }
        match t {
            1 => {
                input! { x: i64 }
                slimes.put((x, next_id), x);
                next_id += 1;
            }
            2 => {
                input! { y: i64 }
                // K 番目を取り出し、y だけ大きくして戻す。要素数は変わらないので
                // K <= 要素数 は常に成り立つ。
                let slot = slimes.slot_by_index(k - 1).unwrap();
                let value = *unsafe { slimes.slot_ref(slot) }.inner().value();
                unsafe { slimes.slot_remove(slot) };
                slimes.put((value + y, next_id), value + y);
                next_id += 1;
            }
            _ => {
                let value = *slimes.get_by_index(k - 1).unwrap().value();
                writeln!(out, "{value}").unwrap();
            }
        }
    }

    let stdout = std::io::stdout();
    stdout.lock().write_all(out.as_bytes()).unwrap();
}
