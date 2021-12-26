extern crate stpack;
use stpack::{stpack, Stpack};

stpack! {
    pub struct Header {
        pub reserved: u32,
        pub count: u16,
        pub addr_table_off: u16,
        pub name_table_off: u16,
        pub token_table_off: u16,
    }
}

pub type AddrTblEntry = u32;
pub type StrTblOff = u16;

/*
 *                        --------- .---------------------------.
 *                         ^  ^  ^  |  reserved: u32            |
 *                         |  |  |  |  sym_count: u32           |
 *                         |  |  |  |  addr_table_off: u32      |
 *                         |  |  |  |  name_table_off: u32      |
 *                         v  |  |  |  token_table_off: u32     |
 *         addr_table_off --- |  |  +===========================+
 *                            |  |  |  addr[0]: AddrTblEntry    |
 *                            |  |  |  addr[1]                  |
 *                            |  |  |    ..                     |
 *                            v  |  |  addr[sym_count - 1]      |
 *         name_table_off ------ |  +===========================+ -------
 *                               |  |  name_off[0]: StrTblOff   |   ^  ^
 *                               |  |  name_off[1]              |   |  |
 *                               |  |    ..                     |   |  |
 *                               |  |  name_off[sym_count - 1]  |   |  |
 *                               |  +===========================+   v  |
 *                               |  |    +----------------+ -----------+- name_off[0]
 *                               |  |    | len: u8        |     |      |
 *                               |  |    | token[0]: u8   |     |      |
 *                               |  |    | token[1]       |     |      |
 *                               |  |    |   ..           |     |      |
 *                               |  |    | token[len - 1] |     |      v
 *                               |  |    +----------------+ ------------- name_off[1]
 *                               |  |    | len            |     |
 *                               |  |    | token[0]       |     |
 *                               |  |    | token[1]       |     |
 *                               |  |    |   ..           |     |
 *                               |  |    | token[len - 1] |     |
 *                               |  |    +----------------+     |
 *                               |  |    |       ..       |     |
 *                               v  |    +----------------+     |
 *        token_table_off --------- +===========================+ -------
 *                                  |  token_off[0]: StrTblOff  |   ^  ^
 *                                  |  token_off[1]             |   |  |
 *                                  |    ..                     |   |  |
 *                                  |  token_off[N-1]           |   |  |
 *                                  +===========================+   v  |
 *                                  |    +----------------+ -----------+- token_off[0]
 *                                  |    | len: u8        |     |      |
 *                                  |    | byte[0]: u8    |     |      |
 *                                  |    | byte[1]        |     |      |
 *                                  |    |   ..           |     |      |
 *                                  |    | byte[len - 1]  |     |      v
 *                                  |    +----------------+ ------------- token_off[1]
 *                                  |    | len            |     |
 *                                  |    | token[0]       |     |
 *                                  |    | token[1]       |     |
 *                                  |    |   ..           |     |
 *                                  |    | token[len - 1] |     |
 *                                  |    +----------------+     |
 *                                  |    |       ..       |     |
 *                                  |    +----------------+     |
 *                                  '==========================='
 */
