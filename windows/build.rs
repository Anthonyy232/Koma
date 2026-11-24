use std::path::{Path, PathBuf};

fn main() {
    // The icon, and the manifest that makes the process see physical pixels.
    embed_resource::compile("koma.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
    let app = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    contract::rerun_if_changed(&app.join("app.contract"));
    println!("cargo:rerun-if-changed=../app.json");
    println!("cargo:rerun-if-changed=../app.forma");
    let plan = contract::compile_path(&app.join("app.contract"))
        .unwrap_or_else(|e| panic!("app.contract:{e}"));
    let baked =
        contract::bake(plan, koma_data::Koma::default()).unwrap_or_else(|e| panic!("bake: {e:?}"));
    let plan = baked.encode();
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let target = std::env::var("TARGET").unwrap();
    contract::native::write_desktop_packages("windows", &app, &plan, &target, &out)
        .unwrap_or_else(|e| panic!("packages: {e}"));
    std::fs::write(out.join("app.plan"), plan).unwrap();

    let manifest = contract::Manifest::read(&app).unwrap_or_else(|e| panic!("app.json: {e}"));
    let entries = contract::snapshot::entries(&app).unwrap();
    let compat = exact_bake::compatibility_id_sources_with_entries(
        &app,
        "windows",
        &target,
        &manifest,
        Some(koma_data::GRANTS),
        None,
        Some(&entries),
        exact_bake::hands_out_work!(koma_data::Koma::default()),
    )
    .unwrap_or_else(|e| panic!("compatibility id: {e}"));
    std::fs::write(out.join("compat.json"), compat.to_json()).unwrap();
    let shell = compat
        .windows_shell(&baked, &manifest)
        .unwrap_or_else(|e| panic!("app.json: {e}"));
    std::fs::write(out.join("shell.rs"), shell).unwrap();
}
