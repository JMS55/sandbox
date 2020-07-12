## Immediate Todo
1. Visualize axis lock controls, particle placement, paused gameplay, video recording start/finish, etc
2. FPS display
3. GUI
4. Save/Load simulations with files
5. MISX package

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Add back support for wayland when less buggy (missing virtual keycodes, slow input events infinitely triggering, buggy fullscreen toggling on gnome, buggy decorations)
3. Video recording file has the wrong framerate
4. Tweak x264enc settings
5. Hardware accelerated h264 encoding, with x264enc fallback
6. Enable video recording for the flatpak build
7. Remove Cargo.toml hacks from flatpak build when the rust sdk extension updates
8. Make Fire not spread so fast when starting from the top/left
9. Fix Electricity getting stuck with 1 particle of water in mid-air
10. Replace heap_array.rs with 'placement new' when it gets added to rust
11. WASM

## Release Procedure
1. Bump Cargo.toml version
2. Update CHANGELOG.md
3. Add another release element to flatpak/com.github.jms55.sandbox.metainfo
4. Update flatpak/cargo-sources.json
5. Fix description in flatpak/com.github.jms55.sandbox.metainfo to match README.md if needed
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
