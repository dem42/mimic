//! This crate is the vulkan graphics API backend for a 3D renderer.
//! The purpose is to implement a backend API which can be use by a 3D renderer frontend.

extern crate memoffset;
extern crate nalgebra_glm as glm;

pub mod backend;
pub mod buffers;
pub mod depth;
pub mod devices;
pub mod drawing;
pub mod graphics_pipeline;
pub mod models;
pub mod msaa;
pub mod presentation;
pub mod textures;
pub mod uniforms;
pub mod util;
pub mod window;
