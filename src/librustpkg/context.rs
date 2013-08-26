// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Context data structure used by rustpkg

use std::os;
use extra::workcache;

// NOTE: aaaaargh
#[deriving(Clone)]
pub struct Ctx {
    // Sysroot
    sysroot_opt: Path // Not sure if this should be an Option, but
                      // workcache ctxt needs it to figure out where
                      // to put the database file
}

#[deriving(Clone)]
pub struct BuildCtx {
    // Context for workcache
    workcache_cx: workcache::Context,
    // Everything else
    cx: Ctx
}

impl BuildCtx {
    pub fn sysroot_opt(&self) -> Path {
        self.cx.sysroot_opt.clone() // NOTE: :-(
    }

    pub fn sysroot_to_use(&self) -> Option<@Path> {
        self.cx.sysroot_to_use()
    }
}

impl Ctx {
    pub fn sysroot_opt(&self) -> Path {
        self.sysroot_opt.clone() // NOTE: :-(
    }
}

impl Ctx {
    /// Debugging
    pub fn sysroot_opt_str(&self) -> ~str {
        self.sysroot_opt.to_str()
    }

    // Hack so that rustpkg can run either out of a rustc target dir,
    // or the host dir
    pub fn sysroot_to_use(&self) -> Option<@Path> {
        Some(@(if !in_target(&self.sysroot_opt) {
            self.sysroot_opt.clone() // :-(
        }
        else {
            self.sysroot_opt.pop().pop().pop()
        }
    ))
   }
}

/// We assume that if ../../rustc exists, then we're running
/// rustpkg from a Rust target directory. This is part of a
/// kludgy hack used to adjust the sysroot.
pub fn in_target(sysroot_opt: &Path) -> bool {
    debug!("Checking whether %s is in target", sysroot_opt.to_str());
    os::path_is_dir(&sysroot_opt.pop().pop().push("rustc"))
}
