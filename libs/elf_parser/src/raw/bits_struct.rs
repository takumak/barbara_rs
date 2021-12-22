#[macro_export]
macro_rules! bits_struct {

    /**** struct body ****/

    {@struct $svis:vis struct $sname:ident : $tname:ident
     { $($fields:tt)* }
     { $($methods:tt)* }
     { }} => {
        unpacker! {
            $svis struct $sname {
                $($fields)*
            }
        }

        impl $tname for $sname {
            $($methods)*
        }
    };

    {@struct $svis:vis struct $sname:ident : $tname:ident
     { $($fields:tt)* }
     { $($methods:tt)* }
     { $fvis:vis $fname:ident : {$ftyp:ty} $fgetter:ident($fgret:ty); $($remains:tt)* }} => {
        bits_struct!{
            @struct $svis struct $sname : $tname
            { $($fields)* $fvis $fname : $ftyp,}
            { $($methods)*
              fn $fgetter(&self) -> $fgret {
                  <$fgret>::from(self.$fname)
              }
            }
            { $($remains)* }
        }
    };

    /**** get fields of first type => struct body ****/

    {@structs $tname:ident { } { } { } { $($fields:tt)* }} => { };

    {@structs $tname:ident
     { $svis:vis struct $sname:ident; $($snames:tt)* }
     { $($fields_1:tt)* }
     { $($fields_n:tt)* }
     { }} => {
         bits_struct!{@struct $svis struct $sname : $tname { } { } { $($fields_1)* }}
         bits_struct!{@structs $tname { $($snames)* } { } { } { $($fields_n)* }}
    };

    {@structs $tname:ident
     { $($sname:tt)+ }
     { $($fields_1:tt)* }
     { $($fields_n:tt)* }
     { $fvis:vis $fname:ident : {$ftyp:ty, $($typs:ty,)*} $fgetter:ident($fgret:ty); $($remains:tt)* }} => {
        bits_struct!{
            @structs $tname
            { $($sname)+ }
            { $($fields_1)* $fvis $fname : {$ftyp} $fgetter($fgret); }
            { $($fields_n)* $fvis $fname : {$($typs,)*} $fgetter($fgret); }
            { $($remains)* }
        }
    };

    /**** trait ****/

    {@trait $tvis:vis trait $tname:ident
     { $($results:tt)* }
     { }} => {
        $tvis trait $tname {
            $($results)*
        }
    };

    {@trait $tvis:vis trait $tname:ident
     { $($results:tt)* }
     { $fvis:vis $fname:ident : {$($ftyp:tt)+} $fgetter:ident($fgret:ty); $($remains:tt)* }} => {
        bits_struct!{
            @trait $tvis trait $tname
            { $($results)* fn $fgetter(&self) -> $fgret; }
            { $($remains)* }
        }
    };

    /**** entrypoint ****/

    {$tvis:vis trait $tname:ident;
     { $($structs:tt)+ }
     { $($fields:tt)+ }} => {
        bits_struct!{@trait $tvis trait $tname { } { $($fields)+ }}
        bits_struct!{@structs $tname { $($structs)+ } { } { } { $($fields)+ }}
    };
}

/*

Example:
    bits_struct! {
        pub trait Header;
        {
            pub struct Header32;
            pub struct Header64;
        };
        {
            pub addr: {u32, u64,} get_addr(u64);
        }
    }

Expands to:
    pub trait Header {
        pub fn get_addr(&self) -> u64;
    }
    pub struct Header32 {
        pub addr: u32;
    }
    impl Header for Header32 {
        pub fn get_addr(&self) -> u64 {
            u64::from(self.addr)
        }
    }
    pub struct Header64 {
        pub addr: u64;
    }
    impl Header for Header64 {
        pub fn get_addr(&self) -> u64 {
            u64::from(self.addr)
        }
    }

 */
