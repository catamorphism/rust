// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use context::*;
use crate::*;
use package_id::*;
use package_source::*;
use version::Version;
use workcache_support::*;

use extra::arc::{Arc,RWArc};
use extra::workcache;
use extra::workcache::*;
use std::os;
use extra::treemap::TreeMap;

/// Convenience functions intended for calling from pkg.rs

/// p is where to put the cache file for dependencies
pub fn default_ctxt(p: Path) -> BuildCtx {
    new_default_ctx(new_workcache_cx(&p), p)
}

pub fn new_default_ctx(c: Context, p: Path) -> BuildCtx {
    BuildCtx {
        cx: Ctx { sysroot_opt: p },
        workcache_cx: c,
    }
}

fn file_is_fresh(path: &str, in_hash: &str) -> bool {
// NOTE: Probably won't work; the in_hash doesn't take the date
// into account
    in_hash == digest_file_with_date(&Path(path))
}

fn binary_is_fresh(path: &str, in_hash: &str) -> bool {
// NOTE: Probably won't work; the in_hash doesn't take the date
// into account
    in_hash == digest_only_date(&Path(path))
}


pub fn new_workcache_cx(p: &Path) -> Context {
    let db_file = p.push("rustpkg_db.json"); // ??? probably wrong
    debug!("Workcache database file: %s", db_file.to_str());
    let db = RWArc::new(Database::new(db_file));
    let lg = RWArc::new(Logger::new());
    let cfg = Arc::new(TreeMap::new());
    let mut rslt: FreshnessMap = TreeMap::new();
// NOTE: Here's where we should set up freshness functions for all the types
// of things we'd like to handle in rustpkg
// * file (source file)
// * url  (version-control URL)
// * exe  (executable)
// * lib  (library)
    rslt.insert(~"file", file_is_fresh);
    rslt.insert(~"binary", binary_is_fresh);
    workcache::Context::new_with_freshness(db, lg, cfg, Arc::new(rslt))
}

pub fn build_lib(sysroot: Path, root: Path, dest: Path, name: ~str, version: Version,
                 lib: Path) {
    let pkg_src = PkgSrc {
        root: root,
        id: PkgId{ version: version, ..PkgId::new(name)},
        libs: ~[mk_crate(lib)],
        mains: ~[],
        tests: ~[],
        benchs: ~[]
    };
    pkg_src.build(&default_ctxt(sysroot), dest, ~[]);
}

pub fn build_exe(sysroot: Path, root: Path, dest: Path, name: ~str, version: Version,
                 main: Path) {
    let pkg_src = PkgSrc {
        root: root,
        id: PkgId{ version: version, ..PkgId::new(name)},
        libs: ~[],
        mains: ~[mk_crate(main)],
        tests: ~[],
        benchs: ~[]
    };
    pkg_src.build(&default_ctxt(sysroot), dest, ~[]);
}

pub fn install_lib(sysroot: Path,
                   workspace: Path,
                   name: ~str,
                   lib_path: Path,
                   version: Version) {
    debug!("self_exe: %?", os::self_exe_path());
    debug!("sysroot = %s", sysroot.to_str());
    debug!("workspace = %s", workspace.to_str());
    // make a PkgSrc
    let pkg_id = PkgId{ version: version, ..PkgId::new(name)};
    let pkg_src = PkgSrc {
        root: workspace.clone(),
        id: pkg_id.clone(),
        libs: ~[mk_crate(lib_path)],
        mains: ~[],
        tests: ~[],
        benchs: ~[]
    };
    let cx = default_ctxt(sysroot);
    pkg_src.build(&cx, workspace.clone(), ~[]);
    cx.install_no_build(&workspace, &pkg_id);
}

pub fn install_exe(sysroot: Path, workspace: Path, name: ~str, version: Version) {
    let nm = fmt!("install %s", name);
    let cx = default_ctxt(sysroot);
    do cx.workcache_cx.with_prep(nm) |prep| {
       let sub_version = version.clone();
       let sub_workspace = workspace.clone();
       let sub_name = name.clone();
       let mycx = cx.clone();
       do prep.exec |exec| {
          mycx.install(exec, &sub_workspace, &PkgId{ version: sub_version.clone(),
                                            ..PkgId::new(sub_name)});
       }};
}

fn mk_crate(p: Path) -> Crate {
    Crate { file: p, flags: ~[], cfgs: ~[] }
}
