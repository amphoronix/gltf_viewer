pub struct Args {
    pub gltf: Option<String>,
    pub ibl_environment: Option<IblEnvironmentPaths>,
}

#[derive(Clone)]
pub struct IblEnvironmentPaths {
    pub skybox: String,
    pub diffuse: String,
    pub specular: String,
}
