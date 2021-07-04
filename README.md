# Ethereal Engine

[![license](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)
[![Build](https://github.com/colbyhall/newport/actions/workflows/ci.yml/badge.svg)](https://github.com/colbyhall/newport/actions/workflows/ci.yml)
[![Crates](https://meritbadge.herokuapp.com/newport)](https://crates.io/crates/newport)

## About
Ethereal Engine is a modular 2D and 3D game engine built in Rust for Rust. It is designed to be easily extendable and easy to use. The project is early on in development. Every API is extremely volatile as the engine is worked on more.

## Goals
* **Independent** - Build with minimal dependencies
* **Modular** - Use modules to build building blocks that are combined for engine features. 
* **Performant** - Iteration times are very important for work flows. Not only game runtime but editor time must be performant
* **Cohesive** - Keep tools and interaction with tools in Rust for simplicity

## Dependencies
Dependencies that couldn't be given up.

* [hassle-rs](https://github.com/Traverse-Research/hassle-rs)
* [ash](https://github.com/MaikKlein/ash)
* [serde](https://github.com/serde-rs/serde)
* [ron](https://github.com/ron-rs/ron)

## Inspiration
This project is heavily inspired by those that came before it. This including AAA engines like the Naughty Dog Engine or other Rust engines such as [Bevy](https://github.com/bevyengine/bevy).
