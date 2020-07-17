## Immediate Todo
1. Fix background
    * Also fix how it dosen't leave black bars and makes weird placing issues
2. Make the video recorder record the glow effect (capture the screen)

1. Visualize axis lock controls, particle placement, paused gameplay, video recording start/finish, etc
2. FPS display
3. GUI
4. Save/Load simulations with files
5. MISX package

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Fix Electricity getting stuck with 1 particle of water in mid-air
3. Make Fire not spread so fast when starting from the top/left

4. Remove Cargo.toml hacks from flatpak build when the rust sdk extension updates
5. Add back support for wayland when less buggy (missing virtual keycodes, slow input events infinitely triggering, buggy fullscreen toggling on gnome, buggy decorations)
6. Replace heap_array.rs with 'placement new' when it gets added to rust
7. WASM

## Release Procedure
1. Bump Cargo.toml version
2. Update CHANGELOG.md
3. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
4. Update flatpak/cargo-sources.json
5. Update flatpak/screenshot.png if needed
6. Tag git commit
7. Update flathub repo
