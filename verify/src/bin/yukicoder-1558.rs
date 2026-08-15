// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1558

use proconio::input;
use std::fmt::Write as _;
use std::io::Write as _;

use div_conquer::DivConquer;

/// ブロック長。
///
/// 1 クエリの計算量は「完全なブロック `M / B` 個の合成」「両端の半端な部分
/// 高々 `2B` 要素の合成」「更新で汚れたブロック 1 個の作り直し `B` 要素の合成」の和になる。
/// どれも同じ置換の合成で定数倍に差がないので、`M / B + 3B` を最小にする
/// `√(M / 3) ≒ 58` に近い値を選ぶ。
const B: usize = 64;

/// 馬の頭数の上限。
const MAX_N: usize = 18;

/// 区間を走る間の順位の入れ替わりを表す置換。
///
/// `Perm(p)` は「その区間に `i` 番目に入った馬が `p[i]` 番目に出る」ことを表す
/// (どちらも 0-indexed)。頭数は入力ごとに変わるが、`DivConquer` に渡す関数は
/// `fn` ポインタなので `N` を捉えられない。そこで長さは常に `MAX_N` とし、
/// `N` 以上の添字は動かさない (恒等) ままにしておく。恒等な部分は合成しても
/// 恒等のままなので、実在する `N` 頭ぶんの結果は変わらない。
#[derive(Clone, Copy)]
struct Perm([u8; MAX_N]);

impl Default for Perm {
    /// 恒等置換。
    ///
    /// `DivConquer::resolve` は `Default` から畳み込みを始めるので、
    /// これが合成の単位元でなければならない。
    fn default() -> Self {
        Self(std::array::from_fn(|i| i as u8))
    }
}

/// `first` の区間を走ってから `second` の区間を走ったときの置換を返す。
///
/// `i` 番目に入った馬は `first` を `first[i]` 番目に出て、
/// その順位で `second` に入り `second[first[i]]` 番目に出る。
/// 合成は非可換だが、`resolve` は左 (前の区間) から順に畳み込むので
/// `merger(acc, next)` の `acc` が必ず前側になり、この向きで整合する。
fn merger(first: Perm, second: Perm) -> Perm {
    Perm(std::array::from_fn(|i| second.0[first.0[i] as usize]))
}

/// ブロックに含まれる区間を先頭から順に合成する。
fn cacher(block: &[Perm]) -> Perm {
    block.iter().fold(Perm::default(), |acc, &p| merger(acc, p))
}

fn resolver(block: &[Perm], _: &()) -> Perm {
    cacher(block)
}

fn cache_resolver(cache: &Perm, _: &()) -> Perm {
    *cache
}

fn main() {
    input! {
        n: usize,
        m: usize,
        q: usize,
    }

    // 情報がまだ届いていない区間は恒等にしておく。制約より、type 2 と type 3 が
    // 参照する区間には必ず先に type 1 が来るので、この初期値は答えに影響しない。
    let mut dc = DivConquer::<B, _, _, _, _>::new(
        vec![Perm::default(); m],
        cacher,
        resolver,
        cache_resolver,
        merger,
    );

    let mut out = String::new();
    for _ in 0..q {
        input! { kind: u8 }
        match kind {
            // 区間 D の置換を丸ごと置き換える。
            1 => {
                input! { d: usize, p: [u8; n] }
                let mut perm = Perm::default();
                for (dst, &src) in perm.0.iter_mut().zip(&p) {
                    *dst = src - 1;
                }
                dc.set(d - 1, perm);
            }
            // 区間 1 に入る順番は馬 1, 2, ..., N なので、区間 1..=S を合成した置換は
            // そのまま「馬 i の着順」を表す。出力は着順から馬を引くので逆置換を作る。
            2 => {
                input! { s: usize }
                let t = dc.into_immut().resolve(..s, &());
                let mut horse = [0; MAX_N];
                for (i, &rank) in t.0[..n].iter().enumerate() {
                    horse[usize::from(rank)] = i + 1;
                }
                for (i, h) in horse[..n].iter().enumerate() {
                    let sep = if i + 1 == n { '\n' } else { ' ' };
                    write!(out, "{h}{sep}").unwrap();
                }
            }
            // 区間 L に i 番目に入った馬が区間 R を出るときの順位が resolve の結果。
            _ => {
                input! { l: usize, r: usize }
                let t = dc.into_immut().resolve(l - 1..r, &());
                let e = t.0[..n]
                    .iter()
                    .enumerate()
                    .map(|(i, &rank)| i.abs_diff(usize::from(rank)))
                    .sum::<usize>();
                writeln!(out, "{e}").unwrap();
            }
        }
    }

    let stdout = std::io::stdout();
    let mut o = stdout.lock();
    o.write_all(out.as_bytes()).unwrap();
}
