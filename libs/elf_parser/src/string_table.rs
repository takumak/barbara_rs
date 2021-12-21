pub fn parse<'a>(strtab: &'a [u8]) -> Vec<&'a str> {
    if strtab.len() == 0 {
        return vec![];
    }

    let strtab =
        if strtab[strtab.len() - 1] == 0 {
            &strtab[..(strtab.len() - 1)]
        } else {
            &strtab
        };

    strtab
        .split(|&c| c == 0)
        .map(|s| core::str::from_utf8(s)
             .unwrap_or("** UTF8 DECODE ERROR **"))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::string_table::parse;

    #[test]
    fn zero_entries() {
        assert_eq!(parse(&[]), Vec::<&str>::new());
    }

    #[test]
    fn single_zero_length_entry() {
        assert_eq!(parse(&[0]), vec![""]);
    }

    #[test]
    fn double_zero_length_entry() {
        assert_eq!(parse(&[0, 0]), vec!["", ""]);
    }

    #[test]
    fn single_entry() {
        assert_eq!(parse(&[b'a', 0]), vec!["a"]);
    }

    #[test]
    fn non_empty_and_empty() {
        assert_eq!(parse(&[b'a', 0, 0]), vec!["a", ""]);
    }

    #[test]
    fn empty_and_non_empty() {
        assert_eq!(parse(&[0, b'a', 0]), vec!["", "a"]);
    }

    #[test]
    fn incomplete() {
        assert_eq!(parse(&[b'a', b'b']), vec!["ab"]);
    }
}
