use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
  tauri_build::build();

  let out_dir = env::var("OUT_DIR").unwrap();
  let llvm_ar = "C:/Users/Ronn/.rustup/toolchains/stable-x86_64-pc-windows-gnu/lib/rustlib/x86_64-pc-windows-gnu/bin/llvm-ar.exe";

  let resource_a = PathBuf::from(&out_dir).join("libresource.a");
  if resource_a.exists() {
    // Index the archive so the default -lresource doesn't throw "no index"
    let _ = Command::new(llvm_ar).args(&["s", resource_a.to_str().unwrap()]).status();

    // Extract rsrc.o and pass directly as object file to preserve .rsrc section from --gc-sections
    let _ = Command::new(llvm_ar)
      .current_dir(&out_dir)
      .args(&["x", "libresource.a"])
      .status();

    let rsrc_o = PathBuf::from(&out_dir).join("rsrc.o");
    if rsrc_o.exists() {
      println!("cargo:rustc-link-arg={}", rsrc_o.to_str().unwrap());
    }
  }
}
