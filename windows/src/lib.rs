//! Koma's Windows host: the window and Exact's Windows packages around `koma_data`.

const PLAN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app.plan"));
const COMPAT: &str = include_str!(concat!(env!("OUT_DIR"), "/compat.json"));
// From the bake: `ADMIT_AGENT` (Exact's driver, development bakes only), `SERVICES` (no network in
// production) and `window()` (`app.json`).
include!(concat!(env!("OUT_DIR"), "/shell.rs"));
// The Windows side of the Exact packages `app.contract` uses (`exact_packages()`).
include!(concat!(env!("OUT_DIR"), "/packages.rs"));

pub fn run() -> i32 {
    let window = exact_windows::Window {
        overlay: true,
        ..window()
    };
    let composed = exact_painted::Composed {
        packages: exact_packages(),
        ..Default::default()
    };
    // Carries out the source's file-system requests.
    type Source = exact_data_host::Storage<koma_data::Koma>;
    exact_windows::run_baked::<Source, ADMIT_AGENT>(
        "Koma", window, PLAN, COMPAT, composed, SERVICES,
    )
}
