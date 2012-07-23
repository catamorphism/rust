// This works
type cat = { meows: uint };

trait fuzzy {
  fn pet();
}

trait sleepy {
  fn sleep();
}

impl of sleepy for cat {
  fn sleep() {}
}

impl of fuzzy for cat {
  fn pet() {
    self.sleep();
  }
}

fn main() {}
