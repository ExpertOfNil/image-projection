use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=res/*");
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = std::env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["res/"];
    copy_items(&paths_to_copy, out_dir, &copy_options)?;
    Ok(())
}
