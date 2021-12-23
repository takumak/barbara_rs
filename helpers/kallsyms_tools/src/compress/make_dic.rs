use crate::compress::guess_best_token::strictly_find_best_token;

fn split_by_token<'a>(sym: &'a [u8], token: &[u8]) -> Vec<&'a [u8]> {
    if sym.len() == 0 {
        return vec![];
    } else if token.len() > sym.len() {
        return vec![sym];
    }
    // todo: performance improvement needed
    match sym.windows(token.len()).position(|w| w == token) {
        Some(pos) => {
            let mut r =
                if pos == 0 {
                    vec![]
                } else {
                    vec![&sym[..pos]]
                };
            r.append(&mut split_by_token(&sym[(pos + token.len())..], token));
            r
        },
        None => vec![sym],
    }
}

pub fn make_dic(syms: Vec<&[u8]>) -> Vec<(Vec<u8>, usize)> {
    let mut syms = syms;
    let mut dic: Vec<(Vec<u8>, usize)> = vec![];

    while !syms.is_empty() {
        let (token, score) = strictly_find_best_token(&syms);
        dic.push((token.to_vec(), score));

        let mut newsyms: Vec<&[u8]> = vec![];
        for sym in syms.iter() {
            newsyms.append(&mut split_by_token(sym, &token));
        }
        syms = newsyms;
    }

    dic.sort_by(|a, b|
                b.1.cmp(&a.1)
                .then(b.0.len().cmp(&a.0.len())
                      .then(a.0.cmp(&b.0))));

    dic
}

#[cfg(test)]
mod tests {
    use crate::compress::make_dic::make_dic;

    #[test]
    fn test() {
        let syms: Vec<&[u8]> = vec![
            b"foo_test_1",
            b"bar1_test_23",
            b"bar2_test_456",
        ];

        assert_eq!(
            make_dic(syms),
            vec![
                (b"_test_".to_vec(), 18),
                (b"bar".to_vec(), 6),
                (b"456".to_vec(), 3),
                (b"foo".to_vec(), 3),
                (b"23".to_vec(), 2),
                (b"1".to_vec(), 2),
                (b"2".to_vec(), 1),
            ]
        );
    }
}
