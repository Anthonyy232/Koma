//! `app.contract`'s shapes as Rust structs (a changed field fails to compile) and its packages' bindings.

fn main() {
    let path = std::path::Path::new("../app.contract");
    contract::rerun_if_changed(path);
    let (plan, call_records) =
        contract::compile_path_for_rust(path).unwrap_or_else(|e| panic!("app.contract:{e:?}"));
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let shapes = contract::rust(&plan, &call_records).expect("contract rust");
    std::fs::write(out.join("shapes.rs"), shapes).unwrap();
    let modules = contract::module_rust(&plan).expect("contract module_rust");
    std::fs::write(out.join("modules.rs"), modules).unwrap();
}
