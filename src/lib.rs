// SPDX-FileCopyrightText: Copyright (c) 2022-2025 Objectionary.com
// SPDX-License-Identifier: MIT

//! This is a memory structure with vertices and edges between them,
//! which we call Surging Object `DiGraph` (SODG), because it expects
//! modifications comping from a user (through [`Sodg::add`],
//! [`Sodg::bind`], and [`Sodg::put`]) and then decides itself when
//! it's time to delete some vertices (something similar to
//! "garbage collection").
//!
//! For example, here is how you create a simple
//! di-graph with two vertices and an edge between them:
//!
//! ```
//! use std::str::FromStr as _;
//! use sodg::{Label, Sodg};
//! let mut sodg : Sodg<16> = Sodg::empty(256);
//! sodg.add(0);
//! sodg.add(1);
//! sodg.bind(0, 1, Label::from_str("foo").unwrap());
//! ```

#![doc(html_root_url = "https://docs.rs/sodg/0.0.0")]
#![deny(warnings)]
#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_inherent_impl)]
#![allow(clippy::multiple_crate_versions)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

mod clone;
mod ctors;
mod debug;
mod dot;
mod hex;
mod inspect;
mod label;
mod merge;
mod misc;
mod next;
mod ops;
mod script;
mod serialization;
mod slice;
mod xml;

const HEX_SIZE: usize = 8;
const MAX_BRANCHES: usize = 16;
const MAX_BRANCH_SIZE: usize = 16;

/// An object-oriented representation of binary data
/// in hexadecimal format, which can be put into vertices of the graph.
///
/// You can create it from Rust primitives:
///
/// ```
/// use sodg::Hex;
/// let d = Hex::from(65534);
/// assert_eq!("00-00-FF-FE", d.print());
/// ```
///
/// Then, you can turn it back to Rust primitives:
///
/// ```
/// use sodg::Hex;
/// let d = Hex::from(65534_i64);
/// assert_eq!(65534, d.to_i64().unwrap());
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub enum Hex {
    Vector(Vec<u8>),
    Bytes([u8; HEX_SIZE], usize),
}

/// A label on an edge.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Label {
    Greek(char),
    Alpha(usize),
    Str([char; 8]),
}

/// A wrapper of a plain text with graph-modifying instructions.
///
/// For example, you can pass the following instructions to it:
///
/// ```text
/// ADD(0);
/// ADD($ν1); # adding new vertex
/// BIND(0, $ν1, foo);
/// PUT($ν1, d0-bf-D1-80-d0-B8-d0-b2-d0-b5-d1-82);
/// ```
///
/// In the script you can use "variables", similar to `$ν1` used
/// in the text above. They will be replaced by autogenerated numbers
/// during the deployment of this script to a [`Sodg`].
pub struct Script {
    /// The text of it.
    txt: String,
    /// The vars dynamically discovered.
    vars: HashMap<String, usize>,
}

/// A struct that represents a Surging Object Di-Graph (SODG).
///
/// You add vertices to it, bind them one to one with edges,
/// put data into some of them, and read data back, for example:
///
/// ```
/// use sodg::{Label, Sodg};
/// let mut sodg : Sodg<16> = Sodg::empty(256);
/// sodg.add(0);
/// sodg.add(1);
/// sodg.bind(0, 1, Label::Alpha(0));
/// sodg.add(2);
/// sodg.bind(1, 2, Label::Alpha(1));
/// assert_eq!(1, sodg.kids(0).count());
/// assert_eq!(1, sodg.kids(1).count());
/// ```
///
/// This package is used in [reo](https://github.com/objectionary/reo)
/// project, as a memory model for objects and dependencies between them.
#[derive(Serialize, Deserialize)]
pub struct Sodg<const N: usize> {
    stores: emap::Map<usize>,
    branches: emap::Map<microstack::Stack<usize, MAX_BRANCH_SIZE>>,
    vertices: emap::Map<Vertex<N>>,
    /// This is the next ID of a vertex to be returned by the [`Sodg::next_v`] function.
    #[serde(skip_serializing, skip_deserializing)]
    next_v: usize,
}

#[derive(PartialEq, Serialize, Deserialize, Clone)]
enum Persistence {
    Empty,
    Stored,
    Taken,
}

const BRANCH_NONE: usize = 0;
const BRANCH_STATIC: usize = 1;

#[derive(Serialize, Deserialize, Clone)]
struct Vertex<const N: usize> {
    branch: usize,
    data: Hex,
    persistence: Persistence,
    edges: micromap::Map<Label, usize, N>,
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    use log::LevelFilter;
    use simple_logger::SimpleLogger;

    SimpleLogger::new()
        .without_timestamps()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
}
