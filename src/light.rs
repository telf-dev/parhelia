// Unused; for messing with different shading ideas.

use super::hit::{Hit, HitRecord};
use super::vec3::{Color, Point3, Vec3};



//Flat light with the same intensities at all distances
pub struct SimpleLight{
    strength: f64,
    i_diff: Color,
    i_spec: Color,
    origin: Point3,

}

impl SimpleLight {
    pub fn new(strength: f64, i_diff: Color, i_spec: Color, o: Point3) -> SimpleLight {
        SimpleLight {
            strength,
            i_diff,
            i_spec,
            origin: o,
        }
    }
}


pub type Lighting = Vec<Box<dyn Light>>; 

impl Light for SimpleLight {
    fn strength(&self) -> f64{
        self.strength
    }
    fn diffuse(&self) -> Color{
        self.i_diff
    }
    fn specular(&self) -> Color {
        self.i_spec
    }
    fn origin(&self) -> Point3 {
        self.origin    
    }
}


pub trait Light: Send + Sync { 
    fn strength(&self) -> f64;
    fn diffuse(&self) -> Color;
    fn specular(&self) -> Color;
    fn origin(&self) -> Point3;
}