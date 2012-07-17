/*
Computes item dependency graph
*/

import syntax::ast::*;
import syntax::{visit, ast_util, ast_map};
import syntax::ast_util::def_id_of_def;
import syntax::attr;
import syntax::visit::*;
import syntax::print::pprust::{expr_to_str, path_to_str};
import std::map::hashmap;
import driver::session::*;
import middle::ty::*;

export find_deps;

type map = std::map::hashmap<def_id, ()>;

type dependency = {source_id: def_id, // Item being depended on
                   target_id: node_id, // Local item that depends on it
                   depender_str: ident,
                   dependee_str: str};

// item A (which is always local) depends on item B
type adj_list = ~[dependency];

type ctxt1 = {ast_map: ast_map::map,
            exp_map: resolve::exp_map,
            tcx: ty::ctxt,
            method_map: typeck::method_map,
            // "A depends on B"
            mut graph_nodes: adj_list};

type ctx = {// item we're in
            current_item: node_id,
            current_item_name: ident,
            x: ctxt1
};

fn find_deps(crate: @crate, ast_map: ast_map::map,
             exp_map: resolve::exp_map,
             tcx: ty::ctxt, method_map: typeck::method_map) {
    let cx: ctxt1 = {ast_map: ast_map,
              exp_map: exp_map, tcx: tcx, method_map: method_map,
              mut graph_nodes: ~[]};
    visit_tys(cx, crate);
    print_deps(cx, cx.graph_nodes);
}

fn print_deps(_cx: ctxt1, g: adj_list) {
    for g.each |d| {
        // Not printing: ???
        log(error, #fmt("%s -> %s", *d.depender_str, d.dependee_str));
    };
}

fn visit_tys(cx: ctxt1, crate: @crate) {
    let vtor: vt<ctxt1> = visit::mk_vt(@{visit_item:
      fn@(it: @item, &&cx: ctxt1, &&_vt: vt<ctxt1>) {
         let vtor: vt<ctx> = visit::mk_vt(@{visit_ty:
 fn@(t: @ast::ty, &&cx: ctx, &&vt: vt<ctx>) {
    visit::visit_ty(t, cx, vt);
    alt t.node {
      ty_path(p, n_id) {
        alt cx.x.tcx.def_map.find(n_id) {
          some(d) {
            alt d {
              def_ty(d_id) | def_class(d_id) {
                let path = path_to_str(p);
                record_dep(cx, d_id, path);
              }
              _ { /* Not a type, or a param or other type we don't care
                     about */
             }
            }
          }
          none {
            fail "poo"; // Shouldn't happen
          }
        }
      }
      _ { /* Any other types: either are visited recursively,
      or we don't care */
      }
    }} with *visit::default_visitor::<ctx>()});
        visit::visit_item(it, {current_item: it.id,
                               current_item_name: it.ident,
                               x: cx}, vtor);
    }
            with *visit::default_visitor::<ctxt1>()});
    visit::visit_crate(*crate, cx, vtor);
}

fn record_dep(cx: ctx, d_id: def_id, dstr: str) {
    vec::push(cx.x.graph_nodes, {source_id: d_id, target_id: cx.current_item,
                                            depender_str:
                              cx.current_item_name, dependee_str: dstr});
}

/*
fuck

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