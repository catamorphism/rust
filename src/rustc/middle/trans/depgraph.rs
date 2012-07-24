/*
Computes item dependency graph
*/

import syntax::ast::*;
import syntax::{visit, ast_util, ast_map};
import syntax::ast_util::{def_id_of_def, new_def_hash};
import syntax::attr;
import syntax::visit::*;
import syntax::print::pprust::{expr_to_str, path_to_str};
import std::map::{hashmap, int_hash};
import driver::session::*;
import middle::ty::*;

export find_deps;

// this is pretty sloppy, would rather just use one map
type seen_deps = std::map::hashmap<def_id, ()>;

type dependency = {source_id: def_id, // Item being depended on
                   target_id: node_id, // Local item that depends on it
                   depender_str: ident,
                   dependee_str: ~str};

// item A (which is always local) depends on item B
type adj_list = ~[dependency];

type ctxt1 = {ast_map: ast_map::map,
            exp_map: resolve::exp_map,
            tcx: ty::ctxt,
            method_map: typeck::method_map,
            // "A depends on B"
            mut graph_nodes: adj_list
             };

type ctx = {// item we're in
            current_item: node_id,
            current_item_name: ident,
    // cache to avoid adding duplicate entries to the vec
            seen_deps: @seen_deps,
            x: @ctxt1
};

fn find_deps(crate: @crate, ast_map: ast_map::map,
             exp_map: resolve::exp_map,
             tcx: ty::ctxt, method_map: typeck::method_map) {
    #debug("find_deps!");
    let cx: @ctxt1 = @{ast_map: ast_map,
              exp_map: exp_map, tcx: tcx, method_map: method_map,
              mut graph_nodes: ~[]};
    #debug("visit_tys");
    visit_tys(cx, crate);
    #debug("print_deps");
    print_deps(cx);
}

fn print_deps(-cx: @ctxt1) {
    // tjc: fix file name
    let opt_w = io::buffered_file_writer(#fmt("deps_%u.dot",
                                              cx.graph_nodes.len()));
    alt opt_w {
      result::err(e) { fail e; }
      result::ok(w) {
        do str::byte_slice("digraph G{\n") |s| {w.write(s)};
        for (copy cx.graph_nodes).each |d| {
            do str::byte_slice(#fmt("%s -> %s;\n", *d.depender_str,
                                    d.dependee_str)) |s| { w.write(s) };
        };
        do str::byte_slice("\n}") |s| {w.write(s)};
      }
    }
}

fn visit_tys(cx: @ctxt1, crate: @crate) {
    #debug("visit_tys!");
    let vtor: vt<@ctxt1> = visit::mk_vt(@{visit_item:
      fn@(it: @item, &&cx: @ctxt1, &&vt: vt<@ctxt1>) {
        alt it.node {
          // Don't care about modules
          item_mod(m) {
            vt.visit_mod(m, it.span, it.id, cx, vt);
            ret;
          }
          _           {}
        }

         let vtor: vt<ctx> = visit::mk_vt(@{visit_ty:
 // This is probably going to be terrible,
 // b/c it doesn't consider inferred tys.
 fn@(t: @ast::ty, &&cx: ctx, &&vt: vt<ctx>) {
    visit::visit_ty(t, cx, vt);
    alt t.node {
      ty_path(p, n_id) {
        alt cx.x.tcx.def_map.find(n_id) {
          some(d) {
            alt d {
              def_ty(d_id) | def_class(d_id) {
                let path = path_to_str_no_params(p);
                record_dep(cx, d_id, path);
              }
              _ { /* Not a type, or a param or other type we don't care
                     about */
             }
            }
          }
          none {
            fail ~"poo"; // Shouldn't happen
          }
        }
      }
      _ { /* Any other types: either are visited recursively,
      or we don't care */
      }
    }} with *visit::default_visitor::<ctx>()});
        visit::visit_item(it, {current_item: it.id,
                               current_item_name: it.ident,
                               seen_deps: @new_def_hash(),
                               x: cx}, vtor);
    }
            with *visit::default_visitor::<@ctxt1>()});
    visit::visit_crate(*crate, cx, vtor);
}

fn record_dep(cx: ctx, d_id: def_id, dstr: ~str) {
  //  #debug("Recording a dep: %s -> %s", *cx.current_item_name, dstr);
  //  #debug(">>>> %u", cx.x.graph_nodes.len());
    // need to avoid duplicates
    alt cx.seen_deps.find(d_id) {
      none {
        cx.seen_deps.insert(d_id, ());
      }
      some(_) {
        ret; // Already in cache
      }
    }
    vec::push(cx.x.graph_nodes, {source_id: d_id, target_id: cx.current_item,
                                            depender_str:
                              cx.current_item_name, dependee_str: dstr});
  //  #debug(">>>> %u", cx.x.graph_nodes.len());
}

fn path_to_str_no_params(path: @path) -> ~str {
    let mut rs = ~"";
    let mut first = true;
    for path.idents.each |id| {
        // tjc: not underscores
        if first { first = false; } else { rs += ~"_"; }
        rs += *id;
    }
    // hell
    if rs == ~"node" {
        ~"a_node"
    }
    else {
        rs
    }
}

/*
not really

fn walk_ty(cx: ctx, t: ty::t) {
    // Handle any sub-components
    visit::visit_ty(cx, t);
    alt ty::get(t).struct {
      ty_nil | ty_bot | ty_bool | ty_int(_) | ty_uint(_) | ty_float(_) |
      ty_str | ty_estr(_) | ty_self { } // No deps
      ty_param(*) { } // No deps
      ty_enum(d_id, _) | ty_trait(d_id, _) | ty_class(d_id, _) {
        record_dep(cx, d_id);
      }
      _ { /* Visited recursively */ }
    }
}
*/