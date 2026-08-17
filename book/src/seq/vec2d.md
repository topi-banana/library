# Array2d — 2 次元配列

`h × w` の 2 次元配列を、行優先で平坦にした `Vec<T>` 1 本で保持します。
確保が 1 回で済み、要素がメモリ上に連続して並ぶため、
`Vec<Vec<T>>` より走査が速く、行ごとの間接参照もありません。

- 実装: [`crates/vec2d/src/lib.rs`](https://github.com/topi-banana/library/blob/main/crates/vec2d/src/lib.rs)
- verify: なし

対応するジャッジ問題が無いため、ユニットテストだけで検証しています。

crate 名は `vec2d` ですが、型名は `Array2d` です。`use vec2d::Array2d;` で読み込みます。

## メモリ表現

持っているのは幅 `w` と平坦な `body` だけで、高さは `body.len() / w` から求めます。

```text
h = 3, w = 4

body: [  0  1  2  3 |  4  5  6  7 |  8  9 10 11 ]
         row 0         row 1         row 2

a[1]    ->            [  4  5  6  7 ]
a[1..3] ->            [  4  5  6  7    8  9 10 11 ]
```

行数を別に持たないので、`body` の長さが `w` の倍数であることが不変条件です。
`w == 0` のときは割り算ができないため、高さは常に `0` として扱います。

## API

要素の型に境界はありません。

| 項目 | 計算量 | 説明 |
| --- | --- | --- |
| `Array2d::with_capacity(h, w)` | `O(1)` (確保は `h * w` に比例) | 幅を `w` に固定し、`h * w` 要素分の容量だけ確保した空の配列を作る |
| `width()` | `O(1)` | 幅 `w` |
| `height()` | `O(1)` | 高さ `body.len() / w`。`w == 0` なら `0` |
| `a[row]` | `O(1)` | `row` 行目を `&[T]` で返す |
| `a[row][col]` | `O(1)` | 要素。`Vec<Vec<T>>` と同じ書き方で引ける |
| `a[range]` | `O(1)` | 行の範囲を**平坦な** `&[T]` で返す |

`IndexMut` も同じ 3 つの形で実装しているので、`a[0][1] = x` や `a[..].fill(0)` が書けます。

行の範囲に使える型は `Range`、`RangeTo`、`RangeFrom`、`RangeFull`、
`RangeInclusive`、`RangeToInclusive` の 6 種類です。
添字はどれも**行番号**であって、平坦にした後の要素番号ではありません。

### 行範囲は行の列ではなく平坦なスライス

`a[1..3]` が返すのは `&[[T]]` ではなく `&[T]` で、2 行分が繋がった 1 本のスライスです。
これは内部表現をそのまま切り出しているためで、`fill` や `sort` を
複数行にまたげてかけられる代わりに、行の境界は消えます。

行に戻したいときは `chunks(a.width())` で切り直します。

```rust,ignore
// a[1..] は 2 行分の平坦なスライス
let rows: Vec<&[usize]> = a[1..].chunks(a.width()).collect();
assert_eq!(rows, vec![&a[1], &a[2]]);
```

`w == 0` のときは `chunks` がパニックするので、幅が `0` になりうる場面では
`a[..]` が空であることを先に確認してください。

## 使用例

```rust
use vec2d::Array2d;

let a = Array2d::<u32>::with_capacity(3, 4);
assert_eq!(a.width(), 4);
// 容量を確保しただけでは行はまだ存在しない
assert_eq!(a.height(), 0);
```

**現時点では、外から要素を詰める手段がありません。**
公開されている構築 API は `with_capacity` だけで、これは空の配列を返します。
`Index` / `IndexMut` は既にある行にしか届かないため、
今のところ crate の外からは空の配列しか作れません。
値を入れて使うには、`Vec<T>` から作る、行を追加する、
関数から生成する、といった構築 API を追加する必要があります。

`h = 3`、`w = 4` に `0..12` が入っている配列を `a` として、添字の挙動は次の通りです。

```rust,ignore
assert_eq!(a[1], [4, 5, 6, 7]);
assert_eq!(a[1][2], 6);

assert_eq!(a[..2], [0, 1, 2, 3, 4, 5, 6, 7]);
assert_eq!(a[0..=1], [0, 1, 2, 3, 4, 5, 6, 7]);
assert_eq!(a[2..], [8, 9, 10, 11]);

// 書き換える側 (`a` が `mut` である場合)
a[0][1] = 100; // 1 要素だけ書き換える
a[1..].fill(0); // 行 1 以降をまとめて潰す
```

## パニック

範囲外の添字はすべてパニックします。高さ `3`、幅 `4` の配列でのメッセージは次の通りです。

| 式 | メッセージ |
| --- | --- |
| `a[3]` | `index out of bounds: the height is 3 but the row index is 3` |
| `a[..4]` | `range end index 4 out of range for Array2d of height 3` |
| `a[2..1]` | `slice index starts at row 2 but ends at row 1` |
| `a[4..]` | `slice index starts at row 4 but ends at row 3` |
| `a[..=usize::MAX]` | `range end index is out of range for Array2d` |

`a[4..]` が「範囲外」ではなく「逆転した範囲」として報告されるのは、
`RangeFrom` の終端が高さで埋められ、`4..3` になってから検査されるためです。
開始行が高さを超えている点は変わらないので、パニックすること自体は同じです。

`a[..=usize::MAX]` は閉区間を半開区間に直す `end + 1` が溢れるケースで、
`checked_add` で受けて明示的にパニックさせています。
リリースビルドで巻き戻って別の範囲になることはありません。

構築側の `with_capacity(h, w)` は `h * w` をそのまま `Vec::with_capacity` に渡すため、
検査は入っていません。掛け算が `usize` を溢れると、
デバッグビルドではオーバーフローでパニックし、リリースビルドでは巻き戻った容量が確保されます。
確保する容量が変わるだけで、以降の添字の検査には影響しません。

空の範囲は正当です。`a[2..2]`、`a[..0]`、`a[3..]` はいずれも空のスライスを返します。
`a[3..]` が通るのは、開始行が高さ**ちょうど**なら逆転にならないためで、
`a[4..]` との境目はここです。

幅が `0` の配列は高さも `0` になります。`a[..]` は空を返しますが、
`a[0]` は行が 1 つも無いのでパニックします。

## 実装メモ

`Index<usize>` は `row_start` で `row < height()` を確かめてから、
`&self.body[start..][..w]` と 2 段階に切ります。
後段の `[..w]` は不変条件から必ず成立するので、実質の検査は `row_start` の 1 回だけです。

行範囲の 6 種類は `impl_row_range_index!` マクロでまとめて実装しています。
マクロに渡すのは範囲の型と、そこから `(開始行, 終了行)` を取り出すクロージャ風の式だけで、
検査と `w` 倍は `row_range` に集約されています。

```rust,ignore
std::ops::RangeFrom<usize> => |r, h| (r.start, h),
```

`row_range` は `start <= end` を先に、`end <= height()` を後に確かめます。
この順序が `a[4..]` のメッセージを決めています。

`Index` と `IndexMut` で `row_slice` / `row_slice_mut` に分かれているのは、
`&mut self` を借りたまま `self.row_range(..)` を呼べないためです。
`row_slice_mut` では範囲を先に変数へ取り出してから借りています。

`height()` の `checked_div(self.w).unwrap_or(0)` は、幅 `0` での 0 除算を避けるためのものです。
このとき `body` は空でしかありえないので、高さ `0` は実態とも合っています。

現在 `Debug`、`Clone`、`PartialEq` は導出していません。
テストで中身を比較しているのは、`Index` が返すスライス同士の比較です。
