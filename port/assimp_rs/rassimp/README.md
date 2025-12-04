# Open Asset Import Library - Rust ports (rassimp)

[![Crates.io](https://img.shields.io/crates/v/assimp.svg)](https://crates.io/crates/assimp)
[![Docs.rs](https://docs.rs/assimp/badge.svg)](https://docs.rs/assimp)
[![Build Status](https://github.com/jkvargas/assimp-rs/workflows/Rust/badge.svg)](https://github.com/jkvargas/assimp-rs/actions)

**rassimp** provides Rust ports for the official [Open Asset Import Library (Assimp)](https://github.com/assimp/assimp), 
supporting import of __40+ 3D file formats__ with in-memory model conversion. Implemented via FFI to the native C++ library.

### Core Features
- Cross-platform support (Windows/Linux/macOS)
- Full scene graph import (meshes/materials/animations/bones)
- Built-in mesh processing tools:
  - Triangulation
  - Normal/tangent generation
  - Vertex cache optimization
  - Degenerate primitive removal

### Quick Start
Add to your `Cargo.toml`:
```toml
[dependencies]
rassimp = "0.1"
```

### Supported Formats
See the [full list in official docs](https://github.com/assimp/assimp/blob/master/doc/Fileformats.md). Includes:
- X

### Building
Requires the native [Assimp library](https://github.com/assimp/assimp):
```bash
# Install system dependencies
sudo apt install libassimp-dev  # Ubuntu/Debian
brew install assimp             # macOS

# Build Rust bindings
cargo build
```

### Documentation
- [API Documentation](https://docs.rs/assimp)
- [Example Code](https://github.com/jkvargas/assimp-rs/tree/master/examples)
- [Original Library Docs](https://assimp-docs.readthedocs.io)

### Contributing
PRs are welcome! Please ensure:
1. All tests pass with `cargo test`
2. Update [examples](https://github.com/jkvargas/assimp-rs/tree/master/examples)
3. Follow Rust API design conventions

### License
[BSD-3-Clause](LICENSE) - same as the original library