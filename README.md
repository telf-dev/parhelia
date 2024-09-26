
This is a small raytracer based on Raytracing in One Weekend, mainly for practising Rust 
and learning computer graphics stuff.

## Features:

- Diffuse, specular, dielectric (water, glass etc), phongian (matt materials with a 'shine', 
think ceramic cups, bathroom tiles etc) scattering
- Lighting - uses 3D objects which emit light, rather than a point source
- Camera with adjustable placement, lens blur, focal length (to image)


## Usage
From the top directory, compile with `cargo build -r`. Run with `./release/raytracer [config filepath] [scene number] > image.ppm`. Load the output in a ppm viewer, e.g. [here](https://www.cs.rhodes.edu/welshc/COMP141_F16/ppmReader.html)

Scene number may be set to 1 or 2; 1 displays a specular, diffuse and dielectric example, 2 displays a phongian example. Alternatively, alter the scene code!

The position, focus and field of view of the camera, as well as the number of samples per pixel and the max ray scatters, are read from a given JSON file - 
./config/hires.json and ./config/lowres.json are included in the repo. More samples results in higher granularity in the final image, more scattering
produces more realistic lighting and reflections. Note that the `as_w` and `as_h` fields in the config are used to calculate the aspect ratio, e.g. setting them to 16.0 
and 9.0 respectively produces an aspect ratio of 16:9.

## TODO:

- CUDA - would be fun (and much faster) to convert to GPU-based rendering
- Camera and object movement
- Phongian reflection fixes (reflected ray from a non-light-emitting material is too dark)

![Not your average bowling alley](https://github.com/telf-dev/parhelia/blob/main/examples/super_high_res.png?raw=true)
