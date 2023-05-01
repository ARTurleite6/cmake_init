mod cmake_init;
mod constants;

use clap::Parser;
use cmake_init::App;

fn main() -> std::io::Result<()> {
    let args = App::parse();

    args.setup_project()?;
    return Ok(());
}
