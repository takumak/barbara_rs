pub fn read_one_from_offset<'a>(strtab: &'a [u8], offset: usize) -> &'a str {
    if offset >= strtab.len() {
        return "";
    }

    let right_all = &strtab[offset..];
    let target = right_all.split(|c| *c == 0).next().unwrap_or(right_all);
    core::str::from_utf8(target)
        .unwrap_or("** UTF8 DECODE ERROR **")
}

#[cfg(test)]
mod tests {
    use crate::string_table::read_one_from_offset;

    #[test]
    fn empty() {
        assert_eq!(read_one_from_offset(&[], 0), "");
        assert_eq!(read_one_from_offset(&[], 1), "");
        assert_eq!(read_one_from_offset(&[0], 0), "");
    }

    #[test]
    fn single_entry() {
        assert_eq!(read_one_from_offset(&[b'a'], 0), "a");
        assert_eq!(read_one_from_offset(&[b'a'], 1), "");
        assert_eq!(read_one_from_offset(&[b'a'], 2), "");
        assert_eq!(read_one_from_offset(&[b'a', 0], 0), "a");
        assert_eq!(read_one_from_offset(&[b'a', 0], 1), "");
        assert_eq!(read_one_from_offset(&[b'a', 0], 2), "");
    }

    #[test]
    fn two_entries() {
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 0), "a");
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 1), "");
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 2), "c");
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 3), "");
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 4), "");
        assert_eq!(read_one_from_offset(&[b'a', 0, b'c', 0], 5), "");
    }
}
