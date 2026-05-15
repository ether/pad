pub fn wrap_line(line: &str, width: u16) -> Vec<(usize, String)> {
    if width == 0 {
        return vec![(0, line.to_string())];
    }
    let w = width as usize;
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return vec![(0, String::new())];
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + w).min(chars.len());
        out.push((i, chars[i..end].iter().collect()));
        i = end;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_wrap_when_fits() {
        let v = wrap_line("hello", 10);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, "hello");
    }

    #[test]
    fn wraps_at_width() {
        let v = wrap_line("abcdefghij", 4);
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].1, "abcd");
        assert_eq!(v[1].1, "efgh");
        assert_eq!(v[2].1, "ij");
    }

    #[test]
    fn empty_line_yields_one_empty_visual() {
        let v = wrap_line("", 10);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].1, "");
    }
}
