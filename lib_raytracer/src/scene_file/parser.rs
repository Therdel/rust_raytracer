use std::io::{self, BufRead};

use crate::Scene;
use crate::scene_file::{json_format, MeshLoader};
use crate::raytracing::{Camera, Light, LightColor, Material, Mesh, Plane, Sphere, Triangle, MaterialIndex, MeshIndex, MeshTriangle, Instance, bvh, Background};
use nalgebra_glm as glm;
use crate::object_file::WindingOrder::{Clockwise, CounterClockwise};
use crate::raytracing::color::ColorRgb;
use crate::raytracing::MaterialType::{Phong, ReflectAndPhong, ReflectAndRefract};

pub struct Parser<S: BufRead, M: MeshLoader> {
    pub file_reader: S,
    pub mesh_loader: M,
}

impl From<json_format::Vec3> for glm::Vec3 {
    fn from(vec: json_format::Vec3) -> Self {
        glm::vec3(vec.x, vec.y, vec.z)
    }
}

impl From<json_format::Vec4> for glm::Vec4 {
    fn from(vec: json_format::Vec4) -> Self {
        glm::vec4(vec.x, vec.y, vec.z, vec.w)
    }
}

impl From<json_format::ColorRgb> for ColorRgb {
    fn from(color: json_format::ColorRgb) -> Self {
        glm::vec3(color.r, color.g, color.b)
        // TODO: Why does this crash? color.into()
    }
}

impl From<json_format::Camera> for Camera {
    fn from(camera: json_format::Camera) -> Self {
        let orientation_degrees = camera.orientation_degrees.into();
        Self {
            position: camera.position.into(),
            orientation: glm::radians(&orientation_degrees),
            screen_dimensions: glm::vec2(camera.screen_dimensions.0, camera.screen_dimensions.1),
            y_fov_degrees: camera.y_fov_degrees,
            z_near: camera.z_near,
            z_far: camera.z_far,
        }
    }
}

impl From<json_format::Background> for Background {
    fn from(background: json_format::Background) -> Self {
        match background {
            json_format::Background::SolidColor(color) =>
                Background::SolidColor(color.into()),
            json_format::Background::ColoredDirection => 
                Background::ColoredDirection,
        }
    }
}

impl<S: BufRead, M: MeshLoader> Parser<S, M> {
    pub fn parse_json(&mut self) -> io::Result<Scene> {
        let json: json_format::Scene = serde_json::from_reader(&mut self.file_reader)?;

        let camera = json.camera.into();
        let background = json.background.into();
        let mut scene = Scene::new(camera, background);

        scene.lights = Self::convert_lights(json.lights);
        scene.materials = Self::convert_materials(json.materials);
        scene.planes = Self::convert_planes(json.planes, &scene.materials);
        scene.spheres = Self::convert_spheres(json.spheres, &scene.materials);
        scene.triangles = Self::convert_triangles(json.triangles, &scene.materials);
        (scene.mesh_triangles,
         scene.mesh_bvh_nodes,
         scene.meshes) = self.convert_meshes(json.meshes, &scene.materials);
        scene.mesh_instances = Self::convert_mesh_instances(json.mesh_instances, &scene.meshes, &scene.materials);
        Ok(scene)
    }

    fn convert_vec<JsonElem, ModelElem, F>(vec: Vec<JsonElem>, convert: F) -> Vec<ModelElem>
        where F: FnMut(JsonElem) -> ModelElem {
        vec
            .into_iter()
            .map(convert)
            .collect()
    }

    fn convert_lights(lights: Vec<json_format::Light>) -> Vec<Light> {
        Self::convert_vec(lights, |light|
            Light {
                position: light.position.into(),
                color: LightColor {
                    ambient: light.color.ambient.into(),
                    diffuse: light.color.diffuse.into(),
                    specular: light.color.specular.into(),
                },
            }
        )
    }

    fn convert_materials(materials: Vec<json_format::Material>) -> Vec<Material> {
        use json_format::MaterialType as JsonMat;

        Self::convert_vec(materials, |material| {
            let material_type = match material.material_type {
                JsonMat::Phong => Phong,
                JsonMat::ReflectAndPhong => ReflectAndPhong,
                JsonMat::ReflectAndRefract { index_inner, index_outer } =>
                    ReflectAndRefract { index_inner, index_outer }
            };

            Material {
                name: material.name,
                emissive: material.emissive.into(),
                ambient: material.ambient.into(),
                diffuse: material.diffuse.into(),
                specular: material.specular.into(),
                shininess: material.shininess,
                material_type
            }
        })
    }

    fn find_material(materials: &[Material],
                     name: &str) -> Option<MaterialIndex> {
        materials
            .iter()
            .enumerate()
            .find(|&(_, material)| {
                material.name == name
            })
            .map(|(index, _)| MaterialIndex(index))
    }

    fn find_mesh(meshes: &[Mesh],
                 name: &str) -> Option<MeshIndex> {
        meshes
            .iter()
            .enumerate()
            .find(|&(_, mesh)| {
                mesh.name == name
            })
            .map(|(index, _)| MeshIndex(index))
    }

    fn convert_planes(planes: Option<Vec<json_format::Plane>>,
                      materials: &[Material]) -> Vec<Plane> {
        if let Some(planes) = planes {
            Self::convert_vec(planes, |plane|
                Plane::new(plane.normal.into(), plane.distance,
                           Self::find_material(materials, &plane.material).unwrap())
            )
        } else {
            vec![]
        }
    }

    fn convert_spheres(spheres: Option<Vec<json_format::Sphere>>,
                       materials: &[Material]) -> Vec<Sphere> {
        if let Some(spheres) = spheres {
            Self::convert_vec(spheres, |sphere|
                Sphere {
                    center: sphere.center.into(),
                    radius: sphere.radius,
                    material: Self::find_material(materials, &sphere.material).unwrap(),
                }
            )
        } else {
            vec![]
        }
    }

    fn convert_triangles(triangles: Option<Vec<json_format::Triangle>>,
                         materials: &[Material]) -> Vec<Triangle> {
        if let Some(triangles) = triangles {
            Self::convert_vec(triangles, |triangle|
                Triangle::new(
                    triangle.vertices.map(Into::into),
                    triangle.normals.map(Into::into),
                    Self::find_material(materials, &triangle.material).unwrap(),
                )
            )
        } else {
            vec![]
        }
    }

    fn convert_meshes(&self,
                      meshes: Option<Vec<json_format::Mesh>>,
                      materials: &[Material]) -> (Vec<MeshTriangle>, Vec<bvh::Node>, Vec<Mesh>) {
        if let Some(meshes) = meshes {
            let mut mesh_triangles = Vec::new();
            let mut mesh_bvh_nodes = Vec::new();
            let meshes = Self::convert_vec(meshes, |mesh| {
                let material = Self::find_material(materials, &mesh.material).unwrap();
                let winding_order = match mesh.winding_order {
                    json_format::WindingOrder::Clockwise => Clockwise,
                    json_format::WindingOrder::CounterClockwise => CounterClockwise
                };

                // TODO: Speed-up by bulk-reserving all mesh triangles once
                self.mesh_loader.load(&mesh.name, &mesh.file_name, material, winding_order,
                                      &mut mesh_triangles, &mut mesh_bvh_nodes).unwrap()
            });
            (mesh_triangles, mesh_bvh_nodes, meshes)
        } else {
            Default::default()
        }
    }

    fn convert_mesh_instances(instances: Option<Vec<json_format::MeshInstance>>,
                              meshes: &[Mesh],
                              materials: &[Material]) -> Vec<Instance<Mesh>> {
        if let Some(instances) = instances {
            Self::convert_vec(instances, |instance| {
                let mesh = Self::find_mesh(meshes, &instance.mesh).unwrap();
                let material_override = instance.material_override.map(|name|
                    Self::find_material(materials, &name).unwrap()
                );
                Instance::new(mesh.0,
                    instance.position.into(),
                    glm::radians(&instance.orientation_degrees.into()),
                    instance.scale.into(),
                    material_override,
                )
            })
        } else {
            vec![]
        }
    }
}

