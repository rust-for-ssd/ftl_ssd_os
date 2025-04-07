To compile:
RUSTFLAGS="--emit=obj" cargo nightly+ rustc -- --emit=obj --target target.json -Z build-std=core


Recompile types:
```bindgen headers/ssd_os.h -o src/ssd_os.rs --use-core --default-charset=signed```


Combine .o files from the .a file into a single .o file:
```riscv32-none-elf-ld -r -o final.o --whole-archive libssd_os_rust_demo.a --no-whole-archive```