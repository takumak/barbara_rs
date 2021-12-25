extern crate kmp_search;
use kmp_search::kmp_search_all;

use crate::compress::guess_best_token::guess_best_token;

fn split_by_token<'a>(sym: &'a [u8], token: &[u8]) -> Vec<&'a [u8]> {
    let found = kmp_search_all(token, sym);

    let mut result: Vec<&[u8]> = Vec::new();
    let mut last_end = 0;
    for start in found {
        if start > last_end {
            result.push(&sym[last_end..start]);
        }
        last_end = start + token.len();
    }

    if last_end < sym.len() {
        result.push(&sym[last_end..]);
    }
    result
}

pub fn make_dic(syms: Vec<&[u8]>) -> Vec<(Vec<u8>, usize)> {
    use crate::compress::char_counter::CharCounter;

    let mut syms = syms;
    let mut dic: Vec<(Vec<u8>, usize)> = Vec::new();

    let mut chars = CharCounter::new();
    let update_chars = |chars: &mut CharCounter, syms: &Vec<&[u8]>| {
        chars.clear();
        for sym in syms.iter() {
            chars.count_up(sym.iter());
        }
    };
    update_chars(&mut chars, &syms);

    while (chars.len() + dic.len()) < 256 && !syms.is_empty() {
        let (token, score) = guess_best_token(&syms);
        dic.push((token.to_vec(), score));

        let mut newsyms: Vec<&[u8]> = vec![];
        for sym in syms.iter() {
            newsyms.append(&mut split_by_token(sym, &token));
        }
        syms = newsyms;
        update_chars(&mut chars, &syms);
    }

    dic.sort_by(|a, b|
                b.1.cmp(&a.1)
                .then(b.0.len().cmp(&a.0.len())
                      .then(a.0.cmp(&b.0))));

    dic.extend(
        chars
            .iter_by_freq()
            .map(|(c, s)| (vec![c], s)));

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
                (b"_test_".to_vec(), 3),
                // this is local optimum result
                // use strictly_find_best_token() to get best result
                // -> the best second token is (b"bar".to_vec(), 6)
                (b"bar1".to_vec(), 1),
                (b"bar2".to_vec(), 1),
                (b"456".to_vec(), 1),
                (b"foo".to_vec(), 1),
                (b"23".to_vec(), 1),
                (b"1".to_vec(), 1),
            ]
        );
    }
}
