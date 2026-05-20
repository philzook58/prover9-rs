use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let ladr_dir = manifest_dir.join("../Prover9/ladr");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/shim.c");
    println!("cargo:rerun-if-changed={}", ladr_dir.display());

    let mut sources = ladr_sources(&ladr_dir);
    sources.push(manifest_dir.join("src/shim.c"));

    let mut build = cc::Build::new();
    build
        .flag(format!("-iquote{}", ladr_dir.display()))
        .warnings(false);
    for source in &sources {
        build.file(source);
    }
    build.compile("prover9");

    let bindings = bindgen::Builder::default()
        .header(manifest_dir.join("wrapper.h").display().to_string())
        .clang_arg(format!("-iquote{}", ladr_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("apply")
        .allowlist_function("free_context")
        .allowlist_function("get_context")
        .allowlist_function("get_rigid_term")
        .allowlist_function("get_variable_term")
        .allowlist_function("assign_order_method")
        .allowlist_function("ground_term")
        .allowlist_function("kbo")
        .allowlist_function("lrpo")
        .allowlist_function("match")
        .allowlist_function("copy_clause")
        .allowlist_function("copy_literal")
        .allowlist_function("copy_literals")
        .allowlist_function("demod1")
        .allowlist_function("demodulator_type")
        .allowlist_function("orient_equalities")
        .allowlist_function("append_literal")
        .allowlist_function("fdemod_clause")
        .allowlist_function("idx_demodulator")
        .allowlist_function("mindex_destroy")
        .allowlist_function("mindex_init")
        .allowlist_function("mindex_update")
        .allowlist_function("resolve2")
        .allowlist_function("merge_literals")
        .allowlist_function("prover9_para_pos")
        .allowlist_function("subsumes")
        .allowlist_function("subsumes_bt")
        .allowlist_function("xx_resolve2")
        .allowlist_function("ith_literal")
        .allowlist_function("literal_to_term")
        .allowlist_function("literals_to_term")
        .allowlist_function("new_literal")
        .allowlist_function("neg_eq")
        .allowlist_function("neg_eq_unit")
        .allowlist_function("negative_literals")
        .allowlist_function("negative_clause")
        .allowlist_function("number_of_literals")
        .allowlist_function("occurs_in")
        .allowlist_function("parse_term_from_string")
        .allowlist_function("pos_eq")
        .allowlist_function("pos_eq_unit")
        .allowlist_function("positive_literals")
        .allowlist_function("positive_clause")
        .allowlist_function("mixed_clause")
        .allowlist_function("maximal_literal")
        .allowlist_function("maximal_signed_literal")
        .allowlist_function("number_of_maximal_literals")
        .allowlist_function("unit_clause")
        .allowlist_function("horn_clause")
        .allowlist_function("ground_clause")
        .allowlist_function("clause_symbol_count")
        .allowlist_function("clause_depth")
        .allowlist_function("contains_pos_eq")
        .allowlist_function("contains_eq")
        .allowlist_function("only_eq")
        .allowlist_function("symbol_count")
        .allowlist_function("destroy_discrim_tree")
        .allowlist_function("discrim_init")
        .allowlist_function("discrim_wild_cancel")
        .allowlist_function("discrim_wild_retrieve_first")
        .allowlist_function("discrim_wild_retrieve_next")
        .allowlist_function("discrim_wild_update")
        .allowlist_function("renumber_variables")
        .allowlist_function("set_lex_val")
        .allowlist_function("str_to_sn")
        .allowlist_function("term_depth")
        .allowlist_function("term_greater")
        .allowlist_function("term_to_clause")
        .allowlist_function("term_to_string")
        .allowlist_function("term_order")
        .allowlist_function("topform_to_term_without_attributes")
        .allowlist_function("tautology")
        .allowlist_function("unify")
        .allowlist_function("undo_subst")
        .allowlist_function("zap_ilist")
        .allowlist_function("zap_literal")
        .allowlist_function("zap_literals")
        .allowlist_function("zap_topform")
        .allowlist_function("zap_term")
        .allowlist_function("prover9_.*")
        .allowlist_type("context")
        .allowlist_type("discrim")
        .allowlist_type("discrim_pos")
        .allowlist_type("Indexop")
        .allowlist_type("literals")
        .allowlist_type("mindex")
        .allowlist_type("Order_method")
        .allowlist_type("Ordertype")
        .allowlist_type("term")
        .allowlist_type("topform")
        .allowlist_type("trail")
        .generate()
        .expect("failed to generate prover9 bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("failed to write bindings");
}

fn ladr_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("c") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("test.c") {
            continue;
        }
        files.push(path);
    }
    files.sort();
    files
}
