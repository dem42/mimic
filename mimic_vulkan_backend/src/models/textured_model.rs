use crate::{
    models::{index::IndexType, vertex::Vertex},
    util::result::Result,
};
use bitflags::bitflags;
use log::info;
use std::{collections::HashMap, convert::TryFrom, path::Path};
use tobj::load_obj;
//////////////////////// Bitflags ///////////////////////
bitflags! {
    pub struct MeshLoadingFlags: u8 {
        const INVERTED_UP = 0b00000001;
    }
}
//////////////////////// Structs ///////////////////////
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<IndexType>,
}
//////////////////////// Impls ///////////////////////
impl Mesh {
    pub fn new(filepath: &Path, loading_props: MeshLoadingFlags) -> Result<Self> {
        let (models, _materials) = load_obj(&filepath, true)?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let mut idx_cnt = 0;
        let mut index_map = HashMap::new();
        for (_i, model) in models.iter().enumerate() {
            let face_num = model.mesh.num_face_indices.len();

            // we passed true to load_obj which triangulates faces
            for face in 0..face_num {
                for i in 0..3 {
                    let v_u32 = model.mesh.indices[3 * face + i];
                    let idx_entry = index_map.entry(v_u32).or_insert(idx_cnt);

                    // check if this vertex index is a duplicate of a vertex we already saw
                    if *idx_entry == idx_cnt {
                        // new vertex -- parse it and put it in vertices
                        let v = usize::try_from(v_u32)?;

                        let pos = glm::vec3(
                            model.mesh.positions[3 * v],
                            model.mesh.positions[3 * v + 1],
                            model.mesh.positions[3 * v + 2],
                        );

                        let color = glm::vec3(1.0, 1.0, 1.0);

                        let tu = model.mesh.texcoords[2 * v];
                        let tv = model.mesh.texcoords[2 * v + 1];
                        let tv = if loading_props.contains(MeshLoadingFlags::INVERTED_UP) {
                            1.0 - tv
                        } else {
                            tv
                        };
                        let tex_coord = glm::vec2(tu, tv);
                        let vertex = Vertex {
                            pos,
                            color,
                            tex_coord,
                        };
                        vertices.push(vertex);

                        // we used the idx_cnt value so increment
                        idx_cnt += 1;
                    }

                    indices.push(*idx_entry);
                }
            }
        }
        info!("Model \"{:?}\" loaded. Vertices: {}", filepath, idx_cnt);
        Ok(Self { vertices, indices })
    }
}