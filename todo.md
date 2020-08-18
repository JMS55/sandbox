## Immediate Todo
1. Fix UI bounding box extending a bit too far to the right

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Replace simdnoise and flume with a shader
3. Fix 2 Electricity getting stuck with 1 particle of water in mid-air
---
4. Update flatpak to 20.08 and remove Cargo.toml hacks
5. Add back support for wayland when less buggy
6. Replace heap_array.rs with 'placement new' when it gets added to rust
---
7. Save/Load simulations with files
8. MISX package
9. Mobile linux
10. WASM

## Release Procedure
1. Bump Cargo.toml version
2. Update CHANGELOG.md
3. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
4. Update flatpak/cargo-sources.json
5. Update flatpak/screenshot.png if needed
6. Tag git commit
7. Update flathub repo
