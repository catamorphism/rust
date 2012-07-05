#[doc = "
High-level text containers.

Ropes are a high-level representation of text that offers
much better performance than strings for common operations,
and generally reduce memory allocations and copies, while only
entailing a small degradation of less common operations.

More precisely, where a string is represented as a memory buffer,
a rope is a tree structure whose leaves are slices of immutable
strings. Therefore, concatenation, appending, prepending, substrings,
etc. are operations that require only trivial tree manipulation,
generally without having to copy memory. In addition, the tree
structure of ropes makes them suitable as a form of index to speed-up
access to Unicode characters by index in long chunks of text.

The following operations are algorithmically faster in ropes:

* extracting a subrope is logarithmic (linear in strings);
* appending/prepending is near-constant time (linear in strings);
* concatenation is near-constant time (linear in strings);
* char length is constant-time (linear in strings);
* access to a character by index is logarithmic (linear in strings);
"];

#[doc = "The type of ropes."]
type rope = node::root;

/*
 Section: Creating a rope
 */

#[doc = "Create an empty rope"]
fn empty() -> rope { node::empty() }

#[doc = "
Adopt a string as a rope.

# Arguments

* str - A valid string.

# Return value

A rope representing the same string as `str`. Depending of the length
of `str`, this rope may be empty, flat or complex.

# Performance notes

* this operation does not copy the string;
* the function runs in linear time.
"]
fn of_str(str: @str) -> rope {
    ret of_substr(str, 0u, str::len(*str));
}

#[doc = "
As `of_str` but for a substring.

# Arguments
* byte_offset - The offset of `str` at which the rope starts.
* byte_len - The number of bytes of `str` to use.

# Return value

A rope representing the same string as `str::substr(str, byte_offset,
byte_len)`.  Depending on `byte_len`, this rope may be empty, flat or
complex.

# Performance note

This operation does not copy the substring.

# Safety notes

* this function does _not_ check the validity of the substring;
* this function fails if `byte_offset` or `byte_len` do not match `str`.
"]
fn of_substr(str: @str, byte_offset: uint, byte_len: uint) -> rope {
    if byte_len == 0u { ret node::empty(); }
    if byte_offset + byte_len  > str::len(*str) { fail; }
    ret node::content(node::of_substr(str, byte_offset, byte_len));
}

/*
Section: Adding things to a rope
 */

#[doc = "
Add one char to the end of the rope

# Performance note

* this function executes in near-constant time
"]
fn append_char(rope: rope, char: char) -> rope {
    ret append_str(rope, @str::from_chars([char]));
}

#[doc = "
Add one string to the end of the rope

# Performance note

* this function executes in near-linear time
"]
fn append_str(rope: rope, str: @str) -> rope {
    ret append_rope(rope, of_str(str))
}

#[doc = "
Add one char to the beginning of the rope

# Performance note
* this function executes in near-constant time
"]
fn prepend_char(rope: rope, char: char) -> rope {
    ret prepend_str(rope, @str::from_chars([char]));
}

#[doc = "
Add one string to the beginning of the rope

# Performance note
* this function executes in near-linear time
"]
fn prepend_str(rope: rope, str: @str) -> rope {
    ret append_rope(of_str(str), rope)
}

#[doc = "Concatenate two ropes"]
fn append_rope(left: rope, right: rope) -> rope {
    option::map_default(left, right)
      {|left_content| option::map_default(right, left)
        {|right_content|
        node::content(node::concat2(left_content, right_content))}}
}

#[doc = "
Concatenate many ropes.

If the ropes are balanced initially and have the same height, the resulting
rope remains balanced. However, this function does not take any further
measure to ensure that the result is balanced.
"]
fn concat(v: [rope]) -> rope {
    //Copy `v` into a mut vector
    let mut len = vec::len(v);
    if len == 0u { ret node::empty(); }
    let ropes = vec::to_mut(vec::from_elem(len, v[0]));
    for uint::range(1u, len) {|i|
       ropes[i] = v[i];
    }

    //Merge progresively
    while len > 1u {
        for uint::range(0u, len/2u) {|i|
            ropes[i] = append_rope(ropes[2u*i], ropes[2u*i+1u]);
        }
        if len%2u != 0u {
            ropes[len/2u] = ropes[len - 1u];
            len = len/2u + 1u;
        } else {
            len = len/2u;
        }
    }

    //Return final rope
    ret ropes[0];
}


/*
Section: Keeping ropes healthy
 */


#[doc = "
Balance a rope.

# Return value

A copy of the rope in which small nodes have been grouped in memory,
and with a reduced height.

If you concatenate many ropes, it is generally a good idea
to rebalance your rope at some point, before using it for other purposes.
"]
fn bal(rope:rope) -> rope {
    option::map_default(rope, rope) {|x|
        option::map_default(node::bal(x), rope, node::content)}
}

/*
Section: Transforming ropes
 */


#[doc = "
Extract a subrope from a rope.

# Performance note

* on a balanced rope, this operation takes algorithmic time;
* this operation does not involve any copying

# Safety note

* this function fails if char_offset/char_len do not represent
  valid positions in rope
"]
fn sub_chars(rope: rope, char_offset: uint, char_len: uint) -> rope {
    if char_len == 0u { ret node::empty(); }
    alt(rope) {
      option::none { fail }
      option::some(node) {
        if char_len > node.char_len { fail }
        else {
            ret node::content(node::sub_chars(node, char_offset, char_len))
        }
      }
    }
}

#[doc = "
Extract a subrope from a rope.

# Performance note

* on a balanced rope, this operation takes algorithmic time;
* this operation does not involve any copying

# Safety note

* this function fails if byte_offset/byte_len do not represent
  valid positions in rope
"]
fn sub_bytes(rope: rope, byte_offset: uint, byte_len: uint) -> rope {
    if byte_len == 0u { ret node::empty(); }
    alt(rope) {
      option::none { fail }
      option::some(node) {
        if byte_len > node.byte_len { fail }
        else {
            ret node::content(node::sub_bytes(node, byte_offset, byte_len))
        }
      }
    }
}

/*
Section: Comparing ropes
 */

#[doc = "
Compare two ropes by Unicode lexicographical order.

This function compares only the contents of the rope, not their structure.

# Return value

A negative value if `left < right`, 0 if eq(left, right) or a positive
value if `left > right`
"]
fn cmp(left: rope, right: rope) -> int {
    alt((left, right)) {
      (option::none, option::none) { ret 0; }
      (option::none, _)     { ret -1;}
      (_, option::none)     { ret  1;}
      (option::some(a), option::some(b)) {
        ret node::cmp(a, b);
      }
    }
}

#[doc = "
Returns `true` if both ropes have the same content (regardless of
their structure), `false` otherwise
"]
fn eq(left: rope, right: rope) -> bool {
    ret cmp(left, right) == 0;
}

#[doc = "
# Arguments

* left - an arbitrary rope
* right - an arbitrary rope

# Return value

`true` if `left <= right` in lexicographical order (regardless of their
structure), `false` otherwise
"]
fn le(left: rope, right: rope) -> bool {
    ret cmp(left, right) <= 0;
}

#[doc = "
# Arguments

* left - an arbitrary rope
* right - an arbitrary rope

# Return value

`true` if `left < right` in lexicographical order (regardless of their
structure), `false` otherwise
"]
fn lt(left: rope, right: rope) -> bool {
    ret cmp(left, right) < 0;
}

#[doc = "
# Arguments

* left - an arbitrary rope
* right - an arbitrary rope

# Return value

 `true` if `left >= right` in lexicographical order (regardless of their
structure), `false` otherwise
"]
fn ge(left: rope, right: rope) -> bool {
    ret cmp(left, right) >= 0;
}

#[doc = "
# Arguments

* left - an arbitrary rope
* right - an arbitrary rope

# Return value

`true` if `left > right` in lexicographical order (regardless of their
structure), `false` otherwise
"]
fn gt(left: rope, right: rope) -> bool {
    ret cmp(left, right) > 0;
}

/*
Section: Iterating
 */

#[doc = "
Loop through a rope, char by char

While other mechanisms are available, this is generally the best manner
of looping through the contents of a rope char by char. If you prefer a
loop that iterates through the contents string by string (e.g. to print
the contents of the rope or output it to the system), however,
you should rather use `traverse_components`.

# Arguments

* rope - A rope to traverse. It may be empty.
* it - A block to execute with each consecutive character of the rope.
       Return `true` to continue, `false` to stop.

# Return value

`true` If execution proceeded correctly, `false` if it was interrupted,
that is if `it` returned `false` at any point.
"]
fn loop_chars(rope: rope, it: fn(char) -> bool) -> bool {
   alt(rope) {
      option::none { ret true }
      option::some(x) { ret node::loop_chars(x, it) }
   }
}

#[doc = "
Loop through a rope, char by char, until the end.

# Arguments
* rope - A rope to traverse. It may be empty
* it - A block to execute with each consecutive character of the rope.
"]
fn iter_chars(rope: rope, it: fn(char)) {
    loop_chars(rope) {|x|
        it(x);
        true
    };
}

#[doc ="
Loop through a rope, string by string

While other mechanisms are available, this is generally the best manner of
looping through the contents of a rope string by string, which may be useful
e.g. to print strings as you see them (without having to copy their
contents into a new string), to send them to then network, to write them to
a file, etc.. If you prefer a loop that iterates through the contents
char by char (e.g. to search for a char), however, you should rather
use `traverse`.

# Arguments

* rope - A rope to traverse. It may be empty
* it - A block to execute with each consecutive string component of the rope.
       Return `true` to continue, `false` to stop

# Return value

`true` If execution proceeded correctly, `false` if it was interrupted,
that is if `it` returned `false` at any point.
"]
fn loop_leaves(rope: rope, it: fn(uint, node::leaf) -> bool) -> bool{
   alt(rope) {
      option::none { ret true }
      option::some(x) {ret node::loop_leaves(x, it)}
   }
}

mod iterator {
    mod leaf {
        fn start(rope: rope) -> node::leaf_iterator::t {
            alt(rope) {
              option::none    { ret node::leaf_iterator::empty() }
              option::some(x) { ret node::leaf_iterator::start(x) }
            }
        }
        fn next(it: node::leaf_iterator::t) -> option<(uint, node::leaf)> {
            ret node::leaf_iterator::next(it);
        }
    }
    mod char {
        fn start(rope: rope) -> node::char_iterator::t {
            alt(rope) {
              option::none    { ret node::char_iterator::empty() }
              option::some(x) { ret node::char_iterator::start(x) }
            }
        }
        fn next(it: node::char_iterator::t) -> option<char> {
            ret node::char_iterator::next(it)
        }
    }
}

/*
 Section: Rope properties
 */

#[doc ="
Returns the height of the rope.

The height of the rope is a bound on the number of operations which
must be performed during a character access before finding the leaf in
which a character is contained.

# Performance note

Constant time.
"]
fn height(rope: rope) -> uint { option::map_default(rope,
                                                    0u, {|x| x.height}) }


#[doc ="
The number of character in the rope

# Performance note

Constant time.
"]
pure fn char_len(&&rope: rope) -> uint {
    option::map_default(rope, 0u) {|x| x.char_len}
}

#[doc = "
The number of bytes in the rope

# Performance note

Constant time.
"]
pure fn byte_len(&&rope: rope) -> uint {
    option::map_default(rope, 0u) {|x| x.byte_len}
}

#[doc = "
The character at position `pos`

# Arguments

* pos - A position in the rope

# Safety notes

The function will fail if `pos` is not a valid position in the rope.

# Performance note

This function executes in a time proportional to the height of the
rope + the (bounded) length of the largest leaf.
"]
fn char_at(rope: rope, pos: uint) -> char {
   alt(rope) {
      option::none { fail }
      option::some(x) { node::char_at(x, pos) }
   }
}


/*
 Section: Implementation
*/
mod node {

    #[doc = "Implementation of type `rope`. `none` represents an empty rope,
            `some` represents a non-empty rope"]
    type root = option<@node>;

#[doc = "
    * byte_len - The number of bytes of `content` to use
    * char_len - The number of chars in the leaf.
"]
    class node {
        let mut char_len: uint;
        let mut byte_len: uint;
        let mut height:   uint;
        let node: node_;
        new(node: node_) {
            self.node = node;
            alt node {
              leaf({byte_offset: ofs, content: s}) {
                self.byte_len = s.len() - ofs;
                self.char_len = str::count_chars(*s, ofs, self.byte_len);
                self.height = 0u;
              }
              concat({left: l, right: r, _}) {
                self.byte_len = l.byte_len + r.byte_len;
                self.char_len = l.char_len + r.char_len;
                self.height = uint::max(l.height, r.height) + 1u;
              }
            };
        }
    }

    type leaf = {byte_offset: uint, content: @str};
// FIXME (#2744): Perhaps a `vec` of nodes instead of two `left`/`right` nodes
    type concat = {left: @node, right: @node};

    enum node_ {
    #[doc = "
    A text component in a rope.

    This is actually a slice in a rope, so as to ensure maximal sharing.

    # Fields

    * byte_offset = The number of bytes skipped in `content`
    * content - Contents of the leaf.

        Note that we can have `char_len < str::char_len(content)`, if
        this leaf is only a subset of the string. Also note that the
        string can be shared between several ropes, e.g. for indexing
        purposes.
    "]
         leaf(leaf),
    #[doc = "
    A node obtained from the concatenation of two other nodes

    # Fields

    * left - The node containing the beginning of the text.
    * right - The node containing the end of the text.

        Used to pre-allocate the correct amount of storage for serialization.

    * height - Height of the subrope.

        Used for rebalancing and to allocate stacks for traversals.
    "]
        concat(concat),
    }

    #[doc = "
    The maximal number of chars that _should_ be permitted in a single node.

    This is not a strict value
    "]
    const hint_max_leaf_char_len: uint = 256u;

    #[doc = "
       Creates an empty rope
     "]
    fn empty() -> root { option::none }

    #[doc ="
       Creates a rope from a node
     "]
    fn content(&&n: @node) -> root { option::some(n) }

    #[doc = "
    The maximal height that _should_ be permitted in a tree.

    This is not a strict value
    "]
    const hint_max_node_height:   uint = 16u;

    #[doc = "
    Adopt a string as a node.

    If the string is longer than `max_leaf_char_len`, it is
    logically split between as many leaves as necessary. Regardless,
    the string itself is not copied.

    Performance note: The complexity of this function is linear in
    the length of `str`.
    "]
    fn of_str(str: @str) -> @node {
        ret of_substr(str, 0u, str::len(*str));
    }

    #[doc ="
    Adopt a slice of a string as a node.

    If the slice is longer than `max_leaf_char_len`, it is logically split
    between as many leaves as necessary. Regardless, the string itself
    is not copied

    # Arguments

    * byte_start - The byte offset where the slice of `str` starts.
    * byte_len   - The number of bytes from `str` to use.

    # Safety note

    Behavior is undefined if `byte_start` or `byte_len` do not represent
    valid positions in `str`
    "]
    fn of_substr(str: @str, byte_start: uint, byte_len: uint) -> @node {
        ret of_substr_unsafer(str, byte_start, byte_len,
                              str::count_chars(*str, byte_start, byte_len));
    }

    #[doc = "
    Adopt a slice of a string as a node.

    If the slice is longer than `max_leaf_char_len`, it is logically split
    between as many leaves as necessary. Regardless, the string itself
    is not copied

    # Arguments

    * byte_start - The byte offset where the slice of `str` starts.
    * byte_len - The number of bytes from `str` to use.
    * char_len - The number of chars in `str` in the interval
                 [byte_start, byte_start+byte_len(

    # Safety notes

    * Behavior is undefined if `byte_start` or `byte_len` do not represent
      valid positions in `str`
    * Behavior is undefined if `char_len` does not accurately represent the
      number of chars between byte_start and byte_start+byte_len
    "]
    fn of_substr_unsafer(str: @str, byte_start: uint, byte_len: uint,
                          char_len: uint) -> @node {
        assert(byte_start + byte_len <= str::len(*str));
        let candidate = leaf({byte_offset: byte_start, content: str});
        if char_len <= hint_max_leaf_char_len {
            ret @node(candidate);
        } else {
            //Firstly, split `str` in slices of hint_max_leaf_char_len
            let mut leaves = uint::div_ceil(char_len, hint_max_leaf_char_len);
            //Number of leaves
            let nodes  = vec::to_mut(vec::from_elem(leaves, candidate));

            let mut offset = byte_start;
            let first_leaf_char_len =
                if char_len%hint_max_leaf_char_len == 0u {
                  hint_max_leaf_char_len
                } else {
                char_len%hint_max_leaf_char_len
               };
            for uint::range(0u, leaves) {|i|
                let chunk_char_len =
                     if i == 0u  { first_leaf_char_len }
                     else { hint_max_leaf_char_len };
                let chunk_byte_len =
                    str::count_bytes(*str, offset, chunk_char_len);
                nodes[i] = leaf({byte_offset: offset, content: str});

                offset += chunk_byte_len;
            }

            //Then, build a tree from these slices by collapsing them
            while leaves > 1u {
                for uint::range(0u, uint::div_ceil(leaves, 2u)) {|j|
                    //Concat nodes 0 with 1, 2 with 3 etc.
                    nodes[j] = concat({left: @node(nodes[2*j]),
                                       right: @node(nodes[(2*j) + 1u])});
                    if j == leaves - 1u {
                  //And don't forget the last node if it is in even position
                       nodes[j] = nodes[2*j];
                    }
                }
                leaves = uint::div_ceil(leaves, 2u);
            }
            ret @node(nodes[0u]);
        }
    }

    #[doc ="
    Concatenate a forest of nodes into one tree.

    # Arguments

    * forest - The forest. This vector is progressively rewritten during
               execution and should be discarded as meaningless afterwards.
    "]
    fn tree_from_forest_destructive(forest: [mut @node]) -> @node {
        let mut i;
        let mut len = vec::len(forest);
        while len > 1u {
            i = 0u;
            while i < len - 1u {//Concat nodes 0 with 1, 2 with 3 etc.
                let mut left  = forest[i];
                let mut right = forest[i+1u];
                let left_len  = left.char_len;
                let right_len = right.char_len;
                let mut left_height= left.height;
                let mut right_height= right.height;
                if left_len + right_len > hint_max_leaf_char_len {
                    if left_len <= hint_max_leaf_char_len {
                        left = flatten(left);
                        left_height = left.height;
                    }
                    if right_len <= hint_max_leaf_char_len {
                        right = flatten(right);
                        right_height = right.height;
                    }
                }
                if left_height >= hint_max_node_height {
                    left = of_substr_unsafer(@serialize_node(left),
                                             0u, left.byte_len, left_len);
                }
                if right_height >= hint_max_node_height {
                    right = of_substr_unsafer(@serialize_node(right),
                                             0u, right.byte_len, right_len);
                }
                forest[i/2u] = concat2(left, right);
                i += 2u;
            }
            if i == len - 1u {
                //And don't forget the last node if it is in even position
                forest[i/2u] = forest[i];
            }
            len = uint::div_ceil(len, 2u);
        }
        ret forest[0];
    }

    fn serialize_node(node: @node) -> str unsafe {
        let mut buf = vec::to_mut(vec::from_elem(node.byte_len, 0u8));
        let mut offset = 0u;//Current position in the buffer
        let it = leaf_iterator::start(node);
        loop {
            alt(leaf_iterator::next(it)) {
              option::none { break; }
              option::some((byte_len, x)) {
                //TODO: Replace with memcpy or something similar
                let mut local_buf: [u8] =
                    unsafe::reinterpret_cast(*x.content);
                for uint::range(x.byte_offset, byte_len) {|i|
                    buf[offset] = local_buf[i];
                    offset += 1u;
                }
                unsafe::forget(local_buf);
              }
            }
        }
        ret unsafe::transmute(buf);
    }

    #[doc ="
    Replace a subtree by a single leaf with the same contents.

    * Performance note

    This function executes in linear time.
    "]
    fn flatten(n: @node) -> @node unsafe {
        alt(n.node) {
          leaf(_) { n }
          concat(_) {
            @node(leaf({
                byte_offset: 0u,
                content:     @serialize_node(n)
            }))
          }
        }
    }

    #[doc ="
    Balance a node.

    # Algorithm

    * if the node height is smaller than `hint_max_node_height`, do nothing
    * otherwise, gather all leaves as a forest, rebuild a balanced node,
      concatenating small leaves along the way

    # Return value

    * `option::none` if no transformation happened
    * `option::some(x)` otherwise, in which case `x` has the same contents
       as `node` bot lower height and/or fragmentation.
    "]
    fn bal(n: @node) -> option<@node> {
        if n.height < hint_max_node_height { ret option::none; }
        //1. Gather all leaves as a forest
<<<<<<< Updated upstream
        let mut forest = [mut];
        let it = leaf_iterator::start(node);
=======
        let mut forest = [mut]/~;
        let it = leaf_iterator::start(n);
>>>>>>> Stashed changes
        loop {
            alt (leaf_iterator::next(it)) {
              option::none       { break; }
              option::some((_, x)) { vec::push(forest, @node(leaf(x))); }
            }
        }
        //2. Rebuild tree from forest
        option::some(@*tree_from_forest_destructive(forest))
    }

    #[doc ="
    Compute the subnode of a node.

    # Arguments

    * node        - A node
    * byte_offset - A byte offset in `node`
    * byte_len    - The number of bytes to return

    # Performance notes

    * this function performs no copying;
    * this function executes in a time proportional to the height of `node`.

    # Safety notes

    This function fails if `byte_offset` or `byte_len` do not represent
    valid positions in `node`.
    "]
    fn sub_bytes(n: @node, byte_offset: uint, byte_len: uint) -> @node {
        let mut n = n;
        let mut byte_offset = byte_offset;
        loop {
            if byte_offset == 0u && byte_len == n.byte_len {
                ret n;
            }
            alt(n.node) {
              node::leaf(x) {
                ret @node(leaf({byte_offset: byte_offset,
                                content:     x.content}));
              }
              node::concat(x) {
                let left_len: uint = x.left.byte_len;
                if byte_offset <= left_len {
                    if byte_offset + byte_len <= left_len {
                        //Case 1: Everything fits in x.left, tail-call
                        n = x.left;
                    } else {
                        //Case 2: A (non-empty, possibly full) suffix
                        //of x.left and a (non-empty, possibly full) prefix
                        //of x.right
                        let left_result  =
                            sub_bytes(x.left, byte_offset, left_len);
                        let right_result =
                            sub_bytes(x.right, 0u, left_len - byte_offset);
                        ret concat2(left_result, right_result);
                    }
                } else {
                    //Case 3: Everything fits in x.right
                    byte_offset -= left_len;
                    n = x.right;
                }
              }
            }
        };
    }

    #[doc ="
    Compute the subnode of a node.

    # Arguments

    * node        - A node
    * char_offset - A char offset in `node`
    * char_len    - The number of chars to return

    # Performance notes

    * this function performs no copying;
    * this function executes in a time proportional to the height of `node`.

    # Safety notes

    This function fails if `char_offset` or `char_len` do not represent
    valid positions in `node`.
    "]
    fn sub_chars(n: @node, char_offset: uint, char_len: uint) -> @node {
        let mut n           = n;
        let mut char_offset = char_offset;
        loop {
            alt(n.node) {
              node::leaf(x) {
                if char_offset == 0u && char_len == n.char_len {
                    ret n;
                }
                let byte_offset =
                    str::count_bytes(*x.content, 0u, char_offset);
                ret @node(leaf({byte_offset: byte_offset,
                                content: x.content}));
              }
              node::concat(x) {
                if char_offset == 0u && char_len == n.char_len {ret n;}
                let left_len : uint = x.left.char_len;
                if char_offset <= left_len {
                    if char_offset + char_len <= left_len {
                        //Case 1: Everything fits in x.left, tail call
                        n = x.left;
                    } else {
                        //Case 2: A (non-empty, possibly full) suffix
                        //of x.left and a (non-empty, possibly full) prefix
                        //of x.right
                        let left_result  =
                            sub_chars(x.left, char_offset, left_len);
                        let right_result =
                            sub_chars(x.right, 0u, left_len - char_offset);
                        ret concat2(left_result, right_result);
                    }
                } else {
                    //Case 3: Everything fits in x.right, tail call
                    n = x.right;
                    char_offset -= left_len;
                }
              }
            }
        };
    }

    fn concat2(left: @node, right: @node) -> @node {
        @node(concat({left: left, right: right}))
    }

    fn cmp(a: @node, b: @node) -> int {
        let ita = char_iterator::start(a);
        let itb = char_iterator::start(b);
        let mut result = 0;
        while result == 0 {
            alt((char_iterator::next(ita), char_iterator::next(itb))) {
              (option::none, option::none) {
                break;
              }
              (option::some(chara), option::some(charb)) {
                result = char::cmp(chara, charb);
              }
              (option::some(_), _)         {
                result = 1;
              }
              (_, option::some(_))         {
                result = -1;
              }
            }
        }
        ret result;
    }

    fn loop_chars(node: @node, it: fn(char) -> bool) -> bool {
        ret loop_leaves(node, {|byte_len, leaf|
            str::all_between(*leaf.content,
                             leaf.byte_offset,
                             byte_len, it)
        })
    }

    #[doc ="
    Loop through a node, leaf by leaf

    # Arguments

    * rope - A node to traverse.
    * it - A block to execute with each consecutive leaf of the node.
           Return `true` to continue, `false` to stop

    # Arguments

    `true` If execution proceeded correctly, `false` if it was interrupted,
    that is if `it` returned `false` at any point.
    "]
    fn loop_leaves(node: @node, it: fn(uint, leaf) -> bool) -> bool{
        let mut current = node;
        loop {
            alt(current.node) {
              leaf(x) {
                ret it(current.byte_len, x);
              }
              concat(x) {
                if loop_leaves(x.left, it) { //non tail call
                    current = x.right;       //tail call
                } else {
                    ret false;
                }
              }
            }
        };
    }

    #[doc ="
    # Arguments

    * pos - A position in the rope

    # Return value

    The character at position `pos`

    # Safety notes

    The function will fail if `pos` is not a valid position in the rope.

    Performance note: This function executes in a time
    proportional to the height of the rope + the (bounded)
    length of the largest leaf.
    "]
    fn char_at(node: @node, pos: uint) -> char {
        let mut node    = node;
        let mut pos     = pos;
        loop {
            alt node.node {
              leaf(x) {
                ret str::char_at(*x.content, pos);
              }
              concat({left, right}) {
                let left_len = left.char_len;
                node = if left_len > pos { left }
                       else { pos -= left_len; right };
              }
            }
        };
    }

    mod leaf_iterator {
        type t = {
            stack:            [mut @node],
            mut stackpos: int
        };

        fn empty() -> t {
            let stack : [mut @node] = [mut];
            ret {stack: stack, mut stackpos: -1}
        }

        fn start(node: @node) -> t {
            let stack = vec::to_mut(vec::from_elem(node.height +1u, node));
            ret {
                stack:             stack,
                mut stackpos:  0
            }
        }

        // Returns a pair of a byte len and a leaf
        // tjc: kind of awkward
        fn next(it: t) -> option<(uint, leaf)> {
            if it.stackpos < 0 { ret option::none; }
            loop {
                let current = it.stack[it.stackpos];
                it.stackpos -= 1;
                alt(current.node) {
                  concat(x) {
                    it.stackpos += 1;
                    it.stack[it.stackpos] = x.right;
                    it.stackpos += 1;
                    it.stack[it.stackpos] = x.left;
                  }
                  leaf(x) {
                    ret option::some((current.byte_len, x));
                  }
                }
            };
        }
    }

    mod char_iterator {
        type t = {
            leaf_iterator: leaf_iterator::t,
            mut leaf:  option<(uint, leaf)>,
            mut leaf_byte_pos: uint
        };

        fn start(node: @node) -> t {
            ret {
                leaf_iterator: leaf_iterator::start(node),
                mut leaf:          option::none,
                mut leaf_byte_pos: 0u
            }
        }

        fn empty() -> t {
            ret {
                leaf_iterator: leaf_iterator::empty(),
                mut leaf:  option::none,
                mut leaf_byte_pos: 0u
            }
        }

        fn next(it: t) -> option<char> {
            loop {
                alt(get_current_or_next_leaf(it)) {
                  option::none { ret option::none; }
                  option::some(_) {
                    let next_char = get_next_char_in_leaf(it);
                    alt(next_char) {
                      option::none {
                        cont;
                      }
                      option::some(_) {
                        ret next_char;
                      }
                    }
                  }
                }
            };
        }

        fn get_current_or_next_leaf(it: t) -> option<(uint, leaf)> {
            alt(it.leaf) {
              option::some(_) { ret it.leaf }
              option::none {
                let next = leaf_iterator::next(it.leaf_iterator);
                alt(next) {
                  option::none { ret option::none }
                  option::some(_) {
                    it.leaf          = next;
                    it.leaf_byte_pos = 0u;
                    ret next;
                  }
                }
              }
            }
        }

        fn get_next_char_in_leaf(it: t) -> option<char> {
            alt copy it.leaf {
              option::none { ret option::none }
              option::some((byte_len, aleaf)) {
                if it.leaf_byte_pos >= byte_len {
                    //We are actually past the end of the leaf
                    it.leaf = option::none;
                    ret option::none
                } else {
                    let {ch, next} =
                        str::char_range_at(*aleaf.content,
                                     it.leaf_byte_pos + aleaf.byte_offset);
                    it.leaf_byte_pos = next - aleaf.byte_offset;
                    ret option::some(ch)
                }
              }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    //Utility function, used for sanity check
    fn rope_to_string(r: rope) -> str {
        alt(r) {
          option::none { ret "" }
          option::some(x) {
            let str = @mut "";
            fn aux(str: @mut str, node: @node::node) unsafe {
                alt(node.node) {
                  node::leaf(x) {
                    *str += str::slice(
                        *x.content, x.byte_offset,
                        x.byte_offset + node.byte_len);
                  }
                  node::concat(x) {
                    aux(str, x.left);
                    aux(str, x.right);
                  }
                }
            }
            aux(str, x);
            ret *str
          }
        }
    }


    #[test]
    fn trivial() {
        assert char_len(empty()) == 0u;
        assert byte_len(empty()) == 0u;
    }

    #[test]
    fn of_string1() {
        let sample = @"0123456789ABCDE";
        let r      = of_str(sample);

        assert char_len(r) == str::char_len(*sample);
        assert rope_to_string(r) == *sample;
    }

    #[test]
    fn of_string2() {
        let buf = @ mut "1234567890";
        let mut i = 0;
        while i < 10 { *buf = *buf + *buf; i+=1;}
        let sample = @*buf;
        let r      = of_str(sample);
        assert char_len(r) == str::char_len(*sample);
        assert rope_to_string(r) == *sample;

        let mut string_iter = 0u;
        let string_len  = str::len(*sample);
        let rope_iter   = iterator::char::start(r);
        let mut equal   = true;
        while equal {
            alt(node::char_iterator::next(rope_iter)) {
              option::none {
                if string_iter < string_len {
                    equal = false;
                } break; }
              option::some(c) {
                let {ch, next} = str::char_range_at(*sample, string_iter);
                string_iter = next;
                if ch != c { equal = false; break; }
              }
            }
        }

        assert equal;
    }

    #[test]
    fn iter1() {
        let buf = @ mut "1234567890";
        let mut i = 0;
        while i < 10 { *buf = *buf + *buf; i+=1;}
        let sample = @*buf;
        let r      = of_str(sample);

        let mut len = 0u;
        let it  = iterator::char::start(r);
        loop {
            alt(node::char_iterator::next(it)) {
              option::none { break; }
              option::some(_) { len += 1u; }
            }
        }

        assert len == str::char_len(*sample);
    }

    #[test]
    fn bal1() {
        let num = 5; // 8

        let init = @ "1234567890";
        let buf  = @ mut * init;
        let mut i = 0;
        while i < num { *buf = *buf + *buf; i+=1;}
        let sample = @*buf;
        let r1     = of_str(sample);
        let mut r2 = of_str(init);
        i = 0;
        while i < num { r2 = append_rope(r2, r2); i+= 1;}

        #error("~~~ sample = %s", *sample);

        if !eq(r1, r2) {
            #error("+++ r1 = %?", r1);
            #error("--- r2 = %?", r2);
        }
        assert eq(r1, r2);
        let r3 = bal(r2);
        assert char_len(r1) == char_len(r3);

        assert eq(r1, r3);
    }

    #[test]
    #[ignore]
    fn char_at1() {
        //Generate a large rope
        let mut r = of_str(@ "123456789");
        for uint::range(0u, 10u){|_i|
            r = append_rope(r, r);
        }

        //Copy it in the slowest possible way
        let mut r2 = empty();
        for uint::range(0u, char_len(r)){|i|
            r2 = append_char(r2, char_at(r, i));
        }
        assert eq(r, r2);

        let mut r3 = empty();
        for uint::range(0u, char_len(r)){|i|
            r3 = prepend_char(r3, char_at(r, char_len(r) - i - 1u));
        }
        assert eq(r, r3);

        //Additional sanity checks
        let balr = bal(r);
        let bal2 = bal(r2);
        let bal3 = bal(r3);
        assert eq(r, balr);
        assert eq(r, bal2);
        assert eq(r, bal3);
        assert eq(r2, r3);
        assert eq(bal2, bal3);
    }

    #[test]
    fn concat1() {
        //Generate a reasonable rope
        let chunk = of_str(@ "123456789");
        let mut r = empty();
        for uint::range(0u, 10u){|_i|
            r = append_rope(r, chunk);
        }

        //Same rope, obtained with rope::concat
        let r2 = concat(vec::from_elem(10u, chunk));

        assert eq(r, r2);
    }
}
