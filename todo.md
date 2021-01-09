## Known Bugs
* Flatpak builds are missing window title text
* The UI bounding box extends a bit too far to the right
* Electricity gets stuck with 1 particle of water in mid-air
* Particles should move left/right randomly when they can do either
* Using shift/ctrl modifiers and moving the mouse really fast can leave gaps in particle placement
* The background texture should be slightly shifted over so that it's symmetrical on all sides
* Freezes when tabbing out and then coming back to the game as it suddenly tries to do 1000 updates at once

## Todo
* Parallelize simulation updates
    * Reuse chunk Box allocations/allocate all in a Vec and reuse that
    * Adapt all particle scripts
    * Figure out how to handle RNG
        * Maybe have Game or Sandbox hold an RNG, that's used to seed an RNG per chunk?
    * Fix chunk boundry artifacts
    * Empty chunk optimization?
    * Parallel tempature updates?
    * Lots of cleanup needed, rethink API and struct hierachy
    * Check that axis iteration order and array layout is optimal

* Replace Glitch color with chromatic aberration
* Switch to iced for UI
    * Tooltips on buttons for hotkeys
* Save/Load simulations as images
    * Scale image to sandbox size, quantitize, match to particles

* Replace simdnoise and flume with a shader?
* Physics?
    * It's been suggested to group all connected particles together
    * Inside the group, each particle performs celluar automata movement
    * Physics are run with each group acting was one body
* Convection through empty cells?
* MISX package
* WASM build
* Replace cell_grid.rs when placement new and const non-copy array initializers are stabilized

## Release Procedure
1. Bump Cargo.toml version
2. Run cargo update
3. Update CHANGELOG.md
4. Add another release element to flatpak/com.github.jms55.Sandbox.metainfo
5. Update flatpak/cargo-sources.json
6. Update flatpak/screenshot.png if needed
7. Tag git commit
8. Update flathub repo
