// competitive-verifier: PROBLEM https://yukicoder.me/problems/no/1469

use std::io::{self, Read, Write};

use rle::Rle;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let s = input.trim_end();

    // 「隣り合う 2 文字が同じならその 1 つを消す」を可能な限り繰り返した結果は、
    // 連長圧縮した各区間の文字を 1 つずつ並べたものに等しい。
    // |S| <= 5 * 10^6 なので、chars ではなく bytes で走査する (S は英小文字のみ)。
    let mut res: Vec<u8> = s.bytes().rle().map(|(b, _)| b).collect();
    res.push(b'\n');

    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(&res)
}
