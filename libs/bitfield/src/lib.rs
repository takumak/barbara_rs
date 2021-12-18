#![cfg_attr(not(test), no_std)]

#[macro_export]
macro_rules! bitfield {

    {@mask $left:literal, $right:literal} => {
        ((2 << $left) - 1) ^ ((1 << $right) - 1)
    };

    {@field $stname:ident, $typ:ty { }} => { };

    {@field $stname:ident, $typ:ty { $name:ident[$left:literal : $right:literal]; $($remain:tt)* }} => {
        const $name: $stname = $stname(bitfield!{@mask $left, $right});
        bitfield!{@field $stname, $typ { $($remain)* }}
    };

    {@field $stname:ident, $typ:ty { $name:ident[$bit:literal]; $($remain:tt)* }} => {
        const $name: $stname = $stname(bitfield!{@mask $bit, $bit});
        bitfield!{@field $stname, $typ { $($remain)* }}
    };

    {@impl $stname:ident, $typ:ty { $($body:tt)* }} => {
        impl $stname {
            bitfield!{@field $stname, $typ {$($body)*}}

            fn is_set(&self, m: $stname) -> bool {
                (self.0 & m.0) == m.0
            }

            fn extract(&self, m: $stname) -> $typ {
                (self.0 & m.0) >> m.0.trailing_zeros()
            }

            fn compose(&self, v: $typ) -> $stname {
                $stname(v << self.0.trailing_zeros())
            }
        }

        impl From<$typ> for $stname {
            fn from(v: $typ) -> Self {
                Self(v)
            }
        }

        impl From<$stname> for $typ {
            fn from(v: $stname) -> Self {
                v.0
            }
        }

        impl core::ops::BitOr for $stname {
            type Output = $stname;
            fn bitor(self, rhs: $stname) -> $stname {
                $stname(self.0 | rhs.0)
            }
        }
    };

    {$stname:ident : $typ:ty { $($body:tt)* }} => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        struct $stname($typ);
        bitfield!{@impl $stname, $typ { $($body)* }}
    };

    {pub $stname:ident : $typ:ty { $($body:tt)* }} => {
        #[derive(Clone, Copy, PartialEq, Debug)]
        pub struct $stname($typ);
        bitfield!{@impl $stname, $typ { $($body)* }}
    };

}

#[cfg(test)]
mod tests {
    bitfield!{
        OpenMode: u32 {
            READ[0];
            WRITE[1];
            CREATE[2];
            APPEND[3];
            NUM1[4];
            NUM3[7:5];
        }
    }

    #[test]
    fn test_bool_1() {
        let mode: OpenMode = OpenMode::READ;
        assert_eq!(mode.is_set(OpenMode::READ), true);
        assert_eq!(mode.is_set(OpenMode::WRITE), false);
        assert_eq!(mode.is_set(OpenMode::CREATE), false);
        assert_eq!(mode.is_set(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_2() {
        let mode: OpenMode = OpenMode::WRITE;
        assert_eq!(mode.is_set(OpenMode::READ), false);
        assert_eq!(mode.is_set(OpenMode::WRITE), true);
        assert_eq!(mode.is_set(OpenMode::CREATE), false);
        assert_eq!(mode.is_set(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_3() {
        let mode: OpenMode = OpenMode::READ | OpenMode::WRITE | OpenMode::CREATE;
        assert_eq!(mode.is_set(OpenMode::READ), true);
        assert_eq!(mode.is_set(OpenMode::WRITE), true);
        assert_eq!(mode.is_set(OpenMode::CREATE), true);
        assert_eq!(mode.is_set(OpenMode::APPEND), false);

        assert_eq!(mode.is_set(OpenMode::READ |
                               OpenMode::WRITE |
                               OpenMode::CREATE |
                               OpenMode::APPEND), false);

        assert_eq!(mode.is_set(OpenMode::READ |
                               OpenMode::WRITE |
                               OpenMode::CREATE), true);

        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_num() {
        let mode: OpenMode = OpenMode::CREATE | OpenMode::NUM3.compose(7);
        assert_eq!(mode.is_set(OpenMode::READ), false);
        assert_eq!(mode.is_set(OpenMode::WRITE), false);
        assert_eq!(mode.is_set(OpenMode::CREATE), true);
        assert_eq!(mode.is_set(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 7);
    }

}
