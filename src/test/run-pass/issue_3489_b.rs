// xfail-fast
// aux-build:issue_3489_a.rs

extern mod issue_3489_a;
use issue_3489_a::Foo;

fn f(x: Foo) -> Foo { x }

fn main() {
    assert f(5) == 5;
}
