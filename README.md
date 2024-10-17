# RPS2 SDK
## Overview
Welcome to the RPS2 SDK home! 

This repo contains the main SDK for the RPS2 Project (Rust for PlayStation 2).

This currently includes:
- Minimal bindings to the kernel via `rps2-kernel`.
- Startup code and linker scripts via `rps2-startup`.
- Full unwinding support via `rps2-panic`.
- A simple testing harness via `rps2-libtest`.
- A rusty threading interface and utilities via `rps2-thread`.
- And all tied together with `rps2`, offering an environment similar to `std`.
- A (currently not so)-full unit tests of `rps2`.

This repo also contains some usage examples under `samples/`.