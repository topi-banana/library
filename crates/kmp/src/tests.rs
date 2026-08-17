use super::*;

#[test]
fn test_kmp_search() {
    let pattern = vec!['a', 'b', 'c'];
    let kmp = KMP::new(&pattern);
    let text = vec!['a', 'b', 'a', 'b', 'c', 'a', 'b', 'c', 'a'];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![2, 5]);
}

#[test]
fn test_no_match() {
    let pattern = vec!['x', 'y', 'z'];
    let kmp = KMP::new(&pattern);
    let text = vec!['a', 'b', 'c', 'd', 'e'];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![]);
}

#[test]
fn test_empty_pattern() {
    let pattern: Vec<char> = vec![];
    let kmp = KMP::new(&pattern);
    let text = vec!['a', 'b', 'c'];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![]);
}

#[test]
fn test_empty_text() {
    let pattern = vec!['a'];
    let kmp = KMP::new(&pattern);
    let text: Vec<char> = vec![];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![]);
}

#[test]
fn test_empty_empty() {
    let pattern = vec![];
    let kmp = KMP::new(&pattern);
    let text: Vec<char> = vec![];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![]);
}

#[test]
fn test_bigger_pattern() {
    let pattern = vec!['a', 'b', 'c', 'd', 'e'];
    let kmp = KMP::new(&pattern);
    let text: Vec<char> = vec!['a', 'b'];
    let result: Vec<_> = kmp.search(&text).collect();
    assert_eq!(result, vec![]);
}
