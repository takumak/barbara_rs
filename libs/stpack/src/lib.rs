pub trait Unpacker: Sized {
    const SIZE: usize;
    fn unpack(data: &[u8], le: bool) -> Result<(Self, &[u8]), ()>;
}

#[macro_export]
macro_rules! unpacker {
    {@constructor_from <le> $ftyp:ty} => {
        <$ftyp>::from_le_bytes
    };

    {@constructor_from <be> $ftyp:ty} => {
        <$ftyp>::from_be_bytes
    };

    {@constructor_one <$lebe:ident> $data:ident, $ftyp:ty { $($p:tt)* }} => {
        unpacker!{@constructor_from <$lebe> $ftyp}
        (<[u8; core::mem::size_of::<$ftyp>()]>::try_from(
            &$data[(unpacker!{@allsize $($p)*})..
                   (unpacker!{@allsize $($p)*}+(core::mem::size_of::<$ftyp>()))]
        ).unwrap())
    };

    {@constructor <$lebe:ident> $data:ident
     { $($result:tt)* },
     { $($p:tt)* },
     { }} =>
    {
        Self { $($result)* }
    };

    {@constructor <$lebe:ident> $data:ident
     { $($result:tt)* },
     { $($p:tt)* },
     { $vis:vis $fname:ident : $ftyp:ty }} =>
    {
        unpacker!{@constructor <$lebe> $data {$($result)*}, {$($p)*}, {$vis $fname : $ftyp,}}
    };

    {@constructor <$lebe:ident> $data:ident
     { $($result:tt)* },
     { $($p:tt)* },
     { $vis:vis $fname:ident : $ftyp:ty, $($body:tt)* }} =>
    {
        unpacker!{
            @constructor <$lebe> $data
            {$($result)*
             $fname: unpacker!{@constructor_one <$lebe> $data, $ftyp { $($p)* }},},
            {$($p)* $vis $fname : $ftyp, },
            { $($body)* }}
    };

    {@allsize} => {
        0
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty} => {
        unpacker!{@allsize $vis $fname : $ftyp,}
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty, $($body:tt)*} => {
        core::mem::size_of::<$ftyp>() + unpacker!{@allsize $($body)*}
    };

    {@impl $stname:ident { $($body:tt)* }} => {
        impl $crate::Unpacker for $stname {
            const SIZE: usize = unpacker!{@allsize $($body)*};

            fn unpack(data: &[u8], le: bool) -> Result<(Self, &[u8]), ()> {
                if data.len() < Self::SIZE {
                    Err(())
                } else {
                    let (data, right) = data.split_at(Self::SIZE);
                    let r =
                        if le {
                            unpacker!{@constructor <le> data { }, { }, { $($body)* }}
                        } else {
                            unpacker!{@constructor <be> data { }, { }, { $($body)* }}
                        };

                    Ok((r, right))
                }
            }
        }
    };

    {$(#[$attr:meta])* $vis:vis struct $stname:ident { $($body:tt)* }} => {
        $(#[$attr])*
        $vis struct $stname { $($body)* }
        unpacker!{@impl $stname { $($body)* }}
    };
}

#[cfg(test)]
mod tests {
    use crate::Unpacker;

    unpacker! {
        #[derive(PartialEq, Eq, Debug)]
        struct Foo {
            foo: u8,
            pub bar: u16,
            baz: u32,
        }
    }

    #[test]
    fn foo_size() {
        assert_eq!(Foo::SIZE, 7);
    }

    #[test]
    fn foo_le() {
        let data: Vec<u8> = (0..7).collect();
        assert_eq!(
            Foo::unpack(&data, true),
            Ok((
                Foo {
                    foo: 0x00,
                    bar: 0x0201,
                    baz: 0x06050403,
                },
                &[] as &[u8]
            ))
        );
    }

    #[test]
    fn test_be() {
        let data: Vec<u8> = (0..10).collect();
        assert_eq!(
            Foo::unpack(&data, false),
            Ok((
                Foo {
                    foo: 0x00,
                    bar: 0x0102,
                    baz: 0x03040506,
                },
                &[7u8, 8u8, 9u8] as &[u8]
            ))
        );
    }

    #[test]
    fn test_size_too_small() {
        let data: Vec<u8> = (0..6).collect();
        assert_eq!(
            Foo::unpack(&data, false),
            Err(())
        );
    }

    #[test]
    fn test_debug() {
        let data: Vec<u8> = (0..7).collect();
        let (foo, _) = Foo::unpack(&data, false).unwrap();
        assert_eq!(format!("{:?}", foo), "Foo { foo: 0, bar: 258, baz: 50595078 }");
    }
}
