## Immediate Todo
1. Code cleanup
    * Move Particle and ParticleType into a new file
    * Localize all particle properties as methods on ParticleType (or Particle?)
    * Use consts or something to identify extra_data per particle type
    * Implement indexing on Sandbox, to not have to go through cells
    * Make helper methods for sandbox.cells[x][y].as_mut().unwrap() and maybe &sandbox.cells[x][y].unwrap()
2. Scale video recording by 4

## Later Todo
1. Visualize axis lock controls, particle placement, paused gameplay, video recording start/finish, etc
2. GUI
3. Save/Load simulations with files
4. Bump app version to 1.0
5. App icon and package

## Eventual Todo
1. Make particles move left/right randomly when they can do either
3. Fix Electricity getting stuck with 1 particle of water in mid-air
4. Make Fire not spread so fast when starting from the top/left
5. Replace heap_array.rs with 'placement new' when it gets added to rust
