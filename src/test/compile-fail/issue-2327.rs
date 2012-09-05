struct A {
    x: int;
}

trait X {
    fn x();
}

impl A: X {
    fn x() { }
}

fn main() {
    let a: A = A { x:5 };
    a.x(); //~ ERROR quux
}