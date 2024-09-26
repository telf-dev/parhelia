use rand::Rng;
use rayon::prelude::*;

use std::io::{stderr, Write};
use std::sync::Arc;
use std::env;
use std::path::Path;



mod camera;
mod config;
mod hit;
mod light;
mod material;
mod ray;
mod sphere;
mod vec3;

use camera::Camera;
use config::Config;
use vec3::{Vec3, Point3, Color};
use ray::Ray;
use material::{Material, DiffuseLight, Dielectric, Diffuse, Specular, Phongian};
use sphere::Sphere;
use hit::{Hit, World};


fn ray_color(r: &Ray, world: &World, depth: u64, vpos: Vec3) -> Color {
    if depth <= 0{
        //Exceeded ray bounce limit, no more light is generated
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(rec) = world.hit(r, 0.001, f64::INFINITY){
        // Light emitted by hit object (none if not a light)
        let color_emitted = rec.mat.emitted();

        //Check if the scattering absorbs the ray into the object or not
        
        //Not absorbed
        if let Some((attenuation, scattered)) = rec.mat.scatter(vpos, &world, r, &rec) {
            return color_emitted + attenuation * ray_color(&scattered, &world, depth-1, vpos)
        }
        //Absorbed 
        return color_emitted
    }
    else{
        // If no hit, display sky - Linearly blend white and blue depending on height of y 
        // coord after scaling ray direction.
        // Can alter this as needed, e.g. black if only want illumination from light sources
        let unit_direction = r.direction().normalized();
        let t = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
    }
}

fn main() -> () {

    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("Program use: ./raytracer [config_path] [scene_type]");
    }
    let config = Config::read_config(Path::new(&args[1])).unwrap();

    // Config settings; max depth is the max collisions a ray can have with objects.
    // More collisions -> better reflections etc
    let ASPECT_RATIO: f64 = config.as_w/config.as_h;
    let IMAGE_WIDTH: u64 = config.im_width;
    let IMAGE_HEIGHT: u64 = ((IMAGE_WIDTH as f64) / ASPECT_RATIO) as u64;
    let SAMPLES_PER_PIXEL: u64 = config.samples_per_pixel;
    let MAX_DEPTH: u64 = config.max_depth;
 
    

    //Camera position
    let lookfrom = config.lookfrom;
    //Point the camera's looking at
    let lookat = config.lookat;
    //Vector directly up (regardless of camera tilt)
    let vup = config.vup;
    let dist_to_focus = (lookfrom - lookat).length();
    //Aperture width
    let aperture = config.aperture;
    //vertical FOV
    let vfov = config.vfov;

    let cam = Camera::new(
        lookfrom,
        lookat,
        vup,
        vfov,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

    
    let r: f64 = (std::f64::consts::PI / 4.0).cos();
   
    //World - set up some objects to display
    let mut world = match args[2].parse::<i32>().expect("Scene argument should be a number!") {
        1 => simple_scene(),
        2 => phongian_scene(),
        _ => {
            panic!("Scene selection should be one of [1, 2]!");
                
        },
    };
    //random spheres:
    //let mut world = random_scene();

    //PPM metadata
    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");

    rayon::ThreadPoolBuilder::new().num_threads(10).build_global().unwrap();
    
    // Main rendering loop; scan across each row of the image, emit $SAMPLES_PER_PIXEL rays for
    // each pixel from the camera
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

                pixel_color += ray_color(&r, &world, MAX_DEPTH, lookfrom);

            }

            pixel_color
        }).collect();

        for pixel_color in scanline {
            println!("{}", pixel_color.format_color(SAMPLES_PER_PIXEL));
        }
    }
    eprint!("Done!");
}


fn simple_scene() -> World {
    let mut world = World::new();

    // Ground
    let mat_ground: Arc<dyn Material> = Arc::new(Diffuse::new(Color::new(0.8, 0.8, 0.0), 1.0));
    let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, Arc::clone(&mat_ground));
    
    //Left sphere is almost perfectly reflective, with some fuzz - e.g. polished metal.
    let mat_left:  Arc<dyn Material> = Arc::new(Specular::new(Color::new(0.8, 0.8, 0.8), 0.2, 1.0));
    let sphere_left = Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, Arc::clone(&mat_left));

    // Central sphere - diffuse purple sphere
    let mat_centre: Arc<dyn Material> = Arc::new(Diffuse::new(Color::new(0.2, 0.0, 1.0), 1.0));
    let sphere_centre = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, Arc::clone(&mat_centre));
    
    // Hollow glass sphere: two dielectric spheres with a refractive index of glass and air for the outer/inner
    // sphere respectively
    // Interesting properties!
    let mat_right: Arc<dyn Material> = Arc::new(Dielectric::new(1.5, 1.0));
    let mat_right_inner: Arc<dyn Material> = Arc::new(Dielectric::new(1.0/1.5, 1.0));
    let sphere_right = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, Arc::clone(&mat_right));
    let sphere_right_inner = Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.4, Arc::clone(&mat_right_inner));
    
    // Light source
    let mat_light: Arc<dyn Material> = Arc::new(DiffuseLight::new(Color::new(1.0, 1.0, 1.0), 1.0, Color::new(50.0,50.0,50.0)));
    let light_top = Sphere::new(Point3::new(0.0, 1.0, -1.0), 0.25, Arc::clone(&mat_light));
    
    world.push(Box::new(sphere_ground));
    world.push(Box::new(sphere_centre));
    world.push(Box::new(sphere_left));
    world.push(Box::new(sphere_right));
    world.push(Box::new(sphere_right_inner));
    world.push(Box::new(light_top));

    world
}

fn phongian_scene() -> World {
    let mut world = World::new();
    let mat_ground: Arc<dyn Material> = Arc::new(Diffuse::new(Color::new(0.8, 0.8, 0.0), 1.0));
    let sphere_ground = Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, Arc::clone(&mat_ground));
    let mat_light: Arc<dyn Material> = Arc::new(DiffuseLight::new(Color::new(1.0, 1.0, 1.0), 1.0, Color::new(50.0,50.0,50.0)));
    let light_top = Sphere::new(Point3::new(0.0, 1.0, -1.0), 0.25, Arc::clone(&mat_light));
    let light_right = Sphere::new(Point3::new(2.0, 0.0, -1.0), 0.25, Arc::clone(&mat_light));

    // Phongian material; Matt with some degree of shine, e.g. polished ceramics - get spots of light at certain angles.
    // TODO: Not quite physically accurate - too dark in diffuse areas.
    let mat_centre: Arc<dyn Material> = Arc::new(
        Phongian::new(
                      Color::new(0.2, 0.0, 1.0), 
                      0.2,
                      1.0,
                      1.0,
                      0.1,
                  )
              );
    let sphere_centre = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, Arc::clone(&mat_centre));

    world.push(Box::new(sphere_ground));
    world.push(Box::new(sphere_centre));
    world.push(Box::new(light_top));
    world.push(Box::new(light_right));

    world
}
