use std::io;

macro_rules! print_hd_tl (
    ($field_hd:ident, $($field_tl:ident),+) => ({
//        io::print(stringify!($field)); compile-fail
        io::print(stringify!($field_hd));
        io::print("::[");
        $(
            io::print(stringify!($field_tl));
            io::print(", ");
        )+
        io::print("]\n");
    })
)

fn main() {
    print_hd_tl!(x, y, z, w)
}

