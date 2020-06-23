## Immediate Todo
1. Improve interactions between existing particle types and tempature
2. Code cleanup
    * Move Particle and ParticleType into a new file
    * Localize all particle properties as methods on ParticleType
    * Use consts or something to identify extra_data per particle type
    * Implement indexing on Sandbox, to not have to go through cells
3. Scale video recording by 4

## Later Todo
1. Visualize axis lock controls, particle placement, paused gameplay, etc
2. Save/Load simulations with files
3. GUI
4. App icon and package

## Eventual Todo
1. Make particles move left/right randomly when they can do either
2. Figure out why axis lock sometimes dosen't place enough particles to make a line
3. Fix Electricity getting stuck with 1 particle of water in mid-air
4. Make Fire not spread so fast when starting from the top/left
