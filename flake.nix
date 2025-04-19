{
  description = "A devShell example";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rv32_pkgs = nixpkgs.legacyPackages.${system}.pkgsCross.riscv32-embedded;
        rv64_pkgs = nixpkgs.legacyPackages.${system}.pkgsCross.riscv64-embedded;
        rust_nightly = pkgs.rust-bin.selectLatestNightlyWith (toolchain:
          toolchain.default.override {
            extensions = [ "rust-src" "rust-analyzer" ];
            targets = [ "riscv32imac-unknown-none-elf" ];
             });
      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = with pkgs; [
              rust_nightly
              rv32_pkgs.buildPackages.gdb
              rv32_pkgs.buildPackages.gcc
              rv32_pkgs.buildPackages.binutils
              rust-bindgen
              llvmPackages.clang
              qemu
              bacon
            ];

            env = {
              RISCV_RT_LLVM_ARCH_PATCH="riscv32imac-unknown-none-elf"; # Needed by riscv-rt
              RISCV_RT_BASE_ISA="rv32i"; # Needed by riscv-rt
              RUST_GDB =
                "${rv32_pkgs.buildPackages.gdb}/bin/riscv32-none-elf-gdb";
              LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
              GLIBC_PATH = "${rv32_pkgs.binutils.libc}";
            };
          };
      });
}
