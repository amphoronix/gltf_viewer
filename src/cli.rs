use clap::{Args, Parser};

/// A basic viewer for the glTF 3D asset format
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to the .gltf file of the asset that will be displayed by the viewer
    pub gltf: String,

    #[command(flatten)]
    pub ibl_environment: Option<IblEnvironment>,
}

#[derive(Args, Debug, Clone)]
#[
    group(
        required = false,
        requires_all = [
            "skybox",
            "ibl_diffuse",
            "ibl_specular",
        ]
    )
]
pub struct IblEnvironment {
    /// Path to a .hdr file containing a panorama environment image that should be used to generate the skybox
    #[arg(short = 'S', long, required = false)]
    pub skybox: String,

    /// Path to a .ktx2 file containing an irradiance map for the given skybox
    #[arg(short = 'd', long, required = false)]
    pub ibl_diffuse: String,

    /// Path to a .ktx2 file containing a pre-filtered environment map for the given skybox
    #[arg(short = 's', long, required = false)]
    pub ibl_specular: String,
}

impl From<IblEnvironment> for gltf_viewer::args::IblEnvironmentPaths {
    fn from(value: IblEnvironment) -> Self {
        gltf_viewer::args::IblEnvironmentPaths {
            skybox: value.skybox,
            diffuse: value.ibl_diffuse,
            specular: value.ibl_specular,
        }
    }
}
