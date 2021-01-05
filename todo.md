## Known Bugs
* FPS is displayed as slightly higher than it should be (e.g. 65 fps)
* Flatpak builds are missing window title text
* The UI bounding box extends a bit too far to the right
* Electricity gets stuck with 1 particle of water in mid-air
* Particles should move left/right randomly when they can do either

## Todo
* Use Rayon for tempature changes

* Replace simdnoise and flume with a shader
* Switch to iced for UI
    * Tooltips on buttons for hotkeys
* Save/Load simulations as images
    * Scale image to sandbox size, quantitize, match to particles

* Physics?
    * It's been suggested to group all connected particles together
    * Inside the group, each particle performs celluar automata movement
    * Physics are run with each group acting was one body
* Convection through empty cells?
* MISX package
* WASM build
* Replace heap_array.rs when placement new is stabilized

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
