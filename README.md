# glTF Viewer

This is a basic viewer for the glTF 3D asset format. This project was created primarily as a learning exercise and is a work in progress.

## Usage

The viewer is written in Rust and therefore built and run using cargo:

```
$ cargo run -- --help

A basic viewer for the glTF 3D asset format

Usage: gltf_viewer [OPTIONS] <GLTF>

Arguments:
  <GLTF>  Path to the .gltf file of the asset that will be displayed by the viewer

Options:
  -S, --skybox <SKYBOX>              Path to a .hdr file containing a panorama environment image that should be used to generate the skybox
  -d, --ibl-diffuse <IBL_DIFFUSE>    Path to a .ktx2 file containing an irradiance map for the given skybox
  -s, --ibl-specular <IBL_SPECULAR>  Path to a .ktx2 file containing a pre-filtered environment map for the given skybox
  -h, --help                         Print help
  -V, --version                      Print version
```

The viewer has one required argument which is a path to the .gltf file of the asset that should be displayed. Optionally, you can also provide environment files relevant to image-based lighting.

## Example

A collection of sample glTF assets has been released by the Khronos Group. They can be found here: https://github.com/KhronosGroup/glTF-Sample-Assets

To reference the .gltf file of a given asset from the above repository the following pattern can be used:

```
// glTF asset file
glTF-Sample-Assets/Models/<asset-name>/glTF/<asset-name>.gltf
```

The Khronos Group have also released a collection of environment files that can be used for image-based lighting. They can be found here: https://github.com/KhronosGroup/glTF-Sample-Environments

To reference the relevant files for a given environment from the above repository the following patterns can be used:

```
// Skybox panorama image
glTF-Sample-Environments/<environment-name>.hdr

// Irradiance map used for diffuse calculations in image-based lighting
glTF-Sample-Environments/<environment-name>/lambertian/diffuse.ktx2

// Pre-filtered environment map used for specular calculations in image-based lighting
glTF-Sample-Environments/<environment-name>/ggx/specular.ktx2
```

Below is an example command that demonstrates how these repositories can be used with the viewer. The command will run the viewer and display the `DamagedHelmet` glTF asset in the `field` environment using image-based lighting:

```
cargo run -- glTF-Sample-Assets/Models/DamagedHelmet/glTF/DamagedHelmet.gltf \
    --skybox glTF-Sample-Environments/field.hdr \
    --ibl-diffuse glTF-Sample-Environments/field/lambertian/diffuse.ktx2 \
    --ibl-specular glTF-Sample-Environments/field/ggx/specular.ktx2
```
