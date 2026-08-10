//! 素集合データ構造 (Union-Find)。
//!
//! 経路圧縮と union by size により、各操作をならし `O(α(n))` で行う。
//!
//! ```
//! use dsu::Dsu;
//!
//! let mut dsu = Dsu::new(4);
//! dsu.merge(0, 1);
//! dsu.merge(1, 2);
//! assert!(dsu.same(0, 2));
//! assert!(!dsu.same(0, 3));
//! assert_eq!(dsu.size(0), 3);
//! ```

/// 素集合データ構造。
///
/// `parent_or_size[i]` は、`i` が代表元のとき `-(連結成分のサイズ)`、
/// そうでないとき親の頂点番号を保持する。
#[derive(Debug, Clone)]
pub struct Dsu {
    n: usize,
    parent_or_size: Vec<isize>,
}

impl Dsu {
    /// `n` 頂点、辺が 1 本もない状態で初期化する。
    pub fn new(n: usize) -> Self {
        Self {
            n,
            parent_or_size: vec![-1; n],
        }
    }

    /// 頂点数を返す。
    pub fn len(&self) -> usize {
        self.n
    }

    /// 頂点が 1 つも無いかどうかを返す。
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    /// `a` が属する連結成分の代表元を返す。
    ///
    /// # Panics
    ///
    /// `a >= self.len()` のときパニックする。
    pub fn leader(&mut self, a: usize) -> usize {
        assert!(a < self.n, "頂点番号が範囲外です: {a}");
        let mut root = a;
        while self.parent_or_size[root] >= 0 {
            root = self.parent_or_size[root] as usize;
        }
        // 経路圧縮。根までの頂点をすべて根に直結させる。
        let mut cur = a;
        while self.parent_or_size[cur] >= 0 {
            let next = self.parent_or_size[cur] as usize;
            self.parent_or_size[cur] = root as isize;
            cur = next;
        }
        root
    }

    /// `a` と `b` を連結し、併合後の代表元を返す。
    ///
    /// # Panics
    ///
    /// `a` または `b` が範囲外のときパニックする。
    pub fn merge(&mut self, a: usize, b: usize) -> usize {
        let (mut x, mut y) = (self.leader(a), self.leader(b));
        if x == y {
            return x;
        }
        // 大きい方を新しい代表元にする。
        if self.parent_or_size[x] > self.parent_or_size[y] {
            std::mem::swap(&mut x, &mut y);
        }
        self.parent_or_size[x] += self.parent_or_size[y];
        self.parent_or_size[y] = x as isize;
        x
    }

    /// `a` と `b` が同じ連結成分に属するかを返す。
    ///
    /// # Panics
    ///
    /// `a` または `b` が範囲外のときパニックする。
    pub fn same(&mut self, a: usize, b: usize) -> bool {
        self.leader(a) == self.leader(b)
    }

    /// `a` が属する連結成分の大きさを返す。
    ///
    /// # Panics
    ///
    /// `a >= self.len()` のときパニックする。
    pub fn size(&mut self, a: usize) -> usize {
        let leader = self.leader(a);
        -self.parent_or_size[leader] as usize
    }

    /// 連結成分ごとに頂点番号を昇順で並べたリストを返す。
    pub fn groups(&mut self) -> Vec<Vec<usize>> {
        let n = self.n;
        let leaders: Vec<usize> = (0..n).map(|i| self.leader(i)).collect();
        let mut groups = vec![Vec::new(); n];
        for (i, &leader) in leaders.iter().enumerate() {
            groups[leader].push(i);
        }
        groups.retain(|group| !group.is_empty());
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::Dsu;

    #[test]
    fn merge_and_same() {
        let mut dsu = Dsu::new(5);
        assert!(!dsu.same(0, 1));
        dsu.merge(0, 1);
        dsu.merge(3, 4);
        assert!(dsu.same(0, 1));
        assert!(dsu.same(3, 4));
        assert!(!dsu.same(1, 3));
        dsu.merge(1, 4);
        assert!(dsu.same(0, 3));
    }

    #[test]
    fn merge_returns_leader() {
        let mut dsu = Dsu::new(3);
        let leader = dsu.merge(0, 2);
        assert_eq!(leader, dsu.leader(0));
        assert_eq!(leader, dsu.leader(2));
        // 既に同じ成分なら代表元が変わらない。
        assert_eq!(dsu.merge(0, 2), leader);
    }

    #[test]
    fn size_tracks_component() {
        let mut dsu = Dsu::new(4);
        assert_eq!(dsu.size(0), 1);
        dsu.merge(0, 1);
        dsu.merge(2, 3);
        assert_eq!(dsu.size(0), 2);
        dsu.merge(1, 2);
        assert_eq!(dsu.size(3), 4);
    }

    #[test]
    fn groups_are_sorted_and_partition_vertices() {
        let mut dsu = Dsu::new(5);
        dsu.merge(0, 4);
        dsu.merge(2, 3);
        let mut groups = dsu.groups();
        groups.sort();
        assert_eq!(groups, vec![vec![0, 4], vec![1], vec![2, 3]]);
    }

    #[test]
    fn empty_dsu() {
        let mut dsu = Dsu::new(0);
        assert!(dsu.is_empty());
        assert_eq!(dsu.len(), 0);
        assert!(dsu.groups().is_empty());
    }

    #[test]
    fn deep_chain_does_not_overflow_stack() {
        let n = 200_000;
        let mut dsu = Dsu::new(n);
        for i in 0..n - 1 {
            dsu.merge(i, i + 1);
        }
        assert_eq!(dsu.size(0), n);
    }
}
