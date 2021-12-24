pub trait Stpack: Sized {
    const SIZE: usize;
    fn unpack(data: &[u8], le: bool) -> Result<(Self, &[u8]), ()>;
    fn pack(&self, le: bool) -> Vec<u8> {
        if le {
            self.pack_le()
        } else {
            self.pack_be()
        }
    }
    fn pack_le(&self) -> Vec<u8>;
    fn pack_be(&self) -> Vec<u8>;
}

#[macro_export]
macro_rules! stpack {
    {@constructor_from <le> $ftyp:ty} => {
        <$ftyp>::from_le_bytes
    };

    {@constructor_from <be> $ftyp:ty} => {
        <$ftyp>::from_be_bytes
    };

    {@constructor_one <$lebe:ident> $data:ident, $ftyp:ty { $($p:tt)* }} => {
        stpack!{@constructor_from <$lebe> $ftyp}
        (<[u8; core::mem::size_of::<$ftyp>()]>::try_from(
            &$data[(stpack!{@allsize $($p)*})..
                   (stpack!{@allsize $($p)*}+(core::mem::size_of::<$ftyp>()))]
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
        stpack!{@constructor <$lebe> $data {$($result)*}, {$($p)*}, {$vis $fname : $ftyp,}}
    };

    {@constructor <$lebe:ident> $data:ident
     { $($result:tt)* },
     { $($p:tt)* },
     { $vis:vis $fname:ident : $ftyp:ty, $($body:tt)* }} =>
    {
        stpack!{
            @constructor <$lebe> $data
            {$($result)*
             $fname: stpack!{@constructor_one <$lebe> $data, $ftyp { $($p)* }},},
            {$($p)* $vis $fname : $ftyp, },
            { $($body)* }}
    };

    {@pack $self:ident $name:ident $to_bytes:ident $out:ident
     { $($result:tt)* }
     { }} =>
    {
        fn $name(&$self) -> Vec<u8> {
            let mut $out: Vec<u8> = Vec::new();
            $($result)*
            $out
        }
    };

    {@pack $self:ident $name:ident $to_bytes:ident $out:ident
     { $($result:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty }} =>
    {
        stpack!{@pack $self $name $to_bytes $out { $($result)* } { $vis $fname : $ftyp, }}
    };

    {@pack $self:ident $name:ident $to_bytes:ident $out:ident
     { $($result:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty, $($body:tt)* }} =>
    {
        stpack!{
            @pack $self $name $to_bytes $out
            {$($result)*
             $out.extend_from_slice(&$self.$fname.$to_bytes());}
            { $($body)* }}
    };

    {@allsize} => {
        0
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty} => {
        stpack!{@allsize $vis $fname : $ftyp,}
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty, $($body:tt)*} => {
        core::mem::size_of::<$ftyp>() + stpack!{@allsize $($body)*}
    };

    {@impl $stname:ident { $($body:tt)* }} => {
        impl $crate::Stpack for $stname {
            const SIZE: usize = stpack!{@allsize $($body)*};

            fn unpack(data: &[u8], le: bool) -> Result<(Self, &[u8]), ()> {
                if data.len() < Self::SIZE {
                    Err(())
                } else {
                    let (data, right) = data.split_at(Self::SIZE);
                    let r =
                        if le {
                            stpack!{@constructor <le> data { }, { }, { $($body)* }}
                        } else {
                            stpack!{@constructor <be> data { }, { }, { $($body)* }}
                        };

                    Ok((r, right))
                }
            }

            stpack!{@pack self pack_le to_le_bytes result { } { $($body)* }}
            stpack!{@pack self pack_be to_be_bytes result { } { $($body)* }}
        }
    };

    {$(#[$attr:meta])* $vis:vis struct $stname:ident { $($body:tt)* }} => {
        $(#[$attr])*
        $vis struct $stname { $($body)* }
        stpack!{@impl $stname { $($body)* }}
    };
}

#[cfg(test)]
mod tests {
    use crate::Stpack;

    stpack! {
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
        let foo = Foo::unpack(&data, true);
        assert_eq!(
            foo,
            Ok((
                Foo {
                    foo: 0x00,
                    bar: 0x0201,
                    baz: 0x06050403,
                },
                &[] as &[u8]
            ))
        );
        assert_eq!(foo.unwrap().0.pack(true), data);
    }

    #[test]
    fn test_be() {
        let data: Vec<u8> = (0..7).collect();
        let foo = Foo::unpack(&data, false);
        assert_eq!(
            foo,
            Ok((
                Foo {
                    foo: 0x00,
                    bar: 0x0102,
                    baz: 0x03040506,
                },
                &[] as &[u8]
            ))
        );
        assert_eq!(foo.unwrap().0.pack(false), data);
    }

    #[test]
    fn test_remain_data() {
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
