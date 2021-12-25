extern crate stpack;
use stpack::Stpack;

extern crate kmp_search;
use kmp_search::kmp_search_all;

use crate::types::{
    Header,
    AddrTblEntry,
    StrTblOff,
};

fn str_table(data: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    // write offset table
    let mut off: StrTblOff =
        (core::mem::size_of::<StrTblOff>() * data.len()) as StrTblOff;
    for s in data.iter() {
        result.extend_from_slice(&off.to_le_bytes());
        off += s.len() as StrTblOff + 1;
    }

    // write strings
    for s in data.iter() {
        result.push(s.len() as u8);
        result.extend_from_slice(s);
    }

    result
}

fn tokenize(data: &[u8], dic: &Vec<Vec<u8>>) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::new();

    let mut last_end = 0;
    for (i, token) in dic.iter().enumerate() {
        for start in kmp_search_all(token, data) {
            if start > last_end {
                result.append(&mut tokenize(&data[last_end..start], dic));
            }
            result.push(i as u8);
            last_end = start + token.len();
        }
        if last_end > 0 {
            break;
        }
    }

    assert!(last_end > 0);

    if last_end < data.len() {
        result.append(&mut tokenize(&data[last_end..], dic));
    }

    result
}

pub fn pack(symbols: &Vec<(String, u32)>) -> Vec<u8> {
    use crate::compress::make_dic;

    let dic: Vec<Vec<u8>> = make_dic(
        symbols
            .iter()
            .map(|(name, _addr)| name.as_bytes())
            .collect())
        .iter()
        .map(|(tok, _cnt)| tok.clone())
        .collect();

    let names: Vec<Vec<u8>> =
        symbols
        .iter()
        .map(|(name, _addr)| tokenize(name.as_bytes(), &dic))
        .collect();

    let mut name_table: Vec<u8> = str_table(&names);
    let mut token_table: Vec<u8> = str_table(&dic);

    // compose header
    let count = symbols.len() as u32;
    let addr_table_off = Header::SIZE as u32;
    let name_table_off = addr_table_off +
        (core::mem::size_of::<AddrTblEntry>() as u32 * count);
    let token_table_off = name_table_off + name_table.len() as u32;

    let header = Header {
        reserved: 0,
        count,
        addr_table_off,
        name_table_off,
        token_table_off,
    };

    // join all
    let mut result: Vec<u8> = Vec::new();
    result.extend_from_slice(&[0u8; Header::SIZE]);
    header.pack_le(result.as_mut_slice()).unwrap();
    for (_name, addr) in symbols.iter() {
        result.extend_from_slice(
            &(*addr as AddrTblEntry).to_le_bytes());
    }
    result.append(&mut name_table);
    result.append(&mut token_table);
    result
}


#[cfg(test)]
mod tests {
    use crate::pack::{
        str_table,
        tokenize,
    };

    #[test]
    fn str_table_1() {
        assert_eq!(
            str_table(&vec![
                vec![0u8, 1],
                vec![2u8, 3, 4],
                vec![5u8],
            ]),
            vec![
                // offsets
                6u8, 0,
                9, 0,
                13, 0,
                // payload
                2, 0, 1,
                3, 2, 3, 4,
                1, 5,
            ]
        )
    }

    #[test]
    fn tokenize_1() {
        assert_eq!(
            tokenize(
                &[
                    0u8, 1, 2, 3,
                    4, 5, 6, 7,
                    8, 9, 10, 11,
                    12, 13, 14, 15,
                ],
                &vec![
                    vec![4u8, 5, 6, 7], // 0
                    vec![9u8, 10, 11],  // 1
                    vec![14u8, 15],     // 2
                    vec![12u8, 13],     // 3
                    vec![0u8],          // 4
                    vec![2u8],          // 5
                    vec![1u8],          // 6
                    vec![3u8],          // 7
                    vec![8u8],          // 8
                ]
            ),
            vec![4u8, 6, 5, 7, 0, 8, 1, 3, 2]
        )
    }
}
