#![cfg_attr(not(test), no_std)]

// #![feature(trace_macros)]
// trace_macros!(true);

pub trait Stpack: Sized {
    const SIZE: usize;
    fn unpack(data: &[u8], le: bool) -> Result<Self, ()> {
        if le {
            Self::unpack_le(data)
        } else {
            Self::unpack_be(data)
        }
    }
    fn unpack_le(data: &[u8]) -> Result<Self, ()>;
    fn unpack_be(data: &[u8]) -> Result<Self, ()>;

    fn pack(&self, buf: &mut [u8], le: bool) -> Result<(), ()> {
        if le {
            self.pack_le(buf)
        } else {
            self.pack_be(buf)
        }
    }
    fn pack_le(&self, buf: &mut [u8]) -> Result<(), ()>;
    fn pack_be(&self, buf: &mut [u8]) -> Result<(), ()>;
}

#[macro_export]
macro_rules! stpack {
    // unpack

    {@unpack $self:ident $name:ident $from_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { }} =>
    {
        fn $name($buf: &[u8]) -> Result<Self, ()> {
            if $buf.len() < Self::SIZE {
                Err(())
            } else {
                Ok(Self {
                    $($result)*
                })
            }
        }
    };

    {@unpack $self:ident $name:ident $from_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty }} =>
    {
        stpack!{@unpack $self $name $from_bytes $buf { $($result)* } { $($p)* } { $vis $fname : $ftyp, }}
    };

    {@unpack $self:ident $name:ident $from_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty, $($body:tt)* }} =>
    {
        stpack!{
            @unpack $self $name $from_bytes $buf
            { $($result)*
              $fname: <$ftyp>::$from_bytes(
                  <[u8; core::mem::size_of::<$ftyp>()]>::try_from(
                      &$buf[(stpack!{@allsize $($p)*})..
                            (stpack!{@allsize $($p)*}+(core::mem::size_of::<$ftyp>()))])
                      .unwrap()), }
            { $($p)* $vis $fname : $ftyp,  }
            { $($body)* }}
    };

    // pack

    {@pack $self:ident $name:ident $to_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { }} =>
    {
        fn $name(&$self, $buf: &mut [u8]) -> Result<(), ()> {
            if $buf.len() < Self::SIZE {
                Err(())
            } else {
                {$($result)*}
                Ok(())
            }
        }
    };

    {@pack $self:ident $name:ident $to_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty }} =>
    {
        stpack!{@pack $self $name $to_bytes $buf { $($result)* } { $($p)* } { $vis $fname : $ftyp, }}
    };

    {@pack $self:ident $name:ident $to_bytes:ident $buf:ident
     { $($result:tt)* }
     { $($p:tt)* }
     { $vis:vis $fname:ident : $ftyp:ty, $($body:tt)* }} =>
    {
        stpack!{
            @pack $self $name $to_bytes $buf
            { $($result)*
              <&mut [u8; core::mem::size_of::<$ftyp>()]>::try_from(
                  &mut $buf[(stpack!{@allsize $($p)*})..
                            (stpack!{@allsize $($p)*}+(core::mem::size_of::<$ftyp>()))])
              .unwrap().copy_from_slice(&$self.$fname.$to_bytes()); }
            { $($p)* $vis $fname : $ftyp,  }
            { $($body)* }}
    };

    // size calculator

    {@allsize} => {
        0
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty} => {
        stpack!{@allsize $vis $fname : $ftyp,}
    };

    {@allsize $vis:vis $fname:ident : $ftyp:ty, $($body:tt)*} => {
        core::mem::size_of::<$ftyp>() + stpack!{@allsize $($body)*}
    };

    // impl

    {@impl $stname:ident { $($body:tt)* }} => {
        impl Stpack for $stname {
            const SIZE: usize = stpack!{@allsize $($body)*};
            stpack!{@unpack self unpack_le from_le_bytes buf { } { } { $($body)* }}
            stpack!{@unpack self unpack_be from_be_bytes buf { } { } { $($body)* }}
            stpack!{@pack self pack_le to_le_bytes buf { } { } { $($body)* }}
            stpack!{@pack self pack_be to_be_bytes buf { } { } { $($body)* }}
        }
    };

    // entrypoint

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
    fn test_size() {
        assert_eq!(Foo::SIZE, 7);
    }

    #[test]
    fn foo_le() {
        let data: Vec<u8> = (0..7).collect();
        let foo = Foo::unpack(&data, true).unwrap();
        assert_eq!(
            foo,
            Foo {
                foo: 0x00,
                bar: 0x0201,
                baz: 0x06050403,
            }
        );

        let mut buf: [u8; 7] = [0; 7];
        foo.pack(&mut buf, true).unwrap();
        assert_eq!(buf.to_vec(), data);
        foo.pack(&mut buf, false).unwrap();
        assert_ne!(buf.to_vec(), data);
    }

    #[test]
    fn foo_be() {
        let data: Vec<u8> = (0..7).collect();
        let foo = Foo::unpack(&data, false).unwrap();
        assert_eq!(
            foo,
            Foo {
                foo: 0x00,
                bar: 0x0102,
                baz: 0x03040506,
            }
        );

        let mut buf: [u8; 7] = [0; 7];
        foo.pack(&mut buf, false).unwrap();
        assert_eq!(buf.to_vec(), data);
        foo.pack(&mut buf, true).unwrap();
        assert_ne!(buf.to_vec(), data);
    }

    #[test]
    fn size_too_small() {
        let mut data: [u8; 6] = [0; 6];
        let foo = Foo {
            foo: 0,
            bar: 0,
            baz: 0,
        };
        assert_eq!(Foo::unpack_le(&data), Err(()));
        assert_eq!(foo.pack_le(&mut data), Err(()));
    }

    #[test]
    fn test_debug() {
        let data: Vec<u8> = (0..7).collect();
        let foo = Foo::unpack_be(&data).unwrap();
        assert_eq!(
            format!("{:?}", foo),
            "Foo { foo: 0, bar: 258, baz: 50595078 }"
        );
    }
}
