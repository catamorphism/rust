use  myfloat = core::f32;
type myfloat = f32;   // without this
fn foo() -> myfloat { // this errors with: found typename used as variable
//          ^~~~~~~
    3_f32;
}
// but when adding it I get:
// error: unresolved name
// error: use of undeclared module 'myfloat'
// error: unresolved name myfloat::sqrt
// at the y = line

fn main() {
    let x: f32 = 3_f32;
    let y = myfloat::sqrt(x);
//          ^~~~~~~~~~~~~
}
