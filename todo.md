## Immediate Todo
1. Investigate 65 fps, and not 60 being the max (use release builds)
2. Fix the flatpak build missing a window title
3. Fix UI bounding box extending a bit too far to the right
4. Update ui.rs to use ptype_ui color functions

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Replace simdnoise and flume with a shader
3. Fix 2 Electricity getting stuck with 1 particle of water in mid-air
---
4. Replace heap_array.rs with 'placement new' when it gets added to rust
5. Save/Load simulations with files
6. MISX package
7. Mobile linux
8. WASM
9. Sound effects

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
