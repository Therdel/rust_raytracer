use nalgebra_glm as glm;
use num_traits::identities::Zero;
use num_traits::Signed;

use crate::raytracing::{AABB, Hitpoint, Instance, MeshTriangle, MaterialIndex, Mesh, Plane, Ray, Sphere, Triangle};
use crate::raytracing::bvh::{BVH, Node, NodeType, NodeIndex};
use crate::{utils, Scene};

const NUMERIC_ERROR_COMPENSATION_OFFSET: f32 = 1e-4;

pub trait Intersect {
    type Result;
    /**
     * Tests a `ray` and an object for intersection and returns whether there is one.
     * Returns information of the hitpoint, if any
     **/
    fn intersect(&self, ray: &Ray) -> Option<Self::Result>;
}

impl<Primitive> Intersect for &[Primitive]
    where Primitive: Intersect<Result=Hitpoint> {
    type Result = Primitive::Result;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let mut closest_hitpoint = None;

        let mut check_hitpoint =
            |hitpoint| utils::take_hitpoint_if_closer(&mut closest_hitpoint, hitpoint);

        for primitive in self.iter() {
            check_hitpoint(primitive.intersect(ray));
        }

        closest_hitpoint
    }
}

impl Intersect for Sphere {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        let mut result = None;

        // m = rOrg - C
        let m = ray.origin - self.center;
        // a = rDir * rDir
        let a = glm::dot(&ray.direction, &ray.direction);
        // b = 2(m * rDir)
        let b = 2.0 * glm::dot(&m, &ray.direction);
        // c = (m * m) - r²
        let c = glm::dot(&m, &m) - self.radius*self.radius;

        // 4 rDir² (r² - (m - (m * rDir^ ) * rDir^ )² )
        // where rDir^ means normalized
        //
        // 4 dot(rDir, rDir)
        // * (pow(r, 2) - dot(m - dot(m, rDir^) * rDir^,
        //                    m - dot(m, rDir^) * rDir^)
        //   )
        let r_dir_norm = glm::normalize(&ray.direction);
        let discriminant = 4.0 * glm::dot(&ray.direction, &ray.direction)
            * (self.radius*self.radius
            - glm::dot(&(m - r_dir_norm * glm::dot(&m, &r_dir_norm)),
                       &(m - r_dir_norm * glm::dot(&m, &r_dir_norm)))
        );

        let t: Option<_> = if discriminant.is_zero() {
            Some((-0.5 * b ) / a)
        } else if discriminant.is_positive() {
            // calculate intersections
            // t0 = q / a
            // t1 = c / q
            //
            // where q = -0.5(b - sqrt(discriminant)  for b < 0
            //           -0.5(b + sqrt(discriminant)  otherwise
            let q = if b < 0.0 {
                -0.5 * (b - discriminant.sqrt())
            } else {
                -0.5 * (b + discriminant.sqrt())
            };
            let t0 = q / a;
            let t1 = c / q;


            if t0 < 0.0 && t1 >= 0.0 {
                // first intersection behind ray origin, second valid
                Some(t1)
            } else if t1 < 0.0 && t0 >= 0.0 {
                // second intersection behind ray origin, first falid
                Some(t0)
            } else {
                // either both behind ray origin (invalid) or both valid
                // take the first intersection in ray direction
                Some(f32::min(t0, t1))
            }
        } else {
            None
        };

        if let Some(t) = t {
            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);
                let normal = self.normal(&hit_position);
                let hitpoint = create_hitpoint(t, &hit_position, ray, &normal, &normal, self.material);

                result = Some(hitpoint);
            }
        }
        result
    }
}

impl Intersect for Plane {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        let mut result = None;

        let n_dot_rdir = glm::dot(&self.normal, &ray.direction);
        let parallel = n_dot_rdir == 0.0;
        if !parallel {
            // t = d - N * rOrg
            //     ------------
            //       N * rDir
            let t = (self.distance - glm::dot(&self.normal, &ray.origin))
                / n_dot_rdir;

            let does_intersect_in_ray_direction = t >= 0.0;
            if does_intersect_in_ray_direction {
                let hit_position = utils::ray_equation(ray, t);
                let hitpoint = create_hitpoint(t, &hit_position, ray, &self.normal, &self.normal, self.material);

                result = Some(hitpoint);
            }
        }
        result
    }
}

impl Intersect for Triangle {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Hitpoint> {
        let [p0, p1, p2] = &self.vertices;
        
        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let q = glm::cross(&ray.direction, &e2);
        let a = glm::dot(&e1, &q);

        const EPSILON: f32 = 1e-5;
        if a > -EPSILON && a < EPSILON { return None }

        let f = 1.0/a;
        let s = ray.origin - p0;
        let u = f * glm::dot(&s, &q);
        if u < 0.0 { return None }

        let r = glm::cross(&s, &e1);
        let v = f * glm::dot(&ray.direction, &r);
        if v < 0.0 || u + v > 1.0 { return None }

        let t = f * glm::dot(&e2, &r);
        if t < 0.0 { return None }

        let w = 1.0 - u - v;
        let hit_position = utils::ray_equation(ray, t);
        let hit_normal_gouraud = w * self.normals[0] + u * self.normals[1] + v * self.normals[2];
        let hit_normal_gouraud = glm::normalize(&hit_normal_gouraud);
        let hitpoint = create_hitpoint(t, &hit_position, ray, self.normal(), &hit_normal_gouraud, self.material);

        Some(hitpoint)
    }
}

impl Intersect for MeshTriangle {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        self.0.intersect(ray)
    }
}

impl Intersect for AABB {
    type Result = ();

    // source: https://gamedev.stackexchange.com/a/18459
    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let dirfrac = glm::vec3(
            // r.dir is unit direction vector of ray
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        // lb is the corner of AABB with minimal coordinates - left bottom, rt is maximal corner
        // r.org is origin of ray
        let lb = &self.min;
        let rt = &self.max;
        let t1 = (lb.x - ray.origin.x)*dirfrac.x;
        let t2 = (rt.x - ray.origin.x)*dirfrac.x;
        let t3 = (lb.y - ray.origin.y)*dirfrac.y;
        let t4 = (rt.y - ray.origin.y)*dirfrac.y;
        let t5 = (lb.z - ray.origin.z)*dirfrac.z;
        let t6 = (rt.z - ray.origin.z)*dirfrac.z;

        let tmin = f32::max(f32::max(f32::min(t1, t2), f32::min(t3, t4)), f32::min(t5, t6));
        let tmax = f32::min(f32::min(f32::max(t1, t2), f32::max(t3, t4)), f32::max(t5, t6));

        // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
        let _t;
        if tmax < 0.0 {
            _t = tmax;
            return None;
        }

        // if tmin > tmax, ray doesn't intersect AABB
        if tmin > tmax {
            _t = tmax;
            return None;
        }

        _t = tmin;
        Some(())
    }
}

impl Intersect for (&BVH, &Scene) {
    type Result = Hitpoint;
    
    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let (bvh, scene) = *self;
        let node_indices = &bvh.bvh_node_indices;

        if node_indices.is_empty() {
            return None;
        }

        let mut closest_hitpoint = None;
        const STACK_LEN: usize = 32;
        let mut stack = tinyvec::ArrayVec::<[NodeIndex; STACK_LEN]>::new();

        let root_index = node_indices.start;
        stack.push(root_index);
        while let Some(node_index) = stack.pop() {
            let node: &Node = &scene.mesh_bvh_nodes[node_index];
            
            let Some(()) = node.aabb.intersect(ray) else {
                continue
            };

            match &node.content {
                &NodeType::Node { child_left, child_right } => {
                    stack.push(child_left);
                    stack.push(child_right);
                }
                NodeType::Leaf { triangle_indices } => {
                    for index in triangle_indices {
                        let triangle = &scene.mesh_triangles[*index].0;
                        let hitpoint = triangle.intersect(ray);
                        utils::take_hitpoint_if_closer(&mut closest_hitpoint, hitpoint);
                    }
                }
            }
        }

        closest_hitpoint
    }
}

impl Intersect for (&Mesh, &Scene) {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let (mesh, scene) = *self;
        const USE_BVH: bool = true;
        if USE_BVH {
            (&mesh.bvh, scene).intersect(ray)
        } else if !mesh.triangle_indices.is_empty() {
            let Some(mesh_triangles) = scene.mesh_triangles.get(mesh.triangle_indices.clone()) else {
                panic!("Mesh '{}'  triangle indices ({:?}) out of bounds", mesh.name, mesh.triangle_indices);
            };
            mesh_triangles.intersect(ray)
        } else {
            None
        }
    }
}

impl<'a> Intersect for (&'a Instance<Mesh>, &Scene) {
    type Result = Hitpoint;

    fn intersect(&self, ray: &Ray) -> Option<Self::Result> {
        let (instance, scene) = *self;
        let mesh = &scene.meshes[instance.primitive_index];

        let transform = |vec: &glm::Vec3, mat: &glm::Mat4| -> glm::Vec3 {
            let homogeneous_transformed = *mat * vec.push(1.0);
            // no perspective divide needed as we're only using translate, scale & rotate
            homogeneous_transformed.xyz()
        };

        // transform ray into model-local coordinate-system
        let transformed_origin = transform(&ray.origin, &instance.model_inverse);
        let transformed_direction = glm::normalize(
            &transform(&ray.direction, &instance.rotation_scale_inverse)
        );
        let transformed_ray = Ray { origin: transformed_origin, direction: transformed_direction };

        let mut hitpoint = (mesh, scene).intersect(&transformed_ray)?;
        // transform hitpoint back into world-local coordinate-system
        hitpoint.position = transform(&hitpoint.position, &instance.model);
        hitpoint.hit_normal = glm::normalize(
            &transform(&hitpoint.hit_normal, &instance.rotation_scale)
        );
        hitpoint.position_for_refraction = transform(&hitpoint.position_for_refraction, &instance.model);

        let t_in_world = glm::distance(&ray.origin, &hitpoint.position);
        hitpoint.t = t_in_world;

        if let Some(material) = instance.material_override {
            hitpoint.material = material;
        }
        Some(hitpoint)
    }
}

// impl<Primitive> Intersect for (&Instance<Primitive>, &[Primitive]) 
//     where Primitive: Intersect<Result = Hitpoint> {
//     type Result = Primitive::Result;

//     fn intersect(&self, ray: &Ray) -> Option<Self::Result>{
//         let (instance, primitives) = *self;
//         let primitive = &primitives[instance.primitive_index];

//         let transform = |vec: &glm::Vec3, mat: &glm::Mat4| -> glm::Vec3 {
//             let homogeneous_transformed = *mat * vec.push(1.0);
//             // no perspective divide needed as we're only using translate, scale & rotate
//             homogeneous_transformed.xyz()
//         };

//         // transform ray into model-local coordinate-system
//         let transformed_origin = transform(&ray.origin, &instance.model_inverse);
//         let transformed_direction = glm::normalize(
//             &transform(&ray.direction, &instance.rotation_scale_inverse)
//         );
//         let transformed_ray = Ray { origin: transformed_origin, direction: transformed_direction };

//         let mut hitpoint = primitive.intersect(&transformed_ray)?;
//         // transform hitpoint back into world-local coordinate-system
//         hitpoint.position = transform(&hitpoint.position, &instance.model);
//         hitpoint.hit_normal = glm::normalize(
//             &transform(&hitpoint.hit_normal, &instance.rotation_scale)
//         );
//         hitpoint.position_for_refraction = transform(&hitpoint.position_for_refraction, &instance.model);

//         let t_in_world = glm::distance(&ray.origin, &hitpoint.position);
//         hitpoint.t = t_in_world;

//         if let Some(material) = instance.material_override {
//             hitpoint.material = material;
//         }
//         Some(hitpoint)
//     }
// }

fn create_hitpoint(t: f32, hit_position: &glm::Vec3, ray: &Ray, surface_normal: &glm::Vec3, hit_normal: &glm::Vec3,
                   material: MaterialIndex) -> Hitpoint {
    let n_dot_rdir = glm::dot(surface_normal, &ray.direction);
    let intersect_frontside = n_dot_rdir < 0.0;

    // invert normals when hitting the back or inside of the geometry
    let surface_normal = if intersect_frontside { *surface_normal } else { -*surface_normal };
    let hit_normal = if intersect_frontside { *hit_normal } else { -*hit_normal };

    // compensate numeric error on intersection.
    // moves hitpoint along surface normal in direction of ray origin
    // this avoids cases where hitpoints numerically "sink through" the surface
    let offset = surface_normal * NUMERIC_ERROR_COMPENSATION_OFFSET;
    let hit_position_acne_compensated = *hit_position + offset;

    // refractive ray begins on the other side of the geometry.
    // Preventing acne effects on this side, the acne compensation vector is
    // substracted from the hit position
    let hit_position_for_refraction = *hit_position - offset;

    Hitpoint {
        t,
        position: hit_position_acne_compensated,
        hit_normal,
        position_for_refraction: hit_position_for_refraction,
        on_frontside: intersect_frontside,
        material,
    }
}