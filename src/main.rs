use clap::Parser;

use crate::cli::Cli;
use gltf_viewer::args::Args;
use gltf_viewer::run;

mod cli;

fn main() {
    let cli = Cli::parse();
    run(Args {
        gltf: Some(cli.gltf),
        ibl_environment: cli
            .ibl_environment
            .map(|ibl_environment| ibl_environment.into()),
    });
}
