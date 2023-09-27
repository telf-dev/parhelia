use std::rc::Rc;
use std::sync::Arc;

use super::ray::Ray;
use super::material::Scatter;
use super::vec3::{Vec3, Point3};



pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<dyn Scatter>,
    pub t: f64,
    pub front_face: bool,
}

impl HitRecord {
    //Determine whether the ray is hitting the front or back face using 
    //outward normals (always point outwards)
    //ray . out_normal < 0.0 if hitting from front as face opp directions,
    //otherwise hitting back face
    //Therefore set the normal to either out_norm if hit front and 
    //opp of out_norm if hit from back
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = r.direction().dot(outward_normal) < 0.0;

        self.normal = if self.front_face {
            outward_normal
        } else{
            -1.0 * outward_normal
        };
    }
}

pub type World = Vec<Box<dyn Hit>>;



impl Hit for World {
    fn hit(&self, r:&Ray, t_min:f64, t_max: f64) -> Option<HitRecord> {
        let mut tmp_rec = None;
        let mut closest_so_far=t_max;

        for object in self {
            if let Some(rec) = object.hit(r, t_min, closest_so_far){
                closest_so_far = rec.t;
                tmp_rec= Some(rec);
            }
        }

        tmp_rec
    }
}

impl OccludingHit for World {
    fn occluding_hit(&self, r:&Ray, lp: Point3, t_min:f64, t_max: f64) -> bool  {
        for object in self {
            if let Some(rec) = object.hit(r, t_min, t_max){
                return rec.mat.occlusion() == 0.0 && r.direction().dot(lp - rec.p) > 0.0;
            }
        }     
        false    
    }
}


pub trait Hit: Send + Sync {
    fn hit(&self, r: &Ray, t_min:f64, t_max:f64) -> Option<HitRecord>;
}

pub trait OccludingHit: Hit {
    fn occluding_hit(&self, r:&Ray, lp: Point3, t_min:f64, t_max: f64) -> bool;
}