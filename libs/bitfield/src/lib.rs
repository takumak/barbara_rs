#![cfg_attr(not(test), no_std)]
#![feature(const_generics_defaults)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

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

            fn all(&self, m: $stname) -> bool {
                (self.0 & m.0) == m.0
            }

            fn any(&self, m: $stname) -> bool {
                (self.0 & m.0) != 0
            }

            fn extract(&self, m: $stname) -> $typ {
                (self.0 & m.0) >> m.0.trailing_zeros()
            }

            fn compose(&self, v: $typ) -> $stname {
                $stname(v << self.0.trailing_zeros())
            }
        }

        impl core::ops::BitOr for $stname {
            type Output = $stname;
            fn bitor(self, rhs: $stname) -> $stname {
                $stname(self.0 | rhs.0)
            }
        }

        /* extract shortcut */

        impl core::ops::FnOnce<($stname,)> for $stname {
            type Output = $typ;
            extern "rust-call"
            fn call_once(self, (m,): ($stname,)) -> $typ {
                self.extract(m)
            }
        }

        impl core::ops::FnMut<($stname,)> for $stname {
            extern "rust-call"
            fn call_mut(&mut self, (m,): ($stname,)) -> $typ {
                self.extract(m)
            }
        }

        impl core::ops::Fn<($stname,)> for $stname {
            extern "rust-call"
            fn call(&self, (m,): ($stname,)) -> $typ {
                self.extract(m)
            }
        }

        /* compose shortcut */

        impl core::ops::FnOnce<($typ,)> for $stname {
            type Output = $stname;
            extern "rust-call"
            fn call_once(self, (v,): ($typ,)) -> $stname {
                self.compose(v)
            }
        }

        impl core::ops::FnMut<($typ,)> for $stname {
            extern "rust-call"
            fn call_mut(&mut self, (v,): ($typ,)) -> $stname {
                self.compose(v)
            }
        }

        impl core::ops::Fn<($typ,)> for $stname {
            extern "rust-call"
            fn call(&self, (v,): ($typ,)) -> $stname {
                self.compose(v)
            }
        }
    };

    {$stname:ident : $typ:ty { $($body:tt)* }} => {
        #[derive(Clone, Copy)]
        struct $stname($typ);
        bitfield!{@impl $stname, $typ { $($body)* }}
    };

    {pub $stname:ident : $typ:ty { $($body:tt)* }} => {
        #[derive(Clone, Copy)]
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
        assert_eq!(mode.all(OpenMode::READ), true);
        assert_eq!(mode.all(OpenMode::WRITE), false);
        assert_eq!(mode.all(OpenMode::CREATE), false);
        assert_eq!(mode.all(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_2() {
        let mode: OpenMode = OpenMode::WRITE;
        assert_eq!(mode.all(OpenMode::READ), false);
        assert_eq!(mode.all(OpenMode::WRITE), true);
        assert_eq!(mode.all(OpenMode::CREATE), false);
        assert_eq!(mode.all(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_3() {
        let mode: OpenMode = OpenMode::READ | OpenMode::WRITE | OpenMode::CREATE;
        assert_eq!(mode.all(OpenMode::READ), true);
        assert_eq!(mode.all(OpenMode::WRITE), true);
        assert_eq!(mode.all(OpenMode::CREATE), true);
        assert_eq!(mode.all(OpenMode::APPEND), false);

        assert_eq!(mode.all(OpenMode::READ |
                            OpenMode::WRITE |
                            OpenMode::CREATE |
                            OpenMode::APPEND), false);

        assert_eq!(mode.all(OpenMode::READ |
                            OpenMode::WRITE |
                            OpenMode::CREATE), true);

        assert_eq!(mode.any(OpenMode::READ |
                            OpenMode::WRITE |
                            OpenMode::CREATE |
                            OpenMode::APPEND), true);

        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_num() {
        let mode: OpenMode = OpenMode::CREATE | OpenMode::NUM3.compose(7);
        assert_eq!(mode.all(OpenMode::READ), false);
        assert_eq!(mode.all(OpenMode::WRITE), false);
        assert_eq!(mode.all(OpenMode::CREATE), true);
        assert_eq!(mode.all(OpenMode::APPEND), false);
        assert_eq!(mode.extract(OpenMode::NUM1), 0);
        assert_eq!(mode.extract(OpenMode::NUM3), 7);
    }

    #[test]
    fn test_call1() {
        let mode: OpenMode = OpenMode::APPEND | OpenMode::NUM3(6);
        assert_eq!(mode(OpenMode::READ), 0);
        assert_eq!(mode(OpenMode::WRITE), 0);
        assert_eq!(mode(OpenMode::CREATE), 0);
        assert_eq!(mode(OpenMode::APPEND), 1);
        assert_eq!(mode(OpenMode::NUM1), 0);
        assert_eq!(mode(OpenMode::NUM3), 6);
    }
}
