// Works if I change cat to a class
type cat = { meows: uint };

trait fuzzy {
  fn pet();
}

impl of fuzzy for cat {
  fn brush() {
    if self.meows == 0 {
        self.pet();
    }
  }
  fn pet() {
    self.brush();
  }
}

fn main() {}
