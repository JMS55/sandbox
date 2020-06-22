## Immediate Todo
1. More particle types
    * Add Metal (heat blood), Coal, and Steel
2. Add some kind of background (maybe cache noise from a few frames ago, and use that with a different scaling?)
3. Code cleanup
    * Move Particle and ParticleType into a new file
    * Localize all particle properties as methods on ParticleType
    * Use consts or something to identify extra_data per particle type
    * Implement indexing on Sandbox, to not have to go through cells

## Later Todo
1. Make particles move left/right randomly when they can do either
2. Figure out why axis lock sometimes dosen't place enough particles to make a line
3. Fix Electricity getting stuck with 1 particle of water in mid-air

## Eventual Todo
1. Animate Unstable explosion, maybe leave a particle/visual behind?
2. Visualize axis lock controls, particle placement, paused gameplay, etc
3. Save/Load simulations with files
4. GUI
5. Plan out gameplay progression. Start with a limited set of elements, create new ones to unlock them
6. App icon and package
