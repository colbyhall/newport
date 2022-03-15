# Project: Pacific
Velocity based 2d platformer puzzle. Use custom tools and gadgets in the world to traverse the world to keep moving forward.
## Table Of Contents
* [Gameplay](#gameplay)
* [Technology](#technology)
## Gameplay
### Themes
* Overcomming a challenge (puzzle)
* Unique and fullfilling exploration
* Incremental learning with reflection
### Features
* Reactive 2d Platforming
    * Walking
    * Jumping
    * Dashing? - Grappling Hook achieves this same goal
* Grappling Hook
    * Throw a hook in a single direction and pulls character in that direction
* Moving Platforms
    * Characters can gain velocity by standing on them or using the grappling hook on them
* Conveyor Belts
    * Extremely fast moving treadmills for velocity gaining
* Areas
    * Localized regionsof the game that have a beginning, middle checkpoints, and an end
    * Getting to the end moves to the next area
   
### Unknowns
* Grid based?
* low res?
## Technology
### Game Language
I'm looking for something to make a prototype in. This allows me to use something new and experimental. I want to try to expand my understanding of different modes of thought. 
- [ ] C++
    - Pros
        - Everyone uses it (Unreal Engine, Unity, AAA Game Developers)
        - Plenty of libraries (Graphics, Physics, Networking, Font)
        - Ownership Semantics (RAII, Move, Copy)
    - Cons
        - Preprocessor (`#include`, `#define`, `#ifdef`)
        - Build Systems (CMake, Visual Studio)
        - Suffers from Legacy
        - Complex
        - Compile Times
- [x] Rust
    - Pros
        - Safety
        - Traits
        - Better preprocessor
        - Environment (Rust Analyzer, Formatter, Docs)
        - Ownership Semantics
        - Half Decent Libraries
    - Cons
        - Very limited specialization
        - Complex
        - Compile Times
        - Cargo (Workspaces are incomplete. Dependency bloat)
- [ ] Zig
    - Pros
        - Compile Time Function Execution (Best Feature)
            - SOA
            - Reflection
        - Simple
        - Compiles C and C++
        - `cImport`
    - Cons
        - Small Community
        - Incomplete (Not at 1.0 yet)
        - No C++ libraries or things to replace them
        - Little documentation
   
### Shader Language
- [x] HLSL
    - Pros
        - Easily compiles to spirv and dx
        - Preference
        - DxCompiler
        - Bindless support
    - Cons
        - Legacy issues
- [ ] GLSL
    - Pros
        - Better syntax
        - Easily compiles to spirv and dx
        - glsc
    - Cons
        - Legacy issues
        - Bindless support sucks