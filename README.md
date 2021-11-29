# Newport
Modular 2D and 3D game engine built in Rust for Rust. It is designed to be easily extendable and easy to use. The project is early on in development. Every API is extremely volatile as the engine is worked on more. Documentation and Testing is minimal

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![ci](https://github.com/colbyhall/newport/actions/workflows/ci.yml/badge.svg)](https://github.com/colbyhall/newport/actions/workflows/ci.yml)

## Table Of Contents
[Goals](#goals)<br>
[Features](#features)<br>
[Inspiration](#inspiration)<br>
[Research](https://github.com/colbyhall/newport/RESEARCH.md)<br>

## Goals
* **Modular** - Use modules to build building blocks that are combined for engine features. 
* **Performant** - Iteration times are very important for work flows. Not only game runtime but editor time must be performant
* **Cohesive** - Keep tools and interaction with tools in Rust for simplicity

## Features
* **GPU Abstraction Layer** - Thread Safe, HLSL Shaders, Built over Vulkan with DirectX12 back end coming soon, Bindless Resource Model
* **Resource Manager** - Thread Safe, Supports Import Variation, Garbage Collected
* **Entity Component System** - Thread Safe, Async, Readable Scene Format
* **2D Paint** - Shape rendering, Text Rendering, Single Draw Call

## Inspiration
This project is heavily inspired by those that came before it. This including AAA engines like the Naughty Dog Engine or other Rust engines such as [Bevy](https://github.com/bevyengine/bevy).
