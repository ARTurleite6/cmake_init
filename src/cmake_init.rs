use std::{
    io::{Error, ErrorKind, Write},
    path::Path,
    process::Command,
};

use clap::Parser;

use crate::constants::{CLANG_TIDY_CONTENT, CPP_VERSIONS, MAIN_CONTENT};

pub fn create_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(file_path).expect("Error creating file");

    file.write(content.as_bytes())?;

    return Ok(());
}

#[derive(Debug, Parser)]
pub struct App {
    #[clap(help("Project name"))]
    name: String,

    #[arg(
        long,
        short,
        default_value = "17",
        help("Standard C++ Version to be used")
    )]
    version: u8,
    #[arg(long, short, help("Support for clang-tidy"))]
    clang_tidy: bool,
}

impl App {
    pub fn setup_project(&self) -> std::io::Result<()> {
        if !self.validate_arguments() {
            println!("Error setting arguments");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Version of CPP is not valid",
            ));
        }

        self.create_project_directory()?;

        self.set_project_as_env()?;
        std::fs::create_dir("src")?;
        std::fs::create_dir("build")?;
        std::fs::create_dir("deps")?;

        create_file("deps/CMakeLists.txt", "")?;

        let content = self.get_cmake_file_content();
        create_file("CMakeLists.txt", &content)?;
        drop(content);

        create_file("src/main.cpp", MAIN_CONTENT)?;
        std::env::set_current_dir("build")?;

        Command::new("cmake")
            .arg("-DCMAKE_EXPORT_COMPILE_COMMANDS=ON")
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

        if self.clang_tidy {
            create_file(".clang-tidy", CLANG_TIDY_CONTENT)?;
        }
        Ok(())
    }

    pub fn create_project_directory(&self) -> std::io::Result<()> {
        let path = Path::new(&self.name);

        if path.is_dir() {
            println!(
                "Already exists an directory with this name, do you with to overwite it? [Y/n]"
            );
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

    pub fn set_project_as_env(&self) -> std::io::Result<()> {
        std::env::set_current_dir(&self.name)
    }

    pub fn get_cmake_file_content(&self) -> String {
        return format!(
            r#"cmake_minimum_required(VERSION 3.26.3)

project({0})

set(CMAKE_CXX_STANDARD {1})
set(CMAKE_CXX_FLAGS "${{CMAKE_CXX_FLAGS}} -Wall -Wextra -Werror -Wpedantic")

add_subdirectory(deps)
add_executable({0} src/main.cpp)"#,
            self.name, self.version
        );
    }

    pub fn validate_arguments(&self) -> bool {
        return CPP_VERSIONS.contains(&self.version);
    }
}
