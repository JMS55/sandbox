## Known Bugs
* Freezes when tabbing out and then coming back to the game as it suddenly tries to do 1000 updates at once
* Using shift/ctrl modifiers and moving the mouse really fast can leave gaps in particle placement
* Particles should move left/right randomly when they can do either
* Electricity gets stuck with 1 particle of water in mid-air
* The UI bounding box extends a bit too far to the right

## Todo
* Move most of main.rs into game.rs
* Complex sprites?
    * Draw whatever, use that as indicator for drawing something else on top via shader
* Replace noise with shaders
    1. Generate a (4 * SANDBOX_WIDTH)xSANDBOX_HEIGHT noise texture and save it as an image
    2. Sandbox::new() loads the texture
    3. Sandbox::render() writes each particle's shimmer_intensity() to a texture
    4. Sample the noise texture, multiply by previous texture, and add to PixelsContext::texture() before Pixels::ScalingRenderer is used
    5. Delete simdnoise and flume
* Switch to egui/iced for UI
    * Tooltips on buttons for hotkeys
* Rework Glitch
    * Replace Glitch color with chromatic aberration
    * Maybe blur the non-glitch particles/area around it?

* Document particle state / replace magic numbers with constants
* Save/Load simulations as images
    * Scale image to sandbox size, quantitize, match to particles
* WASM build

* Windows package
* Physics?
    * It's been suggested to group all connected particles together
    * Inside the group, each particle performs celluar automata movement
    * Physics are run with each group acting was one body
* Convection through empty cells?

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
