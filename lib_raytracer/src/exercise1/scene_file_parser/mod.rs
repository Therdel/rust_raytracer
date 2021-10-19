mod json_format;

use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::exercise1::object_file::{self, WindingOrder};
use crate::exercise1::Scene;
use crate::raytracing::{Material, Mesh};
use crate::utils::AliasArc;

fn get_material(materials: AliasArc<Vec<Material>, [Material]>, name: &str) -> Option<AliasArc<Vec<Material>, Material>> {
    let index = materials
        .iter()
        .enumerate()
        .find(|&(_, material)| {
            material.name == name
        })
        .map(|(index, _)| index)?;

    let materials_arc = AliasArc::into_parent(materials);
    let alias_arc = AliasArc::new(materials_arc, |vec| &vec[index]);
    Some(alias_arc)
}

fn get_mesh(meshes: AliasArc<Vec<Mesh>, [Mesh]>, name: &str) -> Option<AliasArc<Vec<Mesh>, Mesh>> {
    let index = meshes
        .iter()
        .enumerate()
        .find(|&(_, mesh)| {
            mesh.name == name
        })
        .map(|(index, _)| index)?;

    let mesh_arc = AliasArc::into_parent(meshes);
    let alias_arc = AliasArc::new(mesh_arc, |vec| &vec[index]);
    Some(alias_arc)
}

pub trait MeshLoader {
    fn load(&self, name: &str, file_name: &str, material: AliasArc<Vec<Material>, Material>,
            winding_order: WindingOrder) -> io::Result<Mesh>;
}

pub struct SceneFileParser<S: BufRead, M: MeshLoader> {
    pub file_reader: S,
    pub mesh_loader: M,
}

impl<S: BufRead, M: MeshLoader> SceneFileParser<S, M> {
    pub fn parse_json(&mut self) -> io::Result<Scene> {
        let scene: json_format::Scene = serde_json::from_reader(&mut self.file_reader)?;

        todo!()
    }
}

