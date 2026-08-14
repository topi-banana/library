// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/674

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use ranges::Ranges;

fn main() {
    input! {
        _d: u64,
        q: usize,
        jobs: [(u64, u64); q],
    }

    // 出勤する日の集合を、閉区間 [a, b] を半開区間 a..b+1 に直して持つ。
    // 「b 日目の翌日も出勤なら連勤が続く」ことは、半開区間が端点で接することに対応する。
    // insert は接する区間も 1 本に統合するので、区間の長さがそのまま連勤の日数になる。
    let mut days = Ranges::new();
    let mut ans = 0;
    let mut out = String::with_capacity(q * 20);
    for (a, b) in jobs {
        days.insert(a..b + 1);

        // 予定は増える一方なので、伸びうるのは今入れた区間だけ。
        // a はその区間に必ず含まれるので covering は Some になる。
        let (start, end) = days.covering(&a).unwrap();
        ans = ans.max(end - start);
        writeln!(out, "{ans}").unwrap();
    }

    let stdout = std::io::stdout();
    let mut o = stdout.lock();
    o.write_all(out.as_bytes()).unwrap();
}
