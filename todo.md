## Immediate Todo
0. Investigate windows builds:
    1. Make sure it builds, and that WaylandCSDTheme dosen't prevent that
    2. Stack overflow on cloning particle array?
1. Investigate 65 fps, and not 60 being the max (use release builds)
    * Probably should clamp fps to refresh rate
2. Fix the flatpak build missing a window title
3. UI cleanup
    1. Fix UI bounding box extending a bit too far to the right
    2. Update ui.rs to use ptype_ui color functions
    3. Split up render() into multiple functions
4. Save/Load simulations with files
5. MISX package

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Replace simdnoise and flume with a shader
3. Fix 2 Electricity getting stuck with 1 particle of water in mid-air
4. WASM
---
5. Replace heap_array.rs with 'placement new' when it gets added to rust
6. Mobile linux
7. Sound effects

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
