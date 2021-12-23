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
    use crate::compress::guess_best_token::strictly_find_best_token;

    #[test]
    fn test1() {
        let data: &[&[u8]] = &[
            b"aaa_test_common_token_111",
            b"bbb_test_common_token_222",
            b"ccc_test_common_token_333",
            b"ddd_test_common_token_444",
            b"eee_test_common_token_555",
        ];

        assert_eq!(
            strictly_find_best_token(data),
            (&b"_test_common_token_"[..],
             "_test_common_token_".len() * 5)
        )
    }
}
