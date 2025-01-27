# CrushScript

<img src="assets/logo.png">

CrushScript is an ECMAScript ahead-of-time compiler that is 450x faster than C or Rust
at a specific benchmark. You should never use this.

## How to use

CrushScript only runs on Windows. You need to have Microsoft Visual Studio installed
and run the "Developer Powershell for VS" script which imports all the paths and stuff
for the Visual Studio compiler.

To compile an ECMAScript file, run the following:

```shell
cargo run -- code.js --output code.exe
```
