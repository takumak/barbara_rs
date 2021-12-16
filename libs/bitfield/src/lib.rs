#![feature(const_generics_defaults)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]

struct NumberField<T, I, const LEFT: u32, const RIGHT: u32> {
    _t: core::marker::PhantomData<T>,
    _i: core::marker::PhantomData<I>,
}

struct BoolField<T, I, const BIT: u32> {
    _t: core::marker::PhantomData<T>,
    _i: core::marker::PhantomData<I>,
}

macro_rules! define_typed_number_field {
    ($t:ident, $i:ty) => {
        impl<const LEFT: u32, const RIGHT: u32> $crate::NumberField<$t, $i, LEFT, RIGHT> {
            const MASK: $i = ((2 << LEFT) - 1) ^ ((1 << RIGHT) - 1);
        }

        /**** Fn generator ****/

        impl<const L: u32, const R: u32> FnOnce<($i,)> for $crate::NumberField<$t, $i, L, R> {
            type Output = $t;

            extern "rust-call" fn call_once(self, (val,): ($i,)) -> $t {
                $t((val << R) & Self::MASK)
            }
        }

        impl<const L: u32, const R: u32> FnMut<($i,)> for $crate::NumberField<$t, $i, L, R> {
            extern "rust-call" fn call_mut(&mut self, (val,): ($i,)) -> $t {
                $t((val << R) & Self::MASK)
            }
        }

        impl<const L: u32, const R: u32> Fn<($i,)> for $crate::NumberField<$t, $i, L, R> {
            extern "rust-call" fn call(&self, (val,): ($i,)) -> $t {
                $t((val << R) & Self::MASK)
            }
        }
    }
}

macro_rules! define_typed_bool_field {
    ($t:ident, $i:ty) => {
        impl<const BIT: u32> $crate::BoolField<$t, $i, BIT> {
            const BIT: $i = 1 << BIT;
        }

        /**** BitOr composition ****/

        impl<const B: u32, const OB: u32>
            core::ops::BitOr<$crate::BoolField<$t, $i, OB>> for $crate::BoolField<$t, $i, B>
        {
            type Output = $t;

            fn bitor(self, _: $crate::BoolField<$t, $i, OB>) -> Self::Output {
                $t(Self::BIT | $crate::BoolField::<$t, $i, OB>::BIT)
            }
        }

        impl<const OB: u32> core::ops::BitOr<$crate::BoolField<$t, $i, OB>> for $t
        {
            type Output = $t;

            fn bitor(self, _: $crate::BoolField<$t, $i, OB>) -> Self::Output {
                $t(self.0 | $crate::BoolField::<$t, $i, OB>::BIT)
            }
        }

        impl<const B: u32> core::ops::BitOr<$t> for $crate::BoolField<$t, $i, B>
        {
            type Output = $t;

            fn bitor(self, rhs: $t) -> Self::Output {
                $t(Self::BIT | rhs.0)
            }
        }

        /**** Fn generator ****/

        impl<const B: u32> FnOnce<(bool,)> for $crate::BoolField<$t, $i, B> {
            type Output = $t;

            extern "rust-call" fn call_once(self, (_,): (bool,)) -> $t {
                $t(1 << B)
            }
        }

        impl<const B: u32> FnMut<(bool,)> for $crate::BoolField<$t, $i, B> {
            extern "rust-call" fn call_mut(&mut self, (_,): (bool,)) -> $t {
                $t(1 << B)
            }
        }

        impl<const B: u32> Fn<(bool,)> for $crate::BoolField<$t, $i, B> {
            extern "rust-call" fn call(&self, (_,): (bool,)) -> $t {
                $t(1 << B)
            }
        }
    }
}

macro_rules! bitfield {
    {@field $stname:ident $typ:ty { }} => { };

    {@field $stname:ident $typ:ty { $name:ident : u[$left:literal : $right:literal]; $($remain:tt)* }} => {
        const $name: $crate::NumberField<$stname, $typ, $left, $right> =
            $crate::NumberField::<$stname, $typ, $left, $right> {
                _t: core::marker::PhantomData,
                _i: core::marker::PhantomData,
            };
        bitfield! {@field $stname $typ { $($remain)* }}
    };

    {@field $stname:ident $typ:ty { $name:ident : u[$left:literal]; $($remain:tt)* }} => {
        const $name: $crate::NumberField<$stname, $typ, $left, $left> =
            $crate::NumberField::<$stname, $typ, $left, $left> {
                _t: core::marker::PhantomData,
                _i: core::marker::PhantomData,
            };
        bitfield! {@field $stname $typ { $($remain)* }}
    };

    {@field $stname:ident $typ:ty { $name:ident : bool[$bit:literal]; $($remain:tt)* }} => {
        const $name: $crate::BoolField<$stname, $typ, $bit> = $crate::BoolField::<$stname, $typ, $bit> {
            _t: core::marker::PhantomData,
            _i: core::marker::PhantomData,
        };
        bitfield! {@field $stname $typ { $($remain)* }}
    };

    {$stname:ident : $typ:ty { $($body:tt)* }} => {
        struct $stname ($typ);

        impl $stname {
            bitfield! {@field $stname $typ { $($body)* }}
        }

        define_typed_number_field!($stname, $typ);
        define_typed_bool_field!($stname, $typ);

        /**** NumberField selector ****/

        impl<const L: u32, const R: u32>
            FnOnce<($crate::NumberField<$stname, $typ, L, R>,)> for $stname
        {
            type Output = $typ;

            extern "rust-call" fn call_once(self, _: ($crate::NumberField<$stname, $typ, L, R>,)) -> $typ {
                (self.0 & $crate::NumberField::<$stname, $typ, L, R>::MASK) >> R
            }
        }

        impl<const L: u32, const R: u32> FnMut<($crate::NumberField<$stname, $typ, L, R>,)> for $stname {
            extern "rust-call" fn call_mut(&mut self, _: ($crate::NumberField<$stname, $typ, L, R>,)) -> $typ {
                (self.0 & $crate::NumberField::<$stname, $typ, L, R>::MASK) >> R
            }
        }

        impl<const L: u32, const R: u32> Fn<($crate::NumberField<$stname, $typ, L, R>,)> for $stname {
            extern "rust-call" fn call(&self, _: ($crate::NumberField<$stname, $typ, L, R>,)) -> $typ {
                (self.0 & $crate::NumberField::<$stname, $typ, L, R>::MASK) >> R
            }
        }

        /**** BoolField selector ****/

        impl<const B: u32> FnOnce<($crate::BoolField<$stname, $typ, B>,)> for $stname {
            type Output = bool;

            extern "rust-call" fn call_once(self, _: ($crate::BoolField<$stname, $typ, B>,)) -> bool {
                (self.0 & $crate::BoolField::<$stname, $typ, B>::BIT) != 0
            }
        }

        impl<const B: u32> FnMut<($crate::BoolField<$stname, $typ, B>,)> for $stname {
            extern "rust-call" fn call_mut(&mut self, _: ($crate::BoolField<$stname, $typ, B>,)) -> bool {
                (self.0 & $crate::BoolField::<$stname, $typ, B>::BIT) != 0
            }
        }

        impl<const B: u32> Fn<($crate::BoolField<$stname, $typ, B>,)> for $stname {
            extern "rust-call" fn call(&self, _: ($crate::BoolField<$stname, $typ, B>,)) -> bool {
                (self.0 & $crate::BoolField::<$stname, $typ, B>::BIT) != 0
            }
        }
    };
}

#[cfg(test)]
mod tests {
    bitfield!{
        OpenMode: u32 {
            READ: bool[0];
            WRITE: bool[1];
            CREATE: bool[2];
            APPEND: bool[3];
            NUM1: u[4];
            NUM3: u[7:5];
        }
    }

    #[test]
    fn test_bool_1() {
        let mode: OpenMode = OpenMode::READ(true);
        assert_eq!(mode(OpenMode::READ), true);
        assert_eq!(mode(OpenMode::WRITE), false);
        assert_eq!(mode(OpenMode::CREATE), false);
        assert_eq!(mode(OpenMode::APPEND), false);
        assert_eq!(mode(OpenMode::NUM1), 0);
        assert_eq!(mode(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_2() {
        let mode: OpenMode = OpenMode::WRITE(true);
        assert_eq!(mode(OpenMode::READ), false);
        assert_eq!(mode(OpenMode::WRITE), true);
        assert_eq!(mode(OpenMode::CREATE), false);
        assert_eq!(mode(OpenMode::APPEND), false);
        assert_eq!(mode(OpenMode::NUM1), 0);
        assert_eq!(mode(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_bool_3() {
        let mode: OpenMode = OpenMode::READ | OpenMode::WRITE | OpenMode::CREATE | OpenMode::APPEND;
        assert_eq!(mode(OpenMode::READ), true);
        assert_eq!(mode(OpenMode::WRITE), true);
        assert_eq!(mode(OpenMode::CREATE), true);
        assert_eq!(mode(OpenMode::APPEND), true);
        assert_eq!(mode(OpenMode::NUM1), 0);
        assert_eq!(mode(OpenMode::NUM3), 0);
    }

    #[test]
    fn test_num() {
        let mode: OpenMode = OpenMode::CREATE | OpenMode::NUM3(7);
        assert_eq!(mode(OpenMode::READ), false);
        assert_eq!(mode(OpenMode::WRITE), false);
        assert_eq!(mode(OpenMode::CREATE), true);
        assert_eq!(mode(OpenMode::APPEND), false);
        assert_eq!(mode(OpenMode::NUM1), 0);
        assert_eq!(mode(OpenMode::NUM3), 7);
    }
}
