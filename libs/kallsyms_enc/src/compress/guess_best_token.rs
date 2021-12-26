extern crate kmp_search;
use kmp_search::kmp_search_all;

fn calc_score(len: usize, count: usize) -> usize {
    let plain_len = len * count;
    let compressed_len = 1 + len + count;
    if compressed_len > plain_len {
        (1usize << (usize::BITS - 1)) - (compressed_len - plain_len)
    } else {
        (1usize << (usize::BITS - 1)) + (plain_len - compressed_len)
    }
}

fn enlarge<'a>(
    symbols: &'a [&'a [u8]],
    mut token: &'a [u8],
    count: usize,
    right: bool,
) -> (&'a [u8], usize) {
    let mut new_scores: Vec<(&[u8], usize, usize)> =
        vec![(token, count, calc_score(token.len(), count))];

    loop {
        let mut tbl: [(&[u8], usize); 256] = [(&[], 0); 256];

        for sym in symbols {
            let subject = if right {
                &sym[..(sym.len() - 1)]
            } else {
                &sym[1..]
            };

            let positions = kmp_search_all(&token, subject);
            for i in positions {
                let tok_l = i;
                let tok_r = tok_l + token.len();

                let c = sym[if right { tok_r } else { tok_l }] as usize;
                tbl[c].0 = &sym[tok_l..=tok_r];
                tbl[c].1 += 1;
            }
        }

        let mut best_i = 0usize;
        for i in 0..tbl.len() {
            if tbl[i].1 > tbl[best_i].1 {
                best_i = i;
            }
        }

        let (newtoken, cnt) = tbl[best_i];
        if cnt == 0 {
            break;
        }

        new_scores.push((newtoken, cnt, calc_score(newtoken.len(), cnt)));
        token = newtoken;
    }

    let mut max_i = 0usize;
    for i in 1..new_scores.len() {
        if new_scores[i].2 >= new_scores[max_i].2 {
            max_i = i;
        }
    }

    let (tok, cnt, _score) = new_scores[max_i];
    // println!("{} ({})", std::str::from_utf8(tok).unwrap(), cnt);
    (tok, cnt)
}

pub fn guess_best_token<'a>(symbols: &'a [&'a [u8]]) -> (&'a [u8], usize) {
    let mut tbl: [(&[u8], usize); 256] = [(&[], 0); 256];

    for sym in symbols.iter() {
        for (sym_i, chr) in sym.iter().enumerate() {
            let tbl_i = *chr as usize;
            tbl[tbl_i] = (&sym[sym_i..(sym_i + 1)], tbl[tbl_i].1 + 1);
        }
    }

    let mut max_i = 0usize;
    for i in 1..tbl.len() {
        if tbl[i].1 > tbl[max_i].1 {
            max_i = i;
        }
    }

    let (mut token, mut count) = tbl[max_i];
    (token, count) = enlarge(symbols, token, count, false);
    (token, count) = enlarge(symbols, token, count, true);
    (token, count)
}

#[allow(dead_code)]
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
            let cmp = score.cmp(&best_score).then(
                token
                    .len()
                    .cmp(&best_token.len())
                    .then(token.cmp(&best_token).reverse()),
            );
            if cmp.is_gt() {
                best_token = token;
                best_score = score;
            }
        }
    }

    (best_token, best_score / best_token.len())
}

#[cfg(test)]
mod tests {
    use crate::compress::guess_best_token::{guess_best_token, strictly_find_best_token};

    #[test]
    fn guess_1() {
        let data: &[&[u8]] = &[
            b"aaa_test_common_token_111",
            b"bbb_test_common_token_222",
            b"ccc_test_common_token_333",
            b"ddd_test_common_token_444",
            b"eee_test_common_token_555",
        ];

        assert_eq!(guess_best_token(data), (&b"_test_common_token_"[..], 5))
    }

    #[test]
    fn guess_local_optimum_1() {
        let data: &[&[u8]] = &[b"123abc", b"456abc", b"789abc"];

        assert_eq!(guess_best_token(data), (&b"123abc"[..], 1))
    }

    #[test]
    fn strict_1() {
        let data: &[&[u8]] = &[b"123abc", b"456abc", b"789abc"];

        assert_eq!(strictly_find_best_token(data), (&b"abc"[..], 3))
    }
}
