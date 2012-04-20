// xfail-test
// test for Issue #1516
fn main() {
   let early_error: fn@(str) -> !  = {|msg| fail; };
}