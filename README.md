# ![Newport](assets/branding/logo_black_name.svg)

[![license](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Build](https://github.com/colbyhall/newport/actions/workflows/ci.yml/badge.svg)](https://github.com/colbyhall/newport/actions/workflows/ci.yml)

## About
Newport is a modular 2D and 3D game engine built in Rust for Rust. It is designed to be easily extendable and easy to use. The project is early on in development. Every API is extremely volatile as the engine is worked on more.

## Philosophy
Newport is soley being made in conjunction with a private game project. This will be one of the main driving factors in its feature set. The engine is designed to be completely multithreaded. The purpose of this is to maximize CPU usage. The engine will also be entirely designed in Rust. This includes asset format (Rust Object Notation), modding (Rust -> WASM), and shaders([rust-gpu](https://github.com/EmbarkStudios/rust-gpu)). This is less about loving Rust and more about keeping everything in the same language to loosen up friction. Several engine modules are designed to be used completely indendent. 

## Goals
* **Independent** - Build with minimal dependencies
* **Modular** - Use modules to build building blocks that are combined for engine features. 
* **Performant** - Iteration times are very important for work flows. Not only game runtime but editor time must be performant
* **Cohesive** - Keep tools and interaction with tools in Rust for simplicity

## Dependencies
Dependencies that couldn't be given up.
### Vulkan
* [rust-gpu](https://github.com/EmbarkStudios/rust-gpu)
* [ash](https://github.com/MaikKlein/ash)
### Serialization
* [serde](https://github.com/serde-rs/serde)
* [ron](https://github.com/ron-rs/ron)

## Inspiration
This project is heavily inspired by those that came before it. This including AAA engines like the Naughty Dog Engine or other Rust engines such as [Bevy](https://github.com/bevyengine/bevy).
