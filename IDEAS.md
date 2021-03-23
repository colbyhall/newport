# Newport Ideas Document
This document lays out general API concepts and design for future consideration for implementation.

### Table Of Contents
[Engine Structure](#engine-structure)<br>
[Editor GUI](#editor-gui)<br>
[WASM Mods](#wasm-mods)<br>


## Engine Structure
Using the engine should be simple and safe to setup. Global state should be kept to an absolute minimum retained in engine state through `engine::Module`s

### Needs
1. Easy ability to add engine features through crates and `engine::Module` trait
2. Global engine state which holds systems
3. Easy module initialization ordering
4. `ModuleBuilder` on engine startup

## Editor GUI
I want a custom IMGUI for any editor UI. I want there to be a custom explicit auto layout engine. 

### Needs
1. Auto Layout Engine
2. Easy to add new controls
3. Controls including Button, Text Input, Scrollbox, Window, Slider, Color Picker
4. Single draw call renderer
5. Crates can easily add editor modules through custom windows that can snap in center editor window

## WASM Mods
Modding is a very important part to a games longevity. Being able to easily mod a game can make a game last much longer. I think WASM is perfect for this.

### Needs
1. Custom Mods structure with game API exposed to several languages
2. Hot reloading
3. Catch crashes and show error to user