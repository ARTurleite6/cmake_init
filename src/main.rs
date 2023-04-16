use std::{
    io::{Error, ErrorKind, Write},
    path::Path,
    process::Command,
};

use clap::Parser;

const CPP_VERSIONS: [u8; 5] = [11, 14, 17, 20, 23];

#[derive(Debug, Parser)]
struct App {
    name: String,

    #[arg(long, short, default_value = "20")]
    version: u8,
}

fn create_directory(directory_name: &str) -> std::io::Result<()> {
    let path = Path::new(directory_name);

    if path.is_dir() {
        println!("Already exists an directory with this name, do you with to overwite it? [Y/n]");
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;

        let choice = buffer.trim_end().to_lowercase();
        dbg!(&choice);

        if choice == "n" {
            println!("Project not created then...");
            return Err(Error::new(std::io::ErrorKind::Other, "Negative Response"));
        } else {
            std::fs::remove_dir_all(path)?;
        }
    }

    return std::fs::create_dir(path);
}

fn create_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(file_path).expect("Error creating file");

    file.write(content.as_bytes())?;

    return Ok(());
}

fn get_cmake_file(project_name: &str, cpp_version: u8) -> String {
    return format!(
        r#"cmake_minimum_required(VERSION 3.26.3)

project({0})

set(CMAKE_CXX_STANDARD {1})
add_executable({0} src/main.cpp)"#,
        project_name, cpp_version
    );
}

fn validate_arguments(args: &App) -> bool {
    return CPP_VERSIONS.contains(&args.version);
}

fn main() -> std::io::Result<()> {
    const MAIN_CONTENT: &str = r#"#include <iostream>

    int main() {
        std::cout << "Hello World\n";
        return 0;
    }"#;

    let args = App::parse();

    if !validate_arguments(&args) {
        println!("Error setting arguments");
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Version of CPP is not valid",
        ));
    }

    println!("{:?}", args);

    if let Err(error) = create_directory(&args.name) {
        println!("{:?}", error);
        return Err(error);
    }
    println!("Project created");

    std::env::set_current_dir(&args.name).expect("Couldn't go to project directory");

    std::fs::create_dir("src")?;
    std::fs::create_dir("build-debug")?;

    let content = get_cmake_file(&args.name, args.version);
    if let Err(error) = create_file("CMakeLists.txt", &content) {
        println!("{}", error);
        return Err(error);
    }
    drop(content);

    if let Err(error) = create_file("src/main.cpp", MAIN_CONTENT) {
        println!("{:?}", error);
        return Err(error);
    }

    std::env::set_current_dir("build-debug").expect("Couldn't go to build directory");

    Command::new("cmake")
        .arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=TRUE")
        .arg("-DCMAKE_BUILD_TYPE=Debug")
        .arg("..")
        .output()
        .expect("Error executing cmake command");

    std::env::set_current_dir("..").expect("Couldn't go to root directory");

    Command::new("ln")
        .arg("-s")
        .arg("build-debug/compile_commands.json")
        .output()
        .expect("Error linking to compile_commands");

    return Ok(());
}
