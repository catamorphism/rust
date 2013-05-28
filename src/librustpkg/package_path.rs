// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// rustpkg utilities having to do with local and remote paths

use core::hash;
use core::hash::Streaming;
use core::rt::io::Writer;
use core::prelude::*;

/// Wrappers to prevent local and remote paths from getting confused
/// (These will go away after #6407)
pub struct RemotePath (Path);
pub struct LocalPath (Path);


// normalize should be the only way to construct a LocalPath
// (though this isn't enforced)
/// Replace all occurrences of '-' in the stem part of path with '_'
/// This is because we treat rust-foo-bar-quux and rust_foo_bar_quux
/// as the same name
pub fn normalize(p_: RemotePath) -> LocalPath {
    let RemotePath(p) = p_;
    match p.filestem() {
        None => LocalPath(p),
        Some(st) => {
            let replaced = str::replace(st, "-", "_");
            if replaced != st {
                LocalPath(p.with_filestem(replaced))
            }
            else {
                LocalPath(p)
            }
        }
    }
}

pub fn hash(data: ~str) -> ~str {
    let mut hasher = hash::default_state();
    let buffer = str::as_bytes_slice(data);
    hasher.write(buffer);
    hasher.result_str()
}
