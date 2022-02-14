## Known Bugs
* Temperature calcs can overflow
* Freezes when tabbing out and then coming back to the game as it suddenly tries to do 1000 updates at once
* Using shift/ctrl modifiers and moving the mouse really fast can leave gaps in particle placement
* Electricity gets stuck with 1 particle of water in mid-air
* Particles should move left/right randomly when they can do either
* The UI bounding box extends a bit too far to the right

## Todo
* Visual for shift/ctrl modifiers
* Replace imgui for UI
* Replace noise with shaders
* WASM build
* Improve glitch graphics
* Save/Load simulations as images
* Document particle state / replace magic numbers with constants

* GPU Compute based updates?
* Tempature transfer through empty cells?
* Physics?
    * It's been suggested to group all connected particles together
    * Inside the group, each particle performs celluar automata movement
    * Physics are run with each group acting was one body

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
