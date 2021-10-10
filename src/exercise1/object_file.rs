use std::path::Path;
use crate::raytracing::{Material, Mesh, Triangle};
use tobj::{LoadOptions, Model};
use std::ops::Neg;
use nalgebra_glm as glm;

pub enum WindingOrder {
    Clockwise,
    CounterClockwise,
}

pub fn load_mesh<'a>(id: String,
                     path: &Path,
                     material: &'a Material,
                     winding_order: WindingOrder) -> Result<Mesh<'a>, String> {
    let models = parse_models_from_obj(path)?;

    let mut mesh = Mesh {
        id,
        triangles: vec![],
    };
    let amount_triangles_total = check_and_count_triangles(&models)?;
    mesh.triangles.reserve_exact(amount_triangles_total);

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
            mesh.triangles.push(triangle);
        }
    }

    Ok(mesh)
}

fn parse_models_from_obj(path: &Path) -> Result<Vec<Model>, String> {
    // triangulate meshes, resulting in triangles only
    // also build single/unified index for vertices and normals -> shorter code
    let mut load_options = LoadOptions::default();
    load_options.triangulate = true;
    load_options.single_index = true;

    // TODO: Beautify
    match tobj::load_obj(path, &load_options) {
        Ok((models, _)) => Ok(models),
        Err(error) => Err(format!("Failed to load .obj file: {}", error.to_string()))
    }
}

fn check_and_count_triangles(models: &[Model]) -> Result<usize, String> {
    let mut triangle_count = 0;

    for model in models {
        if model.mesh.positions.len() != model.mesh.normals.len() {
            return Err("Mesh doesn't have exactly one normal per vertex".to_string());
        }

        if model.mesh.indices.len() % 3 != 0 {
            return Err("Mesh vertices not divisible by 3 (not cleanly divisible into triangles)".to_string());
        }

        triangle_count = triangle_count + model.mesh.indices.len() / 3;
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
        flattened[index * 3 + 0],
        flattened[index * 3 + 1],
        flattened[index * 3 + 2],
    )
}

