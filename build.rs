// build.rs — Embeds the UAC manifest so Windows prompts for Admin elevation.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_manifest_file("manifest.xml");
        // Optional: embed icon (uncomment and provide the file)
        // res.set_icon("assets/icon.ico");
        res.compile().expect("Failed to compile Windows resources");
    }
}
