use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use bindgen::Formatter;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("memory.x"), include_bytes!("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=build.rs");

    let libclang_include =
        env::var("LIBCLANG_PATH").expect("LIBCLANG_PATH environment variable not set");

    // Create output directory if it doesn't exist
    let out_dir = Path::new("src/bindings");
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    // for entry in fs::read_dir("./headers").unwrap() {
    let path = Path::new("headers/wrapper.h");
    if path.extension().map_or(false, |ext| ext == "h") {
        println!("cargo:rerun-if-changed={}", path.display());

        let out_path = out_dir.join("generated.rs");

        let bindings = bindgen::Builder::default()
            .header(path.to_str().unwrap())
            .use_core()
            .generate_cstr(true)
            .merge_extern_blocks(true)
            .layout_tests(false)
            .clang_args(&[
                "--target=riscv32-unknown-none",
                &format!("-I{}", libclang_include),
            ])
            .formatter(Formatter::Prettyplease)
            .generate()
            .expect(&format!("Failed to generate bindings for {:?}", path));

        bindings
            .write_to_file(&out_path)
            .expect(&format!("Failed to write bindings to {:?}", out_path));
    }
    // }
}
