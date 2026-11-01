//! Embeds the application icon in the Windows executable.

fn main() {
    println!("cargo:rerun-if-changed=mosaico-settings.rc");
    println!("cargo:rerun-if-changed=../../assets/mosaico.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("mosaico-settings.rc", embed_resource::NONE)
            .manifest_optional()
            .expect("failed to compile the icon resource");
    }
}
