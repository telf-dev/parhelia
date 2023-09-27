use rand::seq::index;
use rand::Rng;

use super::vec3::{Color, Point3, Vec3};
use super::ray::Ray;
use super::hit::{Hit, HitRecord, OccludingHit, World};
use super::light::{Light, Lighting};


pub trait Scatter: Send + Sync {
    fn scatter(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
    fn occlusion(&self) -> f64;
}



pub struct Lambertian {
    albedo: Color,
    occlusion: f64,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { albedo, occlusion: 0.0 }
    }
}

impl Scatter for Lambertian {
    //Calculate a new ray (the ray scattered off the object) and its color.
    fn scatter(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>{
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


//This is actually specular reflection
pub struct Metal {
    albedo: Color,
    fuzz: f64,
    occlusion: f64,
}

impl Metal {
    pub fn new(a: Color, f: f64) -> Metal {
        Metal {albedo: a, fuzz: f, occlusion: 0.0}
    }
}

impl Scatter for Metal {
    fn scatter(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
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

impl Scatter for Dielectric {
    fn scatter(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
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


pub struct PhongMat {
    a: f64,
    d: f64,
    s: f64,
    shine: f64,
    //B = shine/gamma
    B: f64,
    //Ideally want gamma to be a power of 2 for power efficiency; 4 or 8 should suffice
    //gamma can be a float but locked it to int for now so remember power of 2
    g: i32,
    albedo: Color,
    fuzz: f64,
    d_s: f64,
    occlusion: f64,

}

impl PhongMat {
    pub fn new(a: f64, d: f64, s: f64, shine: f64, g: i32, albedo: Color, fuzz: f64, d_s: f64, occlusion :f64) -> PhongMat{
        PhongMat {
            a,
            d,
            s,
            shine,
            B: shine/(g as f64),
            g,
            albedo,
            fuzz,
            d_s,
            occlusion,
         }
    }
}

impl Scatter for PhongMat{
    fn scatter(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>{
        self.illumination(vpos, lights, world, r_in, rec)
    }
    fn occlusion(&self) -> f64 {
        self.occlusion
    }
}

impl Phongian for PhongMat {
    fn illumination(&self, vpos: Point3, lights: &Lighting, world: &World,  r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        //Calculate illumination
        let mut illumination = Color::new(0.0, 0.0, 0.0);
        
        let viewer_direction = (vpos - rec.p).normalized();
        
        for light in lights {
            if Self::is_lit(rec.p, rec.normal, &world, light.origin()) {
                let L = (light.origin()-rec.p).normalized();
                let diffuse = (L.dot(rec.normal));
                
                let R = L.reflect(rec.normal).normalized();
                let lambda = 1.0 - R.dot(viewer_direction);
                
                let tmp = 1.0-self.B*lambda;

                let specular = if 0.0 < tmp {
                    tmp.powi(self.g)
                } else {
                    0.0
                };

                //TODO: ambient term
                
                illumination += (self.d * diffuse * light.diffuse()) 
                    + (self.s * specular * light.specular());
            }
        }
        //TODO: divide illumination by number of lights in scene?

        //Calculate scatter direction
        if rand::thread_rng().gen_range(0.0..1.0) < self.d_s {
            if let Some((attenuation, scattered)) = self.lambertian(&r_in, &rec){
                return Some((illumination * attenuation, scattered));
            }
        }
        else{
            if let Some((attenuation, scattered)) = self.specular(&r_in, &rec) {
                return Some((illumination * attenuation, scattered));
            }
        }
        None
    }

    fn is_lit(p: Point3, n: Vec3, world: &World, lpos: Point3) -> bool {
        //TODO: perhaps make this 0.001; only supposed to calc illumination if this
        //term is positive
        if n.dot(lpos - p) < 0.0 {
            return false
        }

        let ray = Ray::new(p, (lpos - p).normalized());
        return !world.occluding_hit(&ray, lpos, 0.001, f64::INFINITY)
    }
}

impl Specular for PhongMat {
    fn specular(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let scatter_direction = r_in.direction().reflect(rec.normal).normalized();
        let scattered = Ray::new(rec.p, scatter_direction + self.fuzz * Vec3::random_in_unit_sphere());

        if scattered.direction().dot(rec.normal) > 0.0 {
            Some((self.albedo, scattered))
        }
        else {
            None
        }
    }
}

impl Lamb for PhongMat {
    fn lambertian(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)> {
        let mut scatter_direction = rec.normal + Vec3::random_in_unit_sphere().normalized();
        //Catch degen scatter direction (exactly opposite normal, gets 0 length, will cause 
        //zero and infinity errors
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        Some((self.albedo, Ray::new(rec.p, scatter_direction)))
    }
}


pub trait Phongian: Lamb + Specular {
    fn illumination(&self, vpos: Point3, lights: &Lighting, world: &World, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
    fn is_lit(p: Point3, n: Vec3, world: &World, lpos: Point3) -> bool;
}

pub trait Lamb {
    fn lambertian(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}

pub trait Specular {
    fn specular(&self, r_in: &Ray, rec: &HitRecord) -> Option<(Color, Ray)>;
}