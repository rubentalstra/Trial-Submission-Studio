# Documentation Developer Commands

To build and serve the documentation locally, use the following commands:

before running these commands, ensure you have `mdbook` installed. If you don't have it installed, you can do so using
Cargo:

```bash
cargo install mdbook

cargo install mdbook-mermaid

cargo install mdbook-linkcheck2
```

```bash
mdbook build docs

mdbook serve docs --open 
```

This will compile the documentation and open it in your default web browser for easy navigation and reading.
