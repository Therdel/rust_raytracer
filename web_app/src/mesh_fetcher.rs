use std::io;
use std::io::Cursor;

use lib_raytracer::object_file;
use lib_raytracer::object_file::WindingOrder;
use lib_raytracer::scene_file::MeshLoader;
use lib_raytracer::raytracing::{bvh, MaterialIndex, Mesh, MeshTriangle};

use crate::asset_store::AssetStore;

pub struct MeshFetcher<'a> {
    pub asset_store: &'a AssetStore
}

// TODO: Report faulty Eror
// "impl method assumes more implied bounds than the corresponding trait method
// this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
// for more information, see issue #105572 <https://github.com/rust-lang/rust/issues/105572>
// `#[deny(implied_bounds_entailment)]` on by default"
// struct Bam;
// impl MeshLoader for Bam {
//     fn load(&self, name: &str, file_name: &str, material: MaterialIndex,
//             winding_order: WindingOrder, mesh_triangles: &mut Vec<Knirz>) -> io::Result<Mesh> {
//         todo!()
//     }
// }
impl MeshLoader for MeshFetcher<'_> {
    fn load(&self, name: &str, file_name: &str,
            material: MaterialIndex,
            winding_order: WindingOrder,
            mesh_triangles: &mut Vec<MeshTriangle>,
            mesh_bvh_nodes: &mut Vec<bvh::Node>) -> io::Result<Mesh> {
        let mesh_obj = self.asset_store.get_mesh(file_name);
        let mut mesh_obj_bufread = Cursor::new(mesh_obj);

        object_file::load_mesh(name.to_string(),
                               &mut mesh_obj_bufread,
                               material,
                               winding_order,
                               mesh_triangles,
                               mesh_bvh_nodes)
    }
}