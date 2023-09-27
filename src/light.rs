use super::hit::{Hit, HitRecord};
use super::vec3::{Color, Point3, Vec3};



//Flat light with the same intensities at all distances
pub struct SimpleLight{
    i_diff: Color,
    i_spec: Color,
    origin: Point3,

}

impl SimpleLight {
    pub fn new(i_diff: Color, i_spec: Color, o: Point3) -> SimpleLight {
        SimpleLight {
            i_diff,
            i_spec,
            origin: o,
        }
    }
}


pub type Lighting = Vec<Box<dyn Light>>; 

impl Light for SimpleLight {
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
    fn diffuse(&self) -> Color;
    fn specular(&self) -> Color;
    fn origin(&self) -> Point3;
}