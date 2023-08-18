use std::error::Error;
use std::fmt::{self, Debug, Display};
use std::io::{self, BufRead, ErrorKind};
use std::ops::{Neg, Range};
use std::path::Path;

use nalgebra_glm as glm;
use tobj::{LoadError, LoadOptions, Model, MTLLoadResult};

use crate::raytracing::{Mesh, Triangle, MeshTriangle, MaterialIndex};
use crate::raytracing::bvh;

pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

pub fn load_mesh(name: String,
                 obj_buffer: &mut impl BufRead,
                 material: MaterialIndex,
                 winding_order: WindingOrder,
                 mesh_triangles: &mut Vec<MeshTriangle>,
                 mesh_bvh_nodes: &mut Vec<bvh::Node>) -> io::Result<Mesh> {
    let models = parse_models_from_obj_buffer(&name, obj_buffer)?;

    let amount_triangles_total = check_and_count_triangles(&name, &models)?;
    mesh_triangles.reserve_exact(amount_triangles_total);
    let triangle_indices: Range<usize> =
        match amount_triangles_total > 0 {
            true => mesh_triangles.len() .. mesh_triangles.len() + amount_triangles_total,
            false => 0..0
        };

    for model in models {
        let vertices_flat = &model.mesh.positions;
        let normals_flat = &model.mesh.normals;

        // TODO: What about parallelism for big models? Collection should be the bottleneck.
        for indices in model.mesh.indices.chunks_exact(3) {
            let indices = [indices[0] as usize, indices[1] as usize, indices[2] as usize];
            let vertices = deflatten_three_vec3s(vertices_flat, indices);
            let mut normals = deflatten_three_vec3s(normals_flat, indices);

            if let WindingOrder::CounterClockwise = winding_order {
                for normal in &mut normals {
                    *normal = normal.neg();
                }
            }

            let triangle = Triangle::new(vertices, normals, material);
            mesh_triangles.push(MeshTriangle(triangle));
        }
    }

    let bvh = bvh::BVH::build(triangle_indices.clone(), mesh_triangles, mesh_bvh_nodes);
    Ok(Mesh {
        name,
        triangle_indices,
        bvh,
    })
}

fn parse_models_from_obj_buffer(name: &str, obj_buffer: &mut impl BufRead) -> io::Result<Vec<Model>> {
    // triangulate meshes, resulting in triangles only
    // also build single/unified index for vertices and normals -> shorter code
    let load_options = LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    /// throws an error on any requested material - material files are *unsupported*
    fn material_loader(_path: &Path) -> MTLLoadResult { Err(LoadError::OpenFileFailed) }

    match tobj::load_obj_buf(obj_buffer, &load_options, material_loader) {
        Ok((models, _)) => Ok(models),
        Err(error) => ObjLoadError::create_err(name, error)
    }
}

fn check_and_count_triangles(name: &str, models: &[Model]) -> io::Result<usize> {
    let mut triangle_count = 0;

    for model in models {
        if model.mesh.positions.len() != model.mesh.normals.len() {
            return ObjLoadError::create_err(name, "Mesh doesn't have exactly one normal per vertex".to_string());
        }

        if model.mesh.indices.len() % 3 != 0 {
            return ObjLoadError::create_err(name, "Mesh vertices not divisible by 3 (not cleanly divisible into triangles)".to_string());
        }

        triangle_count += model.mesh.indices.len() / 3;
    }

    Ok(triangle_count)
}

fn deflatten_three_vec3s(flattened: &[f32],
                         indices: [usize; 3]) -> [glm::Vec3; 3] {
    [
        deflatten_vec3(flattened, indices[0]),
        deflatten_vec3(flattened, indices[1]),
        deflatten_vec3(flattened, indices[2])
    ]
}

fn deflatten_vec3(flattened: &[f32],
                  index: usize) -> glm::Vec3 {
    glm::vec3(
        flattened[index * 3],
        flattened[index * 3 + 1],
        flattened[index * 3 + 2],
    )
}

#[derive(Debug)]
struct ObjLoadError<InnerError: Display + Debug + Send + Sync> {
    name: String,
    inner_error: InnerError,
}

// TODO: Why is 'static required, what does it mean?
impl<InnerError: 'static + Display + Debug + Send + Sync> ObjLoadError<InnerError> {
    fn create_err<T>(name: &str, inner_error: InnerError) -> io::Result<T> {
        let obj_load_error = Self { name: name.into(), inner_error };
        Err(io::Error::new(ErrorKind::Other, obj_load_error))
    }
}

impl<InnerError: Display + Debug + Send + Sync> Display for ObjLoadError<InnerError> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to load .obj buffer of {}: {}", self.name, self.inner_error)
    }
}

impl<InnerError: Display + Debug + Send + Sync> Error for ObjLoadError<InnerError> {}