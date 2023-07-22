use std::io::{self, BufRead};
use std::sync::Arc;

use crate::exercise1::Scene;
use crate::exercise1::scene_file::{json_format, MeshLoader};
use crate::raytracing::{Camera, Instance, Light, LightColor, Material, Mesh, Plane, Screen, Sphere, Triangle};
use crate::utils::AliasArc;
use nalgebra_glm as glm;
use crate::exercise1::object_file::WindingOrder::{Clockwise, CounterClockwise};
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
            y_fov_degrees: camera.y_fov_degrees,
            z_near: camera.z_near,
            z_far: camera.z_far,
        }
    }
}

impl From<json_format::Screen> for Screen {
    fn from(screen: json_format::Screen) -> Self {
        Self {
            pixel_width: screen.pixel_width,
            pixel_height: screen.pixel_height,
            background: screen.background.into()
        }
    }
}

impl<S: BufRead, M: MeshLoader> Parser<S, M> {
    pub fn parse_json(&mut self) -> io::Result<Scene> {
        let json: json_format::Scene = serde_json::from_reader(&mut self.file_reader)?;

        let camera = json.camera.into();
        let screen = json.screen.into();
        let lights = Self::convert_lights(json.lights);
        let materials = Self::convert_materials(json.materials);
        let planes = Self::convert_planes(json.planes, &materials);
        let spheres = Self::convert_spheres(json.spheres, &materials);
        let triangles = Self::convert_triangles(json.triangles, &materials);
        let meshes = self.convert_meshes(json.meshes, &materials);
        let mesh_instances = Self::convert_mesh_instances(json.mesh_instances, &meshes, &materials);

        let scene = Scene {
            camera,
            screen,
            lights,
            materials,
            planes,
            spheres,
            triangles,
            meshes,
            mesh_instances,
        };
        Ok(scene)
    }

    fn convert_vec<JsonElem, ModelElem, F>(vec: Vec<JsonElem>, convert: F) -> AliasArc<Vec<ModelElem>, [ModelElem]>
        where F: FnMut(JsonElem) -> ModelElem {
        let arc = Arc::new(
            vec
                .into_iter()
                .map(convert)
                .collect()
        );
        AliasArc::new(arc, Vec::as_slice)
    }

    fn convert_lights(lights: Vec<json_format::Light>) -> AliasArc<Vec<Light>, [Light]>{
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

    fn convert_materials(materials: Vec<json_format::Material>) -> AliasArc<Vec<Material>, [Material]> {
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

    fn find_material(materials: AliasArc<Vec<Material>, [Material]>,
                     name: &str) -> Option<AliasArc<Vec<Material>, Material>> {
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

    fn find_mesh(meshes: AliasArc<Vec<Mesh>, [Mesh]>,
                 name: &str) -> Option<AliasArc<Vec<Mesh>, Mesh>> {
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

    fn convert_planes(planes: Option<Vec<json_format::Plane>>,
                      materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Plane>, [Plane]> {
        if let Some(planes) = planes {
            Self::convert_vec(planes, |plane|
                Plane::new(plane.normal.into(), plane.distance,
                           Self::find_material(materials.clone(), &plane.material).unwrap())
            )
        } else {
            AliasArc::new(Default::default(), Vec::as_slice)
        }
    }

    fn convert_spheres(spheres: Option<Vec<json_format::Sphere>>,
                       materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Sphere>, [Sphere]> {
        if let Some(spheres) = spheres {
            Self::convert_vec(spheres, |sphere|
                Sphere {
                    center: sphere.center.into(),
                    radius: sphere.radius,
                    material: Self::find_material(materials.clone(), &sphere.material).unwrap(),
                }
            )
        } else {
            AliasArc::new(Default::default(), Vec::as_slice)
        }
    }

    fn convert_triangles(triangles: Option<Vec<json_format::Triangle>>,
                         materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Triangle>, [Triangle]> {
        if let Some(triangles) = triangles {
            Self::convert_vec(triangles, |triangle|
                Triangle::new(
                    triangle.vertices.map(Into::into),
                    triangle.normals.map(Into::into),
                    Self::find_material(materials.clone(), &triangle.material).unwrap(),
                )
            )
        } else {
            AliasArc::new(Default::default(), Vec::as_slice)
        }
    }

    fn convert_meshes(&self,
                      meshes: Option<Vec<json_format::Mesh>>,
                      materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Mesh>, [Mesh]> {
        if let Some(meshes) = meshes {
            Self::convert_vec(meshes, |mesh| {
                let material = Self::find_material(materials.clone(), &mesh.material).unwrap();
                let winding_order = match mesh.winding_order {
                    json_format::WindingOrder::Clockwise => Clockwise,
                    json_format::WindingOrder::CounterClockwise => CounterClockwise
                };

                self.mesh_loader.load(&mesh.name, &mesh.file_name, material, winding_order).unwrap()
            })
        } else {
            AliasArc::new(Default::default(), Vec::as_slice)
        }
    }

    fn convert_mesh_instances(instances: Option<Vec<json_format::MeshInstance>>,
                              meshes: &AliasArc<Vec<Mesh>, [Mesh]>,
                              materials: &AliasArc<Vec<Material>, [Material]>) -> AliasArc<Vec<Instance<Mesh>>, [Instance<Mesh>]> {
        if let Some(instances) = instances {
            Self::convert_vec(instances, |instance| {
                let mesh = Self::find_mesh(meshes.clone(), &instance.mesh).unwrap();
                let material_override = instance.material_override.map(|name|
                    Self::find_material(materials.clone(), &name).unwrap()
                );
                Instance::new(mesh,
                              instance.position.into(),
                              glm::radians(&instance.orientation_degrees.into()),
                              instance.scale.into(),
                              material_override,
                )
            })
        } else {
            AliasArc::new(Default::default(), Vec::as_slice)
        }
    }
}

