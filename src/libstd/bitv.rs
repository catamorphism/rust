import vec::{to_mut, from_elem};

export bitv;
export mk_bitv;

// (part 1)
// FIXME (#2341): With recursive object types, we could implement binary
// methods like union, intersection, and difference.

// part 2
// At that point, we could
// write an optimizing version of this module that produces a different obj
// for the case where nbits <= 32.

/// The bitvector type
type bitv = ~bitv_;
class bitv_ {
    // would rather make this immutable,
    // and have a constructor that takes a storage
    let mut storage: ~[mut uint];
    let nbits: uint;

    priv {
    pure fn process(v1: bitv, op: fn(uint, uint) -> uint) -> bool {
      let len = v1.storage.len();
      assert (self.storage.len() == len);
      assert (self.nbits == v1.nbits);
      let mut changed = false;
      for uint::range(0, len) |i| {
        let w0 = self.storage[i];
        let w1 = v1.storage[i];
        let w = op(w0, w1);
        // safe b/c bitvs are unique
        if w0 != w unchecked { changed = true; self.storage[i] = w; }
      };
      changed
    }
    }

    // Has to be public because private methods are still
    // instance-private -- but no one should call this from
    // outside the class
    fn set_storage(-s: ~[mut uint]) {
        self.storage <- s;
    }

    fn union(v1: bitv) -> bool { self.process(v1, lor) }

/**
 * Calculates the intersection of two bitvectors
 *
 * Sets `self` to the intersection of `self` and `v1`. Both bitvectors must be
 * the same length. Returns 'true' if `self` changed.
 */
    fn intersect(v1: bitv) -> bool { self.process(v1, land) }

/**
 * Assigns the value of `v1` to `self`
 *
 * Both bitvectors must be the same length. Returns `true` if `self` was
 * changed
 */
    pure fn assign(v: bitv) -> bool { self.process(v, right) }

    /// Makes a copy of a bitvector
    pure fn clone() -> bitv {
        let storage = to_mut(from_elem(self.nbits / uint_bits + 1, 0));
        let len = self.storage.len();
        for uint::range(0, len) |i| { storage[i] = self.storage[i]; };
        // bitv_ doesn't do anything sketch
        let rs = unchecked { ~bitv_(self.nbits, false) };
        // workaround for lack of multiple constructors
        rs.storage <- storage;
        rs
    }

    /// Retrieve the value at index `i`
    #[inline(always)]
        pure fn get(i: uint) -> bool {
        assert (i < self.nbits);
        let bits = uint_bits;
        let w = i / bits;
        let b = i % bits;
        let x = 1 & self.storage[w] >> b;
        x == 1
    }

/**
 * Compares two bitvectors
 *
 * Both bitvectors must be the same length. Returns `true` if both bitvectors
 * contain identical elements.
 */
    pure fn equal(v1: bitv) -> bool {
        if self.nbits != v1.nbits { ret false; }
        let len = v1.storage.len();
        for uint::iterate(0, len) |i| {
          if self.storage[i] != v1.storage[i] { ret false; }
        }
    }

    /// Set all bits to 0
    #[inline(always)]
    fn clear() { for self.each_storage() |w| { w = 0u } }

    /// Set all bits to 1
    #[inline(always)]
    fn set_all() { for self.each_storage() |w| { w = !0u } }

    /// Invert all bits
    #[inline(always)]
    fn invert() { for self.each_storage() |w| { w = !w } }

/**
 * Calculate the difference between two bitvectors
 *
 * Sets each element of `v0` to the value of that element minus the element
 * of `v1` at the same index. Both bitvectors must be the same length.
 *
 * Returns `true` if `v0` was changed.
 */
    fn difference(v: bitv) -> bool {
        v.invert();
        let b = self.intersect(v);
        v.invert();
        b
    }

/**
 * Set the value of a bit at a given index
 *
 * `i` must be less than the length of the bitvector.
 */
#[inline(always)]
    pure fn set(i: uint, x: bool) {
      assert (i < self.nbits);
      let bits = uint_bits;
      let w = i / bits;
      let b = i % bits;
      let flag = 1 << b;
      // Safe b/c bitvs are unique
      unchecked {
      self.storage[w] = if x { self.storage[w] | flag }
          else { self.storage[w] & !flag };
      }
   }

/// Returns true if all bits are 1
 fn is_true() -> bool {
     for self.each() |i| { if !i { ret false; } }
     true
 }

 /// Returns true if all bits are 0
 fn is_false() -> bool {
     for self.each() |i| { if i { ret false; } }
     true
 }

 fn init_to_vec(i: uint) -> uint {
     ret if self.get(i) { 1 } else { 0 };
 }


/**
 * Converts `self` to a vector of uint with the same length.
 *
 * Each uint in the resulting vector has either value 0u or 1u.
 */
 fn to_vec() -> ~[uint] {
     let sub = |x| self.init_to_vec(x);
     ret vec::from_fn::<uint>(self.nbits, sub);
 }

 #[inline(always)]
     fn each(f: fn(bool) -> bool) {
       let mut i = 0u;
       while i < self.nbits {
          if !f(self.get(i)) { break; }
          i = i + 1u;
       }
     }

 #[inline(always)]
     fn each_storage(op: fn(&uint) -> bool) {
     for uint::range(0, self.storage.len()) |i| {
             let mut w = self.storage[i];
             let b = !op(w);
             self.storage[i] = w;
             if !b { break; }
    }
 }

/**
 * Converts `self` to a string.
 *
 * The resulting string has the same length as `self`, and each
 * character is either '0' or '1'.
 */
fn to_str() -> str {
    let mut rs = "";
    for self.each() |i| { if i { rs += "1"; } else { rs += "0"; } }
    rs
}

/**
 * Compare a bitvector to a vector of uint
 *
 * The uint vector is expected to only contain the values 0u and 1u. Both the
 * bitvector and vector must have the same length
 */
fn eq_vec(v: ~[uint]) -> bool {
    assert self.nbits == v.len();
    let mut i = 0;
    while i < self.nbits {
        let w0 = self.get(i);
        let w1 = v[i];
        if !w0 && w1 != 0u || w0 && w1 == 0u { ret false; }
        i = i + 1;
    }
    true
}
    new(nbits: uint, init: bool) {
        let elt = if init { !0 } else { 0 };
        let storage = to_mut(from_elem(nbits / uint_bits + 1u, elt));
        self.storage = storage;
        self.nbits = nbits;
    }
}

/**
 * Constructs a bitvector
 *
 * # Arguments
 *
 * * nbits - The number of bits in the bitvector
 * * init - If true then the bits are initialized to 1, otherwise 0
 */
fn mk_bitv(nbits: uint, init: bool) -> bitv { ~bitv_(nbits, init) }

const uint_bits: uint = 32u + (1u << 32u >> 27u);

pure fn lor(w0: uint, w1: uint) -> uint { ret w0 | w1; }

pure fn land(w0: uint, w1: uint) -> uint { ret w0 & w1; }

pure fn right(_w0: uint, w1: uint) -> uint { ret w1; }

#[cfg(test)]
mod tests {
    #[test]
    fn test_to_str() {
        let zerolen = bitv(0u, false);
        assert to_str(zerolen) == "";

        let eightbits = bitv(8u, false);
        assert to_str(eightbits) == "00000000";
    }

    #[test]
    fn test_0_elements() {
        let mut act;
        let mut exp;
        act = bitv(0u, false);
        exp = vec::from_elem::<uint>(0u, 0u);
        assert (eq_vec(act, exp));
    }

    #[test]
    fn test_1_element() {
        let mut act;
        act = bitv(1u, false);
        assert (eq_vec(act, ~[0u]));
        act = bitv(1u, true);
        assert (eq_vec(act, ~[1u]));
    }

    #[test]
    fn test_10_elements() {
        let mut act;
        // all 0

        act = bitv(10u, false);
        assert (eq_vec(act, ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u]));
        // all 1

        act = bitv(10u, true);
        assert (eq_vec(act, ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(10u, false);
        set(act, 0u, true);
        set(act, 1u, true);
        set(act, 2u, true);
        set(act, 3u, true);
        set(act, 4u, true);
        assert (eq_vec(act, ~[1u, 1u, 1u, 1u, 1u, 0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(10u, false);
        set(act, 5u, true);
        set(act, 6u, true);
        set(act, 7u, true);
        set(act, 8u, true);
        set(act, 9u, true);
        assert (eq_vec(act, ~[0u, 0u, 0u, 0u, 0u, 1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(10u, false);
        set(act, 0u, true);
        set(act, 3u, true);
        set(act, 6u, true);
        set(act, 9u, true);
        assert (eq_vec(act, ~[1u, 0u, 0u, 1u, 0u, 0u, 1u, 0u, 0u, 1u]));
    }

    #[test]
    fn test_31_elements() {
        let mut act;
        // all 0

        act = bitv(31u, false);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u]));
        // all 1

        act = bitv(31u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(31u, false);
        set(act, 0u, true);
        set(act, 1u, true);
        set(act, 2u, true);
        set(act, 3u, true);
        set(act, 4u, true);
        set(act, 5u, true);
        set(act, 6u, true);
        set(act, 7u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(31u, false);
        set(act, 16u, true);
        set(act, 17u, true);
        set(act, 18u, true);
        set(act, 19u, true);
        set(act, 20u, true);
        set(act, 21u, true);
        set(act, 22u, true);
        set(act, 23u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(31u, false);
        set(act, 24u, true);
        set(act, 25u, true);
        set(act, 26u, true);
        set(act, 27u, true);
        set(act, 28u, true);
        set(act, 29u, true);
        set(act, 30u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(31u, false);
        set(act, 3u, true);
        set(act, 17u, true);
        set(act, 30u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u]));
    }

    #[test]
    fn test_32_elements() {
        let mut act;
        // all 0

        act = bitv(32u, false);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u]));
        // all 1

        act = bitv(32u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(32u, false);
        set(act, 0u, true);
        set(act, 1u, true);
        set(act, 2u, true);
        set(act, 3u, true);
        set(act, 4u, true);
        set(act, 5u, true);
        set(act, 6u, true);
        set(act, 7u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(32u, false);
        set(act, 16u, true);
        set(act, 17u, true);
        set(act, 18u, true);
        set(act, 19u, true);
        set(act, 20u, true);
        set(act, 21u, true);
        set(act, 22u, true);
        set(act, 23u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(32u, false);
        set(act, 24u, true);
        set(act, 25u, true);
        set(act, 26u, true);
        set(act, 27u, true);
        set(act, 28u, true);
        set(act, 29u, true);
        set(act, 30u, true);
        set(act, 31u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(32u, false);
        set(act, 3u, true);
        set(act, 17u, true);
        set(act, 30u, true);
        set(act, 31u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u, 1u]));
    }

    #[test]
    fn test_33_elements() {
        let mut act;
        // all 0

        act = bitv(33u, false);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u]));
        // all 1

        act = bitv(33u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u, 1u]));
        // mixed

        act = bitv(33u, false);
        set(act, 0u, true);
        set(act, 1u, true);
        set(act, 2u, true);
        set(act, 3u, true);
        set(act, 4u, true);
        set(act, 5u, true);
        set(act, 6u, true);
        set(act, 7u, true);
        assert (eq_vec(act,
                       ~[1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(33u, false);
        set(act, 16u, true);
        set(act, 17u, true);
        set(act, 18u, true);
        set(act, 19u, true);
        set(act, 20u, true);
        set(act, 21u, true);
        set(act, 22u, true);
        set(act, 23u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 1u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u]));
        // mixed

        act = bitv(33u, false);
        set(act, 24u, true);
        set(act, 25u, true);
        set(act, 26u, true);
        set(act, 27u, true);
        set(act, 28u, true);
        set(act, 29u, true);
        set(act, 30u, true);
        set(act, 31u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 1u, 1u,
                        1u, 1u, 1u, 1u, 1u, 1u, 0u]));
        // mixed

        act = bitv(33u, false);
        set(act, 3u, true);
        set(act, 17u, true);
        set(act, 30u, true);
        set(act, 31u, true);
        set(act, 32u, true);
        assert (eq_vec(act,
                       ~[0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u, 0u, 0u, 0u, 0u, 0u, 0u, 0u, 0u,
                        0u, 0u, 0u, 0u, 1u, 1u, 1u]));
    }

    #[test]
    fn test_equal_differing_sizes() {
        let v0 = bitv(10u, false);
        let v1 = bitv(11u, false);
        assert !equal(v0, v1);
    }

    #[test]
    fn test_equal_greatly_differing_sizes() {
        let v0 = bitv(10u, false);
        let v1 = bitv(110u, false);
        assert !equal(v0, v1);
    }

}

//
// Local Variables:
// mode: rust
// fill-column: 78;
// indent-tabs-mode: nil
// c-basic-offset: 4
// buffer-file-coding-system: utf-8-unix
// End:
//
