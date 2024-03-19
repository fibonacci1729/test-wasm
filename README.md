# `test-wasm`

VERY VERY WIP.

This repository contains a POC demonstrating how one might write `rust` unit tests that execute in WebAssembly.

Each function of type `func()` attributed with the `test_wasm` macro synthesizes a component `export` where the export name is prefixed with `test-`. A simple runner is included that searches the list of component exports for any exported function prefixed with `test-` and attempts to execute them.