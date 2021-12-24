pub(crate) fn read_at<'a>(strtab: &'a [u8], offset: usize) -> &'a [u8] {
    if offset >= strtab.len() {
        return &[];
    }

    let right_all = &strtab[offset..];
    right_all.split(|c| *c == 0).next().unwrap_or(right_all)
}

#[cfg(test)]
mod tests {
    use crate::raw::strtab::read_at;

    #[test]
    fn empty() {
        assert_eq!(read_at(&[], 0), &[]);
        assert_eq!(read_at(&[], 1), &[]);
        assert_eq!(read_at(&[0], 0), &[]);
    }

    #[test]
    fn single_entry() {
        assert_eq!(read_at(&[b'a'], 0), &[b'a']);
        assert_eq!(read_at(&[b'a'], 1), &[]);
        assert_eq!(read_at(&[b'a'], 2), &[]);
        assert_eq!(read_at(&[b'a', 0], 0), &[b'a']);
        assert_eq!(read_at(&[b'a', 0], 1), &[]);
        assert_eq!(read_at(&[b'a', 0], 2), &[]);
    }

    #[test]
    fn two_entries() {
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 0), &[b'a']);
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 1), &[]);
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 2), &[b'c']);
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 3), &[]);
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 4), &[]);
        assert_eq!(read_at(&[b'a', 0, b'c', 0], 5), &[]);
    }
}
