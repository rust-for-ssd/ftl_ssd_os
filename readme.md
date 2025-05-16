# A Flash Translation Layer application for `ssd_os` in Rust

This repository contains code for the MSc Thesis by Jens Birk Andersen and Malthe Mathias Mølgaard Larsen titled "Exploring Rust as an embedded language
for programmable SSDs using non-standard
ISA RISC-V softcores on FPGAs" written in Spring 2025.

# Structure
The repository contains the following applications located in `src/apps`:
- Round Trip.
- Round Trip (C).
- FTL Design 1: A Connector per Component.
- FTL Design 2: A Pipeline per Command Type.
- FTL Design 3: A Distributed L2P Table.

# Development setup
The repository contains a `flake.nix` file that includes all dependencies. We recommend to install `nix` and `direnv` on your machine to utilize the flake. The flake has been tested to work on NixOS stable 24.11 (kernel 6.6.78) and MacOS 15.1. Make sure that `ssd\_os` is cloned into the same directory level as `ftl_ssd_os`:
```
|---ssd_os
|---ftl_ssd_os
|
```

# Building an application
To build an application, make sure all dependencies are correctly installed and run on of the folowing commands from the project root:
- `make round_trip`
- `make round_trip_c`
- `make connector_per_compoennt`
- `make piepline_per_cmd`
- `make distributed_l2p`

# Running an application
Follow the instructions in `ssd_os` found [here](https://github.com/OpenSSD-V/ssd_os). For running it on a Mac, we have provided a Dockerfile found [here]()


# Running test
To run unit tests of the components found in `src/` run `cargo test` from the project root.

---

> **Disclaimer:** the code in this repository has run on ssd_os at commit 82a758466bc7c6a210874c49dd75fd2ef620efb1. It has not been updated to newer versions of the platform.
