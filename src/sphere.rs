// Sphere object - calculating ray intersection given a sphere centre and radius

use std::sync::Arc;

use super::hit::{Hit, HitRecord};
use super::material::Material;
use super::ray::Ray;
use super::vec3::{Point3, Vec3};


// Spheres have a centre, radius and material
pub struct Sphere{
    centre: Point3,
    radius: f64,
    mat: Arc<dyn Material>,
}

impl Sphere{
    pub fn new(centre: Point3, radius: f64, mat: Arc<dyn Material>) -> Sphere {
        Sphere { centre, radius, mat }
    }
}


// Where P(t) = A + td is a ray, A origin, d direction, t scalar,
// And we have a sphere with centre C = (Cx, Cy, Cz), radius r,
// Can solve for whether the ray hits the sphere via (P(t) - C) . (P(t) - C) = r^2 
// Gives quadratic [t^2 * ||d||^2] + [2td . (A-C)] + [|| A-C ||^2 - r^2] = 0
// So use determinant; two roots for t -> intersects sphere at two points,
// one -> hits as a tangent, none -> ray misses.

// We then get the closest hit point within an acceptable range (i.e. at least in front of
// the camera)
// And determine if the ray hit the front side or back side of the sphere using outward surface
// norms; "front" in this case is the outer face, "back" is the inner face.
// Ray will hit the inner face first if the camera is inside the sphere.
impl Hit for Sphere {
    
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let x = r.origin() - self.centre;                       // (A - C) above
        let a  = r.direction().length().powi(2);                 // || d ||^2
        let half_b = r.direction().dot(x);                       // d . (A-C)
        let c = x.length().powi(2) - self.radius * self.radius;  // (||A-C||^2) - r^2
        let discrim = half_b * half_b - a * c;                   // (b^2/2) - ac, discriminant of the quadratic 

        //Doesn't hit
        if discrim < 0.0 { return None }
        
        //Get nearest root in acceptable range (in front of camera)
        let sqrtd = discrim.sqrt();
        let mut root = (-half_b - sqrtd) / a; //Smallest of two possible roots
        if root < t_min || root > t_max {          //occurs e.g. when camera inside sphere; ray then hits point at larger value of t.
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None
            }
        }

        //let p = r.at(root);

                
        // Make a new hit record with info on where the ray hit and the sphere's material
        
        let mut rec = HitRecord {
            p: r.at(root),
            normal: Vec3::new(0.0, 0.0, 0.0),
            t: root,
            mat: Arc::clone(&self.mat),
            front_face: false,
        };
        
        //Calc the outward surface norm and determine whether ray 
        //is hitting from front or back
        //To get the surface normal:
        //Since p - centre gives vec from centre of sphere to p, 
        //div by radius will normalize.
        let outward_normal = (rec.p - self.centre) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}