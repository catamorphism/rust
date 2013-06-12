// Copyright 2013 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// rustpkg unit tests

use context::Ctx;
use core::hashmap::HashMap;
use core::{io, libc, os, result, run, str};
use core::prelude::*;
use extra::tempfile::mkdtemp;
use core::run::ProcessOutput;
use package_path::*;
use package_id::{PkgId};
use package_source::*;
use version::{ExactRevision, NoVersion, Version};
use path_util::{target_executable_in_workspace, target_library_in_workspace,
               target_test_in_workspace, target_bench_in_workspace,
               make_dir_rwx, u_rwx, library_in_workspace,
               built_bench_in_workspace, built_test_in_workspace,
               built_library_in_workspace, built_executable_in_workspace,
                installed_library_in_workspace};
use target::*;

/// Returns the last-modified date as an Option
fn datestamp(p: &Path) -> Option<libc::time_t> {
    p.stat().map(|stat| stat.st_mtime)
}

fn fake_ctxt(sysroot_opt: Option<@Path>) -> Ctx {
    Ctx {
        sysroot_opt: sysroot_opt,
        json: false,
        dep_cache: @mut HashMap::new()
    }
}

fn fake_pkg() -> PkgId {
    let sn = ~"bogus";
    let remote = RemotePath(Path(sn));
    PkgId {
        local_path: normalize(copy remote),
        remote_path: remote,
        short_name: sn,
        version: NoVersion
    }
}

fn remote_pkg() -> PkgId {
    let remote = RemotePath(Path("github.com/catamorphism/test-pkg"));
    PkgId {
        local_path: normalize(copy remote),
        remote_path: remote,
        short_name: ~"test_pkg",
        version: NoVersion
    }
}

fn writeFile(file_path: &Path, contents: &str) {
    let out: @io::Writer =
        result::get(&io::file_writer(file_path,
                                     [io::Create, io::Truncate]));
    out.write_line(contents);
}

fn mk_empty_workspace(short_name: &LocalPath, version: &Version) -> Path {
    let workspace_dir = mkdtemp(&os::tmpdir(), "test").expect("couldn't create temp dir");
    mk_workspace(&workspace_dir, short_name, version);
    workspace_dir
}

fn mk_workspace(workspace: &Path, short_name: &LocalPath, version: &Version) -> Path {
    // include version number in directory name
    let package_dir = workspace.push("src").push(fmt!("%s%s",
                                                      short_name.to_str(), version.to_str()));
    assert!(os::mkdir_recursive(&package_dir, u_rwx));
    package_dir
}

fn mk_temp_workspace(short_name: &LocalPath, version: &Version) -> Path {
    let package_dir = mk_empty_workspace(short_name,
                                         version).push("src").push(fmt!("%s%s",
                                                            short_name.to_str(),
                                                            version.to_str()));

    debug!("Created %s and does it exist? %?", package_dir.to_str(),
          os::path_is_dir(&package_dir));
    // Create main, lib, test, and bench files
    debug!("mk_workspace: creating %s", package_dir.to_str());
    assert!(os::mkdir_recursive(&package_dir, u_rwx));
    debug!("Created %s and does it exist? %?", package_dir.to_str(),
          os::path_is_dir(&package_dir));
    // Create main, lib, test, and bench files

    writeFile(&package_dir.push("main.rs"),
              "fn main() { let _x = (); }");
    writeFile(&package_dir.push("lib.rs"),
              "pub fn f() { let _x = (); }");
    writeFile(&package_dir.push("test.rs"),
              "#[test] pub fn f() { (); }");
    writeFile(&package_dir.push("bench.rs"),
              "#[bench] pub fn f() { (); }");
    package_dir
}

fn is_rwx(p: &Path) -> bool {
    use core::libc::consts::os::posix88::{S_IRUSR, S_IWUSR, S_IXUSR};

    match p.get_mode() {
        None => return false,
        Some(m) =>
            ((m & S_IRUSR as uint) == S_IRUSR as uint
            && (m & S_IWUSR as uint) == S_IWUSR as uint
            && (m & S_IXUSR as uint) == S_IXUSR as uint)
    }
}

fn test_sysroot() -> Path {
    // Totally gross hack but it's just for test cases.
    // Infer the sysroot from the exe name and pray that it's right.
    // (Did I mention it was a gross hack?)
    let self_path = os::self_exe_path().expect("Couldn't get self_exe path");
    self_path.pop()
}

fn command_line_test(cmd: &str, args: &[~str], cwd: &Path) -> ProcessOutput {
    debug!("About to run command: %? %? in %s", cmd, args, cwd.to_str());
    let mut prog = run::Process::new(cmd, args, run::ProcessOptions { env: None,
                                                           dir: Some(cwd),
                                                           in_fd: None,
                                                           out_fd: None,
                                                           err_fd: None
                                                          });
    let output = prog.finish_with_output();
    io::println(fmt!("Output from command %s with args %? was %s {%s}[%?]",
                    cmd, args, str::from_bytes(output.output),
                   str::from_bytes(output.error),
                   output.status));
/*
By the way, rustpkg *wont'* return a nonzero exit code if it fails --
see #4547
*/
    if output.status != 0 {
        fail!("Command %s %? failed with exit code %?",
              cmd, args, output.status);
    }
    output
}

fn make_git_repo(short_name: &str) -> Path {
    let temp_d = mk_temp_workspace(&normalize(RemotePath(Path(short_name))), &NoVersion);
    debug!("Dry run: would initialize %s as a git repository", temp_d.to_str());
    temp_d
}

fn add_git_tag(repo: &Path, tag: &str) {
    debug!("Dry run: would add tag %s to repo %s", tag, repo.to_str());
}

fn create_local_package(pkgid: &str) -> Path {
    let parent_dir = mk_temp_workspace(&normalize(RemotePath(Path(pkgid))), &NoVersion);
    debug!("Created empty package dir for %s, returning %s", pkgid, parent_dir.to_str());
    parent_dir.pop().pop()
}

fn create_local_package_in(pkgid: &str, pkgdir: &Path, version: &Version) -> Path {

    let package_dir = pkgdir.push("src").push(fmt!("%s%s",
                                                   pkgid,
                                                   version.to_str()));

    // Create main, lib, test, and bench files
    assert!(os::mkdir_recursive(&package_dir, u_rwx));
    debug!("Created %s and does it exist? %?", package_dir.to_str(),
          os::path_is_dir(&package_dir));
    // Create main, lib, test, and bench files

    writeFile(&package_dir.push("main.rs"),
              "fn main() { let _x = (); }");
    writeFile(&package_dir.push("lib.rs"),
              "pub fn f() { let _x = (); }");
    writeFile(&package_dir.push("test.rs"),
              "#[test] pub fn f() { (); }");
    writeFile(&package_dir.push("bench.rs"),
              "#[bench] pub fn f() { (); }");
    package_dir
}

fn create_local_package_with_version(short_name: &str, version: &str) -> Path {
    debug!("Dry run -- would create package %s with version %s",
           short_name, version);
    create_local_package(short_name)
        // actually write the version
}

fn create_local_package_with_test(pkgid: &str) -> Path {
    debug!("Dry run -- would create package %s with test");
    create_local_package(pkgid) // Already has tests???
}

fn create_local_package_with_dep(pkgid: &str, subord_pkgid: &str, version: &Version) -> Path {
    let package_dir = create_local_package(pkgid);
    create_local_package_in(subord_pkgid, &package_dir, version);
    // Write a main.rs file into pkgid that references subord_pkgid
    writeFile(&package_dir.push("src").push(pkgid).push("main.rs"),
              fmt!("extern mod %s;\nfn main() {}",
                   subord_pkgid));
    // Write a lib.rs file into subord_pkgid that has something in it
    writeFile(&package_dir.push("src").push(subord_pkgid).push("lib.rs"),
              "pub fn f() {}");
    debug!("Dry run -- would create packages %s and %s in %s", pkgid, subord_pkgid,
           package_dir.to_str());
    package_dir
}

fn create_local_package_with_custom_build_hook(pkgid: &str,
                                               custom_build_hook: &str) -> Path {
    debug!("Dry run -- would create package %s with custom build hook %s",
           pkgid, custom_build_hook);
    create_local_package(pkgid)
    // actually write the pkg.rs with the custom build hook

}

fn assert_lib_exists(repo: &Path, short_name: &str) {
    let lib = target_library_in_workspace(&PkgId::new(short_name), repo);
    assert!(os::path_exists(&lib));
    assert!(is_rwx(&lib));
}

fn command_line_test_output(command: &str, args: &[~str]) -> ~[~str] {
    let mut result = ~[];
    for str::from_bytes(run::process_output(command, args).output).each_split_char('\n') |s| {
        result += [s.to_owned()];
    }
    result
}

fn lib_output_file_name(workspace: &Path, parent: &str, short_name: &str) -> Path {
    debug!("lib_output_file_name: given %s and parent %s and short name %s",
           workspace.to_str(), parent, short_name);
    library_in_workspace(short_name,
                         Build,
                         workspace,
                         "build").expect("lib_output_file_name")
}

fn output_file_name(workspace: &Path, short_name: &str) -> Path {
    workspace.push(fmt!("%s%s", short_name, os::EXE_SUFFIX))
}

fn touch_source_file(workspace: &Path, short_name: &str) {
    use conditions::bad_path::cond;
    let pkg_src_dir = workspace.push("src").push(short_name);
    let contents = os::list_dir(&pkg_src_dir);
    for contents.each() |p| {
        if Path(copy *p).filetype() == Some(~".rs") {
            // should be able to do this w/o a process
            if run::process_output("touch", [p.to_str()]).status != 0 {
                let _ = cond.raise((copy pkg_src_dir, ~"Bad path"));
            }
            break;
        }
    }
}

/// Add a blank line at the end
fn frob_source_file(workspace: &Path, short_name: &str) {
    use conditions::bad_path::cond;
    let pkg_src_dir = workspace.push("src").push(short_name);
    let contents = os::list_dir(&pkg_src_dir);
    let mut maybe_p = None;
    for contents.each() |p| {
        if Path(copy *p).filetype() == Some(~".rs") {
            maybe_p = Some(p);
            break;
        }
    }
    match maybe_p {
        Some(p) => {
            let p = Path(copy *p);
            let w = io::buffered_file_writer(&p);
            match w {
                Err(s) => { let _ = cond.raise((p, fmt!("Bad path: %s", s))); }
                Ok(w)  => w.write_line("")
            }
        }
        None => fail!(fmt!("frob_source_file failed to find a source file in %s",
                           pkg_src_dir.to_str()))
    }
}

#[test]
fn test_make_dir_rwx() {
    let temp = &os::tmpdir();
    let dir = temp.push("quux");
    assert!(!os::path_exists(&dir) ||
            os::remove_dir_recursive(&dir));
    debug!("Trying to make %s", dir.to_str());
    assert!(make_dir_rwx(&dir));
    assert!(os::path_is_dir(&dir));
    assert!(is_rwx(&dir));
    assert!(os::remove_dir_recursive(&dir));
}

#[test]
fn test_install_valid() {
    use path_util::installed_library_in_workspace;

    let sysroot = test_sysroot();
    debug!("sysroot = %s", sysroot.to_str());
    let ctxt = fake_ctxt(Some(@sysroot));
    let temp_pkg_id = fake_pkg();
    let temp_workspace = mk_temp_workspace(&temp_pkg_id.local_path, &NoVersion);
    // should have test, bench, lib, and main
    ctxt.install(&temp_workspace, &temp_pkg_id);
    // Check that all files exist
    let exec = target_executable_in_workspace(&temp_pkg_id, &temp_workspace);
    debug!("exec = %s", exec.to_str());
    assert!(os::path_exists(&exec));
    assert!(is_rwx(&exec));

    let lib = installed_library_in_workspace(temp_pkg_id.short_name, &temp_workspace);
    debug!("lib = %?", lib);
    assert!(lib.map_default(false, |l| os::path_exists(l)));
    assert!(lib.map_default(false, |l| is_rwx(l)));

    // And that the test and bench executables aren't installed
    assert!(!os::path_exists(&target_test_in_workspace(&temp_pkg_id, &temp_workspace)));
    let bench = target_bench_in_workspace(&temp_pkg_id, &temp_workspace);
    debug!("bench = %s", bench.to_str());
    assert!(!os::path_exists(&bench));
}

#[test]
fn test_install_invalid() {
    use conditions::nonexistent_package::cond;
    use cond1 = conditions::missing_pkg_files::cond;

    let ctxt = fake_ctxt(None);
    let pkgid = fake_pkg();
    let temp_workspace = mkdtemp(&os::tmpdir(), "test").expect("couldn't create temp dir");
    let mut error_occurred = false;
    let mut error1_occurred = false;
    do cond1.trap(|_| {
        error1_occurred = true;
    }).in {
        do cond.trap(|_| {
            error_occurred = true;
            copy temp_workspace
        }).in {
            ctxt.install(&temp_workspace, &pkgid);
        }
    }
    assert!(error_occurred && error1_occurred);
}

#[test]
fn test_install_url() {
    let workspace = mkdtemp(&os::tmpdir(), "test").expect("couldn't create temp dir");
    let sysroot = test_sysroot();
    debug!("sysroot = %s", sysroot.to_str());
    let ctxt = fake_ctxt(Some(@sysroot));
    let temp_pkg_id = remote_pkg();
    // should have test, bench, lib, and main
    ctxt.install(&workspace, &temp_pkg_id);
    // Check that all files exist
    let exec = target_executable_in_workspace(&temp_pkg_id, &workspace);
    debug!("exec = %s", exec.to_str());
    assert!(os::path_exists(&exec));
    assert!(is_rwx(&exec));
    let _built_lib =
        built_library_in_workspace(&temp_pkg_id,
                                   &workspace).expect("test_install_url: built lib should exist");
    let lib = target_library_in_workspace(&temp_pkg_id, &workspace);
    debug!("lib = %s", lib.to_str());
    assert!(os::path_exists(&lib));
    assert!(is_rwx(&lib));
    let built_test = built_test_in_workspace(&temp_pkg_id,
                         &workspace).expect("test_install_url: built test should exist");
    assert!(os::path_exists(&built_test));
    let built_bench = built_bench_in_workspace(&temp_pkg_id,
                          &workspace).expect("test_install_url: built bench should exist");
    assert!(os::path_exists(&built_bench));
    // And that the test and bench executables aren't installed
    let test = target_test_in_workspace(&temp_pkg_id, &workspace);
    assert!(!os::path_exists(&test));
    debug!("test = %s", test.to_str());
    let bench = target_bench_in_workspace(&temp_pkg_id, &workspace);
    debug!("bench = %s", bench.to_str());
    assert!(!os::path_exists(&bench));
}

#[test]
fn test_package_ids_must_be_relative_path_like() {
    use conditions::bad_pkg_id::cond;

    /*
    Okay:
    - One identifier, with no slashes
    - Several slash-delimited things, with no / at the root

    Not okay:
    - Empty string
    - Absolute path (as per os::is_absolute)

    */

    let whatever = PkgId::new("foo");

    assert_eq!(~"foo", whatever.to_str());
    assert!("github.com/catamorphism/test_pkg" ==
            PkgId::new("github.com/catamorphism/test-pkg").to_str());

    do cond.trap(|(p, e)| {
        assert!("" == p.to_str());
        assert!("0-length pkgid" == e);
        copy whatever
    }).in {
        let x = PkgId::new("");
        assert_eq!(~"foo", x.to_str());
    }

    do cond.trap(|(p, e)| {
        assert_eq!(p.to_str(), os::make_absolute(&Path("foo/bar/quux")).to_str());
        assert!("absolute pkgid" == e);
        copy whatever
    }).in {
        let z = PkgId::new(os::make_absolute(&Path("foo/bar/quux")).to_str());
        assert_eq!(~"foo", z.to_str());
    }

}

#[test]
fn test_package_version() {
    let temp_pkg_id = PkgId::new("github.com/catamorphism/test_pkg_version");
    match temp_pkg_id.version {
        ExactRevision(~"0.4") => (),
        _ => fail!(fmt!("test_package_version: package version was %?, expected Some(0.4)",
                        temp_pkg_id.version))
    }
    let temp = mk_empty_workspace(&LocalPath(Path("test_pkg_version")), &temp_pkg_id.version);
    let ctx = fake_ctxt(Some(@test_sysroot()));
    ctx.build(&temp, &temp_pkg_id);
    assert!(match built_library_in_workspace(&temp_pkg_id, &temp) {
        Some(p) => p.to_str().ends_with(fmt!("0.4%s", os::consts::DLL_SUFFIX)),
        None    => false
    });
    assert!(built_executable_in_workspace(&temp_pkg_id, &temp)
            == Some(temp.push("build").
                    push("github.com").
                    push("catamorphism").
                    push("test_pkg_version").
                    push("test_pkg_version")));
}

// FIXME #7006: Fails on linux for some reason
#[test]
#[ignore(cfg(target_os = "linux"))]
fn test_package_request_version() {
    let temp_pkg_id = PkgId::new("github.com/catamorphism/test_pkg_version#0.3");
    let temp = mk_empty_workspace(&LocalPath(Path("test_pkg_version")), &ExactRevision(~"0.3"));
    let pkg_src = PkgSrc::new(&temp, &temp, &temp_pkg_id);
    match temp_pkg_id.version {
        ExactRevision(~"0.3") => {
            match pkg_src.fetch_git() {
                Some(p) => {
                    assert!(os::path_exists(&p.push("version-0.3-file.txt")));
                    assert!(!os::path_exists(&p.push("version-0.4-file.txt")));

                }
                None => fail!("test_package_request_version: fetch_git failed")
            }
        }
        ExactRevision(n) => {
            fail!("n is %? and %? %s %?", n, n, if n == ~"0.3" { "==" } else { "!=" }, "0.3");
        }
        _ => fail!(fmt!("test_package_version: package version was %?, expected ExactRevision(0.3)",
                        temp_pkg_id.version))
    }
    let c = fake_ctxt(Some(@test_sysroot()));
    c.install(&temp, &temp_pkg_id);
    debug!("installed_library_in_workspace(%s, %s) = %?", temp_pkg_id.short_name, temp.to_str(),
           installed_library_in_workspace(temp_pkg_id.short_name, &temp));
    assert!(match installed_library_in_workspace(temp_pkg_id.short_name, &temp) {
        Some(p) => {
            debug!("installed: %s", p.to_str());
            p.to_str().ends_with(fmt!("0.3%s", os::consts::DLL_SUFFIX))
        }
        None    => false
    });
    assert!(target_executable_in_workspace(&temp_pkg_id, &temp)
            == temp.push("bin").push("test_pkg_version"));

}


// tests above should be converted to shell out (maybe?)

// CHECK EXIT CODES!

#[test]
fn rustpkg_install_url_2() {
    let temp_dir = mkdtemp(&os::tmpdir(), "rustpkg_install_url_2").expect("rustpkg_install_url_2");
    command_line_test("rustpkg", [~"install", ~"github.com/mozilla-servo/rust-http-client"],
                     &temp_dir);
}

#[test]
fn rustpkg_library_target() {
    let foo_repo = make_git_repo("foo");
    add_git_tag(&foo_repo, "1.0");
    command_line_test("rustpkg", [~"install", ~"foo"], &foo_repo);
    assert_lib_exists(&foo_repo, "foo");
}

#[test]
fn rustpkg_local_pkg() {
    let dir = create_local_package("foo");
    command_line_test("rustpkg", [~"install", ~"foo"], &dir);
}

#[test]
#[ignore (reason = "RUST_PATH not yet implemented -- #5682")]
fn rust_path_test() {
    let dir = mk_workspace(&Path("/home/more_rust"),
                           &normalize(RemotePath(Path("foo"))),
                           &NoVersion);
  //  command_line_test("RUST_PATH=/home/rust:/home/more_rust rustpkg install foo");
    command_line_test("rustpkg", [~"install", ~"foo"], &dir);
}

#[test]
#[ignore(reason = "Package database not yet implemented")]
fn install_remove() {
    let dir = mkdtemp(&os::tmpdir(), "install_remove").expect("install_remove");
    create_local_package_in("foo", &dir, &NoVersion);
    create_local_package_in("bar", &dir, &NoVersion);
    create_local_package_in("quux", &dir, &NoVersion);
    command_line_test("rustpkg", [~"install", ~"foo"], &dir);
    command_line_test("rustpkg", [~"install", ~"bar"], &dir);
    command_line_test("rustpkg", [~"install", ~"quux"], &dir);
    let list_output = command_line_test_output("rustpkg", [~"list"]);
    assert!(list_output.contains(&~"foo"));
    assert!(list_output.contains(&~"bar"));
    assert!(list_output.contains(&~"quux"));
    command_line_test("rustpkg", [~"remove", ~"foo"], &dir);
    let list_output = command_line_test_output("rustpkg", [~"list"]);
    assert!(!list_output.contains(&~"foo"));
    assert!(list_output.contains(&~"bar"));
    assert!(list_output.contains(&~"quux"));
}

#[test]
#[ignore(reason = "Workcache not yet implemented -- see #7075")]
fn no_rebuilding() {
    let workspace = create_local_package("foo");
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let p_id = PkgId::new("foo");
    let date = datestamp(&built_library_in_workspace(&p_id,
                                                    &workspace).expect("no_rebuilding"));
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let newdate = datestamp(&built_library_in_workspace(&p_id,
                                                       &workspace).expect("no_rebuilding (2)"));
    assert_eq!(date, newdate);
}

#[test]
#[ignore(reason = "Workcache not yet implemented -- see #7075")]
fn no_rebuilding_dep() {
    let workspace = create_local_package_with_dep("foo", "bar", &NoVersion);
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let bar_date = datestamp(&lib_output_file_name(&workspace,
                                                  ".rust",
                                                  "bar"));
    let foo_date = datestamp(&output_file_name(&workspace, "foo"));
    assert!(bar_date < foo_date);
}

#[test]
fn do_rebuild_dep_dates_change() {
    let workspace = create_local_package_with_dep("foo", "bar", &NoVersion);
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let bar_date = datestamp(&lib_output_file_name(&workspace, "build", "bar"));
    touch_source_file(&workspace, "bar");
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let new_bar_date = datestamp(&lib_output_file_name(&workspace, "build", "bar"));
    assert!(new_bar_date > bar_date);
}

#[test]
fn do_rebuild_dep_only_contents_change() {
    let workspace = create_local_package_with_dep("foo", "bar", &NoVersion);
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let bar_date = datestamp(&lib_output_file_name(&workspace, "build", "bar"));
    frob_source_file(&workspace, "bar");
// also have to tamper with the datestamp somehow
    command_line_test("rustpkg", [~"build", ~"foo"], &workspace);
    let new_bar_date = datestamp(&lib_output_file_name(&workspace, "build", "bar"));
    assert!(new_bar_date > bar_date);
}

#[test]
fn test_versions() {
    let workspace = create_local_package_with_version("foo", "0.1");
    create_local_package_with_version("foo", "0.2");
    command_line_test("rustpkg", [~"install", ~"foo#0.1"], &workspace);
    let output = run::process_output("rustpkg", [~"list"]);
    // make sure output includes versions
    assert!(!str::contains(str::from_bytes(output.output), "foo#0.2"));
}

#[test]
fn test_build_hooks() {
    let workspace = create_local_package_with_custom_build_hook("foo", "frob");
    command_line_test("rustpkg", [~"do", ~"foo", ~"frob"], &workspace);
}


#[test]
fn test_info() {
    let expected_info = ~"package foo"; // fill in
    let workspace = create_local_package("foo");
    let output = command_line_test("rustpkg", [~"info", ~"foo"], &workspace);
    assert_eq!(str::from_bytes(output.output), expected_info);
}

#[test]
fn test_rustpkg_test() {
    let expected_results = ~"1 out of 1 tests passed"; // fill in
    let workspace = create_local_package_with_test("foo");
    let output = command_line_test("rustpkg", [~"test", ~"foo"], &workspace);
    assert_eq!(str::from_bytes(output.output), expected_results);
}

#[test]
fn test_uninstall() {
    let workspace = create_local_package("foo");
    let _output = command_line_test("rustpkg", [~"info", ~"foo"], &workspace);
    command_line_test("rustpkg", [~"uninstall", ~"foo"], &workspace);
    let output = command_line_test("rustpkg", [~"list"], &workspace);
    assert!(!str::contains(str::from_bytes(output.output), "foo"));
}


/* To do: tests for prefer and unprefer */

