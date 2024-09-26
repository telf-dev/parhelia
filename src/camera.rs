// Rays are cast from the camera outwards

use super::ray::Ray;
use super::vec3::{Point3, Vec3};

pub struct Camera {
    origin: Point3,
    lower_left_corner: Point3,
    horizontal: Vec3,
    vertical: Vec3,
    cu: Vec3,
    cv: Vec3,
    lens_radius: f64,
}

impl Camera {
    pub fn new(
        lookfrom: Point3, 
        lookat: Point3, 
        vup: Vec3,
        vfov: f64, 
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64) -> Camera {
        //Image
        const FOCAL_LENGTH: f64 = 1.0;

        //Vertical fov in radians
        let theta = std::f64::consts::PI/180.0 * vfov;
        // Viewport dimensions
        let viewport_height = 2.0 * (theta/2.0).tan();
        let viewport_width = aspect_ratio * viewport_height;

        
        //vup is a canonical up vector we can use to specify the roll of the 
        //camera by specifying a canonical up regardless of the camera's angle
        
        //cu, cv, cw are a basis for the camera's plane.
        //cv is the "up" on the plane at the same angle as the camera
        //cw is opposite the direction of the cast rays; i.e. rays go in -w direction
        //cu is sideways on the camera's plane; 'right' direction.

        //vup, cw, cv are on the same plane; cu perpendicular

        let cw = (lookfrom - lookat).normalized();
        let cu = vup.cross(cw).normalized();
        let cv = cw.cross(cu);
        
        //Horizontal and vertical vectors across the viewport
        let h = focus_dist * viewport_width * cu;
        let v = focus_dist * viewport_height * cv;

        // Coordinates of the lower left corner of the viewport
        let llc = lookfrom - h/2.0 - v/2.0 - focus_dist * cw;

        Camera { 
            origin: lookfrom,
            lower_left_corner: llc,
            horizontal: h,
            vertical: v,
            cu: cu,
            cv: cv,
            lens_radius: aperture/2.0,
        }
    }

    pub fn get_ray(&self, s: f64, t: f64) -> Ray {
        let rd = self.lens_radius * Vec3::random_in_unit_disk();
        let offset = self.cu * rd.x() + self.cv * rd.y();

        Ray::new(self.origin + offset, 
            self.lower_left_corner + s * self.horizontal + t * self.vertical 
            - self.origin - offset)
    }

}