/*  
    Materials for different types of scattering; diffuse (matt), specular (mirror-like), 
    dielectric (refraction - water, glass)
    Phongian is combined diffuse/specular, giving a matt material with spots of 
    specular reflection, like e.g. a ceramic cup.
*/
use rand::seq::index;
use rand::Rng;

use super::vec3::{Color, Point3, Vec3};
use super::ray::Ray;
use super::hit::{Hit, HitRecord, OccludingHit, World};
use super::light::{Light, Lighting};


pub trait Material: Send + Sync {
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
    fn emitted(&self) -> Color{
        Color::new(0.0, 0.0, 0.0)
    }
    fn occlusion(&self) -> f64;
}

pub struct DiffuseLight {
    albedo: Color,
    occlusion: f64,
    emission: Color,
}

impl DiffuseLight {
    pub fn new(albedo: Color, occlusion: f64, emission: Color) -> DiffuseLight {
        DiffuseLight { albedo, occlusion, emission }
    }
}

impl Material for DiffuseLight {
    //Calculate a new ray (the ray scattered off the object) and its color.
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>{
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere().normalized();
        //Catch degen scatter direction (exactly opposite normal, gets 0 length, will cause 
        //zero and infinity errors
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
    fn emitted(&self) -> Color {
        self.emission
    }

    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}

// Random scattering; creates a diffuse (matt, dull) effect
pub struct Diffuse {
    albedo: Color,
    occlusion: f64,
}

impl Diffuse {
    pub fn new(albedo: Color, occlusion: f64) -> Diffuse {
        Diffuse { albedo, occlusion}
    }
}

impl Material for Diffuse {
    //Calculate a new ray (the ray scattered off the object) and its color.
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>{
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere().normalized();
        //Catch degen scatter direction (exactly opposite normal, gets 0 length, will cause 
        //zero and infinity errors
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}


// Specular reflection (mirror-like surfaces) - reflect at same angle from the norm
// i.e. for a ray arriving at angle theta from the norm, the reflected ray will have direction (180 - 2*theta)
pub struct Specular {
    albedo: Color,
    fuzz: f64,
    occlusion: f64,
}

impl Specular {
    pub fn new(a: Color, f: f64, occlusion: f64) -> Specular {
        Specular {albedo: a, fuzz: f, occlusion}
    }
}

impl Material for Specular {
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let scatter_direction = r_in.direction().reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.p, scatter_direction + self.fuzz * Vec3::random_in_unit_sphere());

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        }
        else {
            None
        }
    }
    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}

pub struct Phongian {
    albedo: Color,
    fuzz: f64,
    k_diff: f64,
    k_spec: f64,
    alpha: f64,
    occlusion: f64
}

impl Phongian {
    pub fn new(a: Color, f: f64, kd: f64, ks: f64, alpha: f64) -> Phongian {
        Phongian {
            albedo: a,
            fuzz: f,
            k_diff: kd,
            k_spec: ks,
            alpha: alpha,
            occlusion: 1.0,
        }
    }
}

impl Material for Phongian {
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>{
        let light_norm_sim = r_in.direction().dot(rec.normal);
        let scatter_direction = r_in.direction().reflect(rec.normal).normalized();
        let ref_viewer_sim = scatter_direction.dot(vpos);

        if light_norm_sim < 0.0 && ref_viewer_sim < 0.0 {
            return None
        }

        if light_norm_sim > self.alpha * ref_viewer_sim {
            let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere().normalized();
            //Catch degen scatter direction (exactly opposite normal, gets 0 length, will cause 
            //zero and infinity errors
            if scatter_direction.near_zero() {
                scatter_direction = rec.normal;
            }
            return Some((self.albedo, Ray::new(rec.p, scatter_direction)));
        }
        else{
            let scattered = Ray::new(rec.p, scatter_direction + self.fuzz * Vec3::random_in_unit_sphere());

            if scattered.direction().dot(rec.normal) > 0.0 {
                return Some((self.albedo, scattered))
            }
        }
        None
    }
    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}


// Dielectrics i.e. glass, water, split light into a reflected and refracted ray.
// For simplicity and speed (do not want to generate 2^n rays from n hits), choose only 
// one of these per interaction with the material
pub struct Dielectric {
    ir: f64,
    occlusion: f64,
}

impl Dielectric {
    pub fn new(index_of_refraction: f64, occlusion: f64) -> Dielectric {
        Dielectric { ir: index_of_refraction, occlusion }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        //Schlick's approximation for reflectance
        let r0 = ((1.0 - ref_idx) / (1.0 + ref_idx)).powi(2);
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, vpos: Point3, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let refraction_ratio = if rec.front_face {
            1.0/self.ir
        } else {
            self.ir
        };

        let unit_direction = r_in.direction().normalized();
        let cos_theta = ((-1.0) * unit_direction).dot(rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta.powi(2)).sqrt();

        let mut rng = rand::thread_rng();
        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let will_reflect = rng.gen::<f64>() < Self::reflectance(cos_theta, refraction_ratio);

        let direction = if cannot_refract || will_reflect {
            //Must reflect (no solution to refraction eqns)
            unit_direction.reflect(rec.normal)
        } else {
            //Can refract
            unit_direction.refract(rec.normal, refraction_ratio)
        };

        let scattered = Ray::new(rec.p, direction);

        Some((Color::new(1.0, 1.0, 1.0), scattered))
    }
    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}
