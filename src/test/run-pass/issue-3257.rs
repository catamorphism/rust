use option::*;

fn main() {
    match Some(true) {
      Some(false) => {
      }
      whatever if false => {
      }
      Some(true) => {
      }
      None => ()
    }
}
