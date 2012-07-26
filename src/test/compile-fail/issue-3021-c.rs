use std;

fn siphash<T>(k0 : u64) {

    trait t {
        fn g(x: T) -> T;  //~ ERROR attempted dynamic environment-capture
        //~~ ERROR unresolved name
    }
}

fn main() {}
