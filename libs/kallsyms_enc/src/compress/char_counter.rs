pub struct CharCounter {
    table: [usize; 256],
}

impl CharCounter {
    pub fn new() -> Self {
        Self { table: [0; 256] }
    }

    pub fn len(&self) -> usize {
        self.table.iter().filter(|c| **c > 0).count()
    }

    pub fn clear(&mut self) {
        self.table = [0; 256];
    }

    pub fn count_up<'a>(&mut self, bytes: impl Iterator<Item = &'a u8>) {
        for c in bytes {
            self.table[*c as usize] += 1;
        }
    }

    pub fn iter_by_freq(&self) -> impl Iterator<Item = (u8, usize)> {
        let mut chr_cnt: Vec<(u8, usize)> = self
            .table
            .iter()
            .enumerate()
            .map(|(c, s)| (c as u8, *s))
            .collect();
        chr_cnt.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        chr_cnt.into_iter().take_while(|(_c, s)| *s != 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::compress::char_counter::CharCounter;

    #[test]
    fn test1() {
        let mut counter = CharCounter::new();
        counter.count_up(
            b"Lorem ipsum dolor sit amet, consectetur \
              adipiscing elit, sed do eiusmod tempor \
              incididunt ut labore et dolore magna aliqua. \
              Ut enim ad minim veniam, quis nostrud \
              exercitation ullamco laboris nisi ut aliquip \
              ex ea commodo consequat. Duis aute irure \
              dolor in reprehenderit in voluptate velit \
              esse cillum dolore eu fugiat nulla pariatur. \
              Excepteur sint occaecat cupidatat non \
              proident, sunt in culpa qui officia deserunt \
              mollit anim id est laborum."
                .iter(),
        );

        let freq_chars: Vec<(u8, usize)> = counter.iter_by_freq().collect();

        assert_eq!(
            freq_chars,
            vec![
                (b' ', 68usize),
                (b'i', 42usize),
                (b'e', 37usize),
                (b't', 32usize),
                (b'a', 29usize),
                (b'o', 29usize),
                (b'u', 28usize),
                (b'n', 24usize),
                (b'r', 22usize),
                (b'l', 21usize),
                (b'd', 18usize),
                (b's', 18usize),
                (b'm', 17usize),
                (b'c', 16usize),
                (b'p', 11usize),
                (b'q', 5usize),
                (b',', 4usize),
                (b'.', 4usize),
                (b'b', 3usize),
                (b'f', 3usize),
                (b'g', 3usize),
                (b'v', 3usize),
                (b'x', 3usize),
                (b'D', 1usize),
                (b'E', 1usize),
                (b'L', 1usize),
                (b'U', 1usize),
                (b'h', 1usize),
            ]
        );
    }

    #[test]
    fn test_empty() {
        let counter = CharCounter::new();

        let freq_chars: Vec<(u8, usize)> = counter.iter_by_freq().collect();

        assert_eq!(freq_chars, vec![]);
    }
}
