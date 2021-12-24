extern crate kmp_search;
use kmp_search::kmp_search_all;

use crate::compress::char_counter::CharCounter;

fn enlarge(symbols: &[&[u8]], mut token: Vec<u8>, count: usize, right: bool) -> (Vec<u8>, usize) {
    let mut new_scores: Vec<(usize, usize)> =
        vec![(token.len() * count, count)];

    loop {
        let mut tbl: [usize; 256] = [0; 256];

        for sym in symbols {
            let mut positions = kmp_search_all(
                &token,
                if right {
                    &sym[..(sym.len() - 1)]
                } else {
                    &sym[1..]
                }
            );
            for i in positions {
                let c = sym[if right { i + token.len() } else { i }];
                tbl[c as usize] += 1;
            }
        }

        let mut best_chr = 0u8;
        let mut best_cnt = 0usize;
        for i in 0..tbl.len() {
            if tbl[i] > best_cnt {
                best_chr = i as u8;
                best_cnt = tbl[i];
            }
        }
        if best_cnt == 0 {
            break;
        }

        if right {
            token.push(best_chr);
        } else {
            token.insert(0, best_chr);
        }
        new_scores.push((token.len() * best_cnt, best_cnt));
    }

    let mut max_i = 0usize;
    for i in 1..new_scores.len() {
        if new_scores[i].0 >= new_scores[max_i].0 {
            max_i = i;
        }
    }

    if right {
        let r = token.len() - (new_scores.len() - 1) + max_i;
        (token[..r].to_vec(), new_scores[max_i].1)
    } else {
        let l = new_scores.len() - 1 - max_i;
        (token[l..].to_vec(), new_scores[max_i].1)
    }
}

pub fn guess_best_token<'a>(symbols: &'a [&'a [u8]]) -> (Vec<u8>, usize) {
    let mut counter = CharCounter::new();
    for sym in symbols {
        counter.count_up(sym.iter());
    }

    let (chr, mut count) = counter.most_one().unwrap();
    let mut token = vec![chr];

    (token, count) = enlarge(symbols, token, count, false);
    (token, count) = enlarge(symbols, token, count, true);

    (token, count)
}

pub fn strictly_find_best_token<'a>(symbols: &'a [&'a [u8]]) -> (&'a [u8], usize) {
    let mut suffix_array: Vec<&[u8]> = vec![];
    for sym in symbols {
        for i in 0..sym.len() {
            suffix_array.push(&sym[i..])
        }
    }
    suffix_array.sort();

    let mut best_token: &[u8] = &[];
    let mut best_score = 0usize;
    for (i, suffix) in suffix_array.iter().enumerate() {
        for j in 1..=suffix.len() {
            let token = &suffix[..j];

            let mut top = i;
            let mut bot = i;
            for k in (0..i).rev() {
                if !suffix_array[k].starts_with(token) {
                    break;
                }
                top = k;
            }
            for k in (i + 1)..suffix_array.len() {
                if !suffix_array[k].starts_with(token) {
                    break;
                }
                bot = k;
            }

            let cnt = bot - top + 1;

            let score = token.len() * cnt;
            let cmp = score
                .cmp(&best_score)
                .then(token.len().cmp(&best_token.len())
                      .then(token.cmp(&best_token).reverse()));
            if cmp.is_gt() {
                best_token = token;
                best_score = score;
            }
        }
    }

    (best_token, best_score)
}

#[cfg(test)]
mod tests {
    use crate::compress::guess_best_token::{
        guess_best_token,
        strictly_find_best_token,
    };

    #[test]
    fn guess_1() {
        let data: &[&[u8]] = &[
            b"aaa_test_common_token_111",
            b"bbb_test_common_token_222",
            b"ccc_test_common_token_333",
            b"ddd_test_common_token_444",
            b"eee_test_common_token_555",
        ];

        assert_eq!(
            guess_best_token(data),
            (b"_test_common_token_".to_vec(), 5)
        )
    }

    #[test]
    fn guess_local_optimum_1() {
        let data: &[&[u8]] = &[
            b"123abc",
            b"456abc",
            b"789abc",
        ];

        assert_eq!(
            guess_best_token(data),
            (b"123abc".to_vec(), 1)
        )
    }

    #[test]
    fn strict_1() {
        let data: &[&[u8]] = &[
            b"123abc",
            b"456abc",
            b"789abc",
        ];

        assert_eq!(
            strictly_find_best_token(data),
            (&b"abc"[..], 9usize)
        )
    }
}
