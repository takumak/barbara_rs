#![cfg_attr(not(test), no_std)]

pub fn kmp_search(pattern: &[u8], subject: &[u8]) -> Option<usize> {
    let firstchar = pattern[0];
    let mut m = 0;
    while m + pattern.len() <= subject.len() {
        let mut next_m = m;
        for i in 0..pattern.len() {
            if subject[m + i] == firstchar {
                next_m = m + i;
            }

            if subject[m + i] != pattern[i] {
                if next_m == m {
                    next_m = m + i;
                }
                break;
            }

            if i == pattern.len() - 1 {
                return Some(m);
            }
        }

        if next_m == m {
            next_m += 1;
        }

        m = next_m
    }

    None
}

struct KMPSearchAll<'a> {
    pattern: &'a [u8],
    subject: &'a [u8],
    curr: usize,
}

impl<'a> Iterator for KMPSearchAll<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr + self.pattern.len() > self.subject.len() {
            None
        } else {
            match kmp_search(self.pattern, &self.subject[self.curr..]) {
                Some(i) => {
                    let pos = self.curr + i;
                    self.curr = pos + self.pattern.len();
                    Some(pos)
                },
                None => {
                    self.curr = self.subject.len();
                    None
                }
            }
        }
    }
}

pub fn kmp_search_all<'a>(pattern: &'a [u8], subject: &'a [u8]) ->
    impl Iterator<Item = usize> + 'a
{
    KMPSearchAll {
        pattern,
        subject,
        curr: 0,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        kmp_search,
        kmp_search_all,
    };

    #[test]
    #[should_panic]
    fn pattern_is_empty() {
        kmp_search(&[], &[]);
    }

    #[test]
    fn subject_is_empty() {
        assert_eq!(kmp_search(&"a".as_bytes(), &[]), None);
    }

    #[test]
    fn match_at_0() {
        assert_eq!(kmp_search(&"ab".as_bytes(), &"ab".as_bytes()), Some(0));
    }

    #[test]
    fn match_at_1() {
        assert_eq!(kmp_search(&"ab".as_bytes(), &"aab".as_bytes()), Some(1));
    }

    #[test]
    fn not_match() {
        assert_eq!(kmp_search(&"ab".as_bytes(), &"aaa".as_bytes()), None);
    }

    #[test]
    fn mismatch_and_match() {
        assert_eq!(kmp_search(&"abc".as_bytes(), &"ab abcd".as_bytes()), Some(3));
    }

    #[test]
    fn all_not_match() {
        assert_eq!(
            kmp_search_all(&"ab".as_bytes(), &"aaa".as_bytes())
                .collect::<Vec<usize>>(),
            vec![]
        );
    }

    #[test]
    fn all_match_one() {
        assert_eq!(
            kmp_search_all(&"aa".as_bytes(), &"aaa".as_bytes())
                .collect::<Vec<usize>>(),
            vec![0]
        );
    }

    #[test]
    fn all_match_two() {
        assert_eq!(
            kmp_search_all(&"aa".as_bytes(), &"aaaa".as_bytes())
                .collect::<Vec<usize>>(),
            vec![0, 2]
        );
    }

    #[test]
    fn all_match_two_2() {
        assert_eq!(
            kmp_search_all(&"ab".as_bytes(), &"aabaab".as_bytes())
                .collect::<Vec<usize>>(),
            vec![1, 4]
        );
    }
}
