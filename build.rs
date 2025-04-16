use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let libclang_include =
        env::var("LIBCLANG_PATH").expect("LIBCLANG_PATH environment variable not set");

    // Create output directory if it doesn't exist
    let out_dir = Path::new("src/bindings/generated");
    fs::create_dir_all(out_dir).expect("Failed to create output directory");

    for entry in fs::read_dir("./headers").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().map_or(false, |ext| ext == "h") {
            println!("cargo:rerun-if-changed={}", path.display());

            // Get filename without extension
            let file_stem = path
                .file_stem()
                .unwrap_or_else(|| panic!("Failed to get file stem for {:?}", path))
                .to_str()
                .unwrap_or_else(|| panic!("Invalid UTF-8 in filename {:?}", path));

            let out_path = out_dir.join(format!("{}.rs", file_stem));

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
                .generate()
                .expect(&format!("Failed to generate bindings for {:?}", path));

            bindings
                .write_to_file(&out_path)
                .expect(&format!("Failed to write bindings to {:?}", out_path));
        }
    }
}
