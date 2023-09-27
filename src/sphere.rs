use std::sync::Arc;

use super::hit::{Hit, HitRecord};
use super::material::Scatter;
use super::ray::Ray;
use super::vec3::{Point3, Vec3};



pub struct Sphere{
    centre: Point3,
    radius: f64,
    mat: Arc<dyn Scatter>,
}

impl Sphere{
    pub fn new(centre: Point3, radius: f64, mat: Arc<dyn Scatter>) -> Sphere {
        Sphere { centre, radius, mat }
    }
}

//Can solve for whether hit a sphere via (P(t) - C) (P(t) - C) = r^2 
//Where P(t) = A + tb is the ray, A origin, b direction, t variable.
//C is the centre (Cx, Cy, Cz) of the sphere, r radius.
//Gives quadratic t^2 * b^2 + 2tb *(A-C) + (A-C)^2 - r^2 = 0
//So use determinant; two roots then goes through sphere at two points,
//one then hits as a tangent, none then misses.
//b^2 - 4ac

//We then get the closest hit point within an acceptable range (i.e. at least in front of
//the camera)
//And determine the ray hit the front side or back side of the sphere using outward surface
//norms; "front" in this case is the outer face, "back" is the inner face which could happen 
//for a sphere if the camera was inside the sphere.
impl Hit for Sphere {
    
    fn hit(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let x = r.origin() - self.centre;
        let a  = r.direction().length().powi(2);
        let half_b = r.direction().dot(x);
        let c = x.length().powi(2) - self.radius * self.radius;
        let discrim = half_b * half_b - a * c;

        //Doesn't hit
        if discrim < 0.0 { return None }
        
        //Get nearest root in acceptable range (in front of camera)
        let sqrtd = discrim.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrtd) / a;
            if root < t_min || root > t_max {
                return None
            }
        }

        let p = r.at(root);

        //Get the surface normal
        //Since p - centre gives vec from centre of sphere to p, 
        //div by radius will normalize.
        let normal = (p - self.centre) / self.radius;
        
        let mut rec = HitRecord {
            p: r.at(root),
            normal: Vec3::new(0.0, 0.0, 0.0),
            t: root,
            mat: Arc::clone(&self.mat),
            front_face: false,
        };
        
        //Calc the outward surface norm and determine whether ray 
        //is hitting from front or back
        let outward_normal = (rec.p - self.centre) / self.radius;
        rec.set_face_normal(r, outward_normal);

        Some(rec)
    }
}