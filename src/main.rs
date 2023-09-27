use std::io::{stderr, Write};
use std::sync::Arc;

use rand::Rng;
use rayon::prelude::*;


mod camera;
mod hit;
mod light;
mod material;
mod ray;
mod sphere;
mod vec3;

use camera::Camera;
use light::{Light, Lighting, SimpleLight};
use vec3::{Vec3, Point3, Color};
use ray::Ray;
use material::{Dielectric, Lambertian, Metal, PhongMat};
use sphere::Sphere;
use hit::{OccludingHit, Hit, HitRecord, World};


fn lambertian_hardcoded(rec: &HitRecord, world: &World, lights: &Lighting, depth: u64) -> Color{
    //Lambertian reflection: Produce random points on the surface of the unit ball 
        //offset along the surface normal; has a distribution of cos(phi) where phi is the angle
        //from the normal. Without normalizing the final term we get a cos^3(phi) dist corresponding 
        //to picking directions on the hemisphere with high prob close to the normal, and a lower
        //prob of scattering rays at grazing angles. Shallow angles contribute less to color so wouldn't care
        //but we want the lambertian dist.
        //lambertian dist is more uniform though still has higher prob closer to norm
        //than further. Get less pronounced shadows, esp in gaps between objects, as less light 
        //bounces straight up i.e. close to normal.
        //Also means diffuse surfaces become brighter as more light bounces towards camera.
        let target = rec.p + rec.normal + Vec3::random_in_unit_sphere().normalized();

        //Alternate method - hemispherical scattering, older method that just randomly scatters
        //rays off the hit point, uniform distribution
        //Gets you lighter surfaces + less prominent shadows because dist uniform so more rays going 
        //towards camera, also not bouncing straight up to other object as much.
        //let target = rec.p + Vec3::random_in_hemisphere(rec.normal);

        let r = Ray::new(rec.p, target-rec.p);
        //Hit an object; return the face normal of the object
        0.5 * ray_color(&r, &world, &lights, depth - 1)
}

fn is_lit(p: Point3, n: Vec3, world: &World, lights: &Lighting) -> Option<Color> {
    for light in lights {
        if n.dot(light.origin() - p) < 0.0 {
            continue;
        }
        else{
            //TODO don't need to normalize here?
            let ray = Ray::new(p, (light.origin() - p).normalized());
            if !world.occluding_hit(&ray, light.origin(), 0.001, f64::INFINITY){
                return Some(light.diffuse());
            }
        }
    }
    return None
}

fn ray_color(r: &Ray, world: &World, lights: &Lighting, depth: u64) -> Color {
    if depth <= 0{
        //Exceeded ray bounce limit, no more light is generated
        return Color::new(0.0, 0.0, 0.0);
    }


    //t_min set to 0.001 because some rays will hit the object they're reflecting off 
    //at -0.0000001 or 0.00000001 or whatever floating point approximation the sphere intersector
    //gives us, rather than t = 0. Without the correction we get shadow acne where the 
    //shapes have black spots because hitting v.near 0 and then get highly absorbed.
    //i.e. ignore hits v. near 0
    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY){
        //Check if the point is occluded from all light sources
        let light_color =  match is_lit(rec.p, rec.normal, &world, &lights) {
            Some(color) => color,
            None => return Color::new(0.0, 0.0, 0.0)
        };


        //lambertian_hardcoded(&rec, world, depth)
        if let Some((attenuation, scattered)) = rec.mat.scatter(r.origin(), &lights, &world, r, &rec) {
            /*light_color * */ attenuation * ray_color(&scattered, &world, lights, depth-1)
        } else{
            Color::new(0.0, 0.0, 0.0)
        }
    }
    else{
        //Linearly blend white and blue depending on height of y coord after scaling ray 
        //direction to get a unit length (so -1.0 < y < 1.0)
        //Will be a horizontal gradient too because look at y component after normalizing
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() {
    const ASPECT_RATIO: f64 = 16.0/9.0;
    const IMAGE_WIDTH: u64 = 256;
    const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    const SAMPLES_PER_PIXEL: u64 = 100;
    const MAX_DEPTH: u64 = 50;

    //World
    let r: f64 = (std::f64::consts::PI / 4.0).cos();
    let mut world = World::new();
    
    //Lighting
    let mut lights = Lighting::new();

    //Hollow glass sphere:
    setup_hollow_sphere(&mut world, &mut lights);

    //Camera
    let lookfrom = Point3::new(0.0, 0.0, 0.0);
    let lookat = Point3::new(0.0, 0.0, -1.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let dist_to_focus = (lookfrom - lookat).length();
    let aperture = 0.0;


    let cam = Camera::new(lookfrom,
        lookat,
        vup,
        90.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
        );


    // //Image
    // const ASPECT_RATIO: f64 = 3.0/2.0;
    // const IMAGE_WIDTH: u64 = 256;
    // const IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    // const SAMPLES_PER_PIXEL: u64 = 500;
    // const MAX_DEPTH: u64 = 50;

    // //World
    // let world = random_scene();

    // //Camera
    // let lookfrom = Point3::new(13.0, 2.0, 3.0);
    // let lookat = Point3::new(0.0, 0.0, 0.0);
    // let vup = Vec3::new(0.0, 1.0, 0.0);
    // let dist_to_focus = 10.0;
    // let aperture = 0.1;

    // let cam = Camera::new(lookfrom,
    //     lookat,
    //     vup,
    //     20.0,
    //     ASPECT_RATIO,
    //     aperture,
    //     dist_to_focus);


    
    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    

    for j in (0..IMAGE_HEIGHT).rev() {
        
        eprintln!("Scanlines remaining: {}", j+1);
        stderr().flush().unwrap();

        let scanline: Vec<Color> =  (0..IMAGE_WIDTH).into_par_iter().map(|i| {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);

            for _ in 0..SAMPLES_PER_PIXEL {
                let mut rng = rand::thread_rng();
                let random_u: f64 = rng.gen();
                let random_v: f64 = rng.gen();

                let u = ((i as f64) + random_u) / ((IMAGE_WIDTH-1) as f64);
                let v = ((j as f64) + random_v) / ((IMAGE_HEIGHT-1) as f64);

                let r = cam.get_ray(u, v);

                pixel_color += ray_color(&r, &world, &lights, MAX_DEPTH);

            }

            pixel_color
        }).collect();

        for pixel_color in scanline {
            println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
    }
    eprint!("Done!");

}


fn setup_hollow_sphere(world: &mut World, lights: &mut Lighting) {
    let mat_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let mat_centre = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let mat_left = Arc::new(Dielectric::new(1.5, 1.0));//Metal::new(Color::new(0.8, 0.8, 0.8), 0.0));
    let mat_left_inner = Arc::new(Dielectric::new(1.5, 1.0));
    let mat_right = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2), 0.0));

    let mat_phong = Arc::new(PhongMat::new(
        1.0,
        1.0,
        0.0,
        0.5,
        4,
        Color::new(0.1, 0.2, 0.5),
        0.0,
        1.0,
        0.0,
    ));

    let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, mat_ground);
    let sphere_centre = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_centre);
    let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, mat_left);
    let sphere_left_inner = Sphere::new(Point3::new(-1.0, 0.0, -1.0), -0.4, mat_left_inner);
    let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, mat_right);

    let sphere_phong = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, mat_phong);

    let light_top = SimpleLight::new(Color::new(1.0, 1.0, 1.0), Color::new(1.0, 0.0, 0.0), Point3::new(0.0, 1.0, -1.0));
    let light_right = SimpleLight::new(Color::new(1.0, 1.0, 1.0), Color::new(1.0, 1.0, 1.0), Point3::new(2.0, 0.0, -1.0));

    world.push(Box::new(sphere_ground));
    world.push(Box::new(sphere_centre));
    //world.push(Box::new(sphere_left));
    //world.push(Box::new(sphere_left_inner));
    //world.push(Box::new(sphere_right));
    world.push(Box::new(sphere_phong));
    lights.push(Box::new(light_right));
    //lights.push(Box::new(light_top));
}

fn random_scene() -> World {
    let mut rng = rand::thread_rng();
    let mut world = World::new();

    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    let ground_sphere = Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground_mat);

    world.push(Box::new(ground_sphere));

    for a in -11..=11 {
        for b in -11..=11 {
            let choose_mat: f64 = rng.gen();
            let center = Point3::new((a as f64) + rng.gen_range(0.0..0.9),
                                     0.2,
                                     (b as f64) + rng.gen_range(0.0..0.9));

            if choose_mat < 0.8 {
                // Diffuse
                let albedo = Color::random(0.0..1.0) * Color::random(0.0..1.0);
                let sphere_mat = Arc::new(Lambertian::new(albedo));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else if choose_mat < 0.95 {
                // Metal
                let albedo = Color::random(0.4..1.0);
                let fuzz = rng.gen_range(0.0..0.5);
                let sphere_mat = Arc::new(Metal::new(albedo, fuzz));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            } else {
                // Glass
                let sphere_mat = Arc::new(Dielectric::new(1.5, 1.0));
                let sphere = Sphere::new(center, 0.2, sphere_mat);

                world.push(Box::new(sphere));
            }
        }
    }

    let mat1 = Arc::new(Dielectric::new(1.5, 1.0));
    let mat2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    let mat3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));

    let sphere1 = Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, mat1);
    let sphere2 = Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, mat2);
    let sphere3 = Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, mat3);

    world.push(Box::new(sphere1));
    world.push(Box::new(sphere2));
    world.push(Box::new(sphere3));

    world
}