fn fail_then_concat() {
    let x = ~[], y = ~[3];
    fail;
    x += y; // This should be rejected b/c x is immutable.
    // Shouldn't matter that this is unreachable.
 //   ~"good" + ~"bye";
}

fn main() {}