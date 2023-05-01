use std::{
    io::{Error, ErrorKind, Write},
    path::Path,
    process::Command,
};

use clap::Parser;

const CPP_VERSIONS: [u8; 4] = [11, 14, 17, 20];

const CLANG_TIDY_CONTENT: &'static str = r#"
---
Checks: '-*,
bugprone-argument-comment,
bugprone-assert-side-effect,
bugprone-bad-signal-to-kill-thread,
bugprone-branch-clone,
bugprone-copy-constructor-init,
bugprone-dangling-handle,
bugprone-dynamic-static-initializers,
bugprone-fold-init-type,
bugprone-forward-declaration-namespace,
bugprone-forwarding-reference-overload,
bugprone-inaccurate-erase,
bugprone-incorrect-roundings,
bugprone-integer-division,
bugprone-lambda-function-name,
bugprone-macro-parentheses,
bugprone-macro-repeated-side-effects,
bugprone-misplaced-operator-in-strlen-in-alloc,
bugprone-misplaced-pointer-arithmetic-in-alloc,
bugprone-misplaced-widening-cast,
bugprone-move-forwarding-reference,
bugprone-multiple-statement-macro,
bugprone-no-escape,
bugprone-not-null-terminated-result,
bugprone-parent-virtual-call,
bugprone-posix-return,
bugprone-reserved-identifier,
bugprone-sizeof-container,
bugprone-sizeof-expression,
bugprone-spuriously-wake-up-functions,
bugprone-string-constructor,
bugprone-string-integer-assignment,
bugprone-string-literal-with-embedded-nul,
bugprone-suspicious-enum-usage,
bugprone-suspicious-include,
bugprone-suspicious-memory-comparison,
bugprone-suspicious-memset-usage,
bugprone-suspicious-missing-comma,
bugprone-suspicious-semicolon,
bugprone-suspicious-string-compare,
bugprone-swapped-arguments,
bugprone-terminating-continue,
bugprone-throw-keyword-missing,
bugprone-too-small-loop-variable,
bugprone-undefined-memory-manipulation,
bugprone-undelegated-constructor,
bugprone-unhandled-self-assignment,
bugprone-unused-raii,
bugprone-unused-return-value,
bugprone-use-after-move,
bugprone-virtual-near-miss,
cert-dcl21-cpp,
cert-dcl58-cpp,
cert-err34-c,
cert-err52-cpp,
cert-err60-cpp,
cert-flp30-c,
cert-msc50-cpp,
cert-msc51-cpp,
cert-str34-c,
cppcoreguidelines-interfaces-global-init,
cppcoreguidelines-narrowing-conversions,
cppcoreguidelines-pro-type-member-init,
cppcoreguidelines-pro-type-static-cast-downcast,
cppcoreguidelines-slicing,
google-default-arguments,
google-explicit-constructor,
google-runtime-operator,
hicpp-exception-baseclass,
hicpp-multiway-paths-covered,
misc-misplaced-const,
misc-new-delete-overloads,
misc-no-recursion,
misc-non-copyable-objects,
misc-throw-by-value-catch-by-reference,
misc-unconventional-assign-operator,
misc-uniqueptr-reset-release,
modernize-avoid-bind,
modernize-concat-nested-namespaces,
modernize-deprecated-headers,
modernize-deprecated-ios-base-aliases,
modernize-loop-convert,
modernize-make-shared,
modernize-make-unique,
modernize-pass-by-value,
modernize-raw-string-literal,
modernize-redundant-void-arg,
modernize-replace-auto-ptr,
modernize-replace-disallow-copy-and-assign-macro,
modernize-replace-random-shuffle,
modernize-return-braced-init-list,
modernize-shrink-to-fit,
modernize-unary-static-assert,
modernize-use-auto,
modernize-use-bool-literals,
modernize-use-emplace,
modernize-use-equals-default,
modernize-use-equals-delete,
modernize-use-nodiscard,
modernize-use-noexcept,
modernize-use-nullptr,
modernize-use-override,
modernize-use-transparent-functors,
modernize-use-uncaught-exceptions,
mpi-buffer-deref,
mpi-type-mismatch,
openmp-use-default-none,
performance-faster-string-find,
performance-for-range-copy,
performance-implicit-conversion-in-loop,
performance-inefficient-algorithm,
performance-inefficient-string-concatenation,
performance-inefficient-vector-operation,
performance-move-const-arg,
performance-move-constructor-init,
performance-no-automatic-move,
performance-noexcept-move-constructor,
performance-trivially-destructible,
performance-type-promotion-in-math-fn,
performance-unnecessary-copy-initialization,
performance-unnecessary-value-param,
portability-simd-intrinsics,
readability-avoid-const-params-in-decls,
readability-const-return-type,
readability-container-size-empty,
readability-convert-member-functions-to-static,
readability-delete-null-pointer,
readability-deleted-default,
readability-inconsistent-declaration-parameter-name,
readability-make-member-function-const,
readability-misleading-indentation,
readability-misplaced-array-index,
readability-non-const-parameter,
readability-redundant-control-flow,
readability-redundant-declaration,
readability-redundant-function-ptr-dereference,
readability-redundant-smartptr-get,
readability-redundant-string-cstr,
readability-redundant-string-init,
readability-simplify-subscript-expr,
readability-static-accessed-through-instance,
readability-static-definition-in-anonymous-namespace,
readability-string-compare,
readability-uniqueptr-delete-release,
readability-use-anyofallof'
"#;

#[derive(Debug, Parser)]
struct App {
    name: String,

    #[arg(long, short, default_value = "17")]
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
set(CMAKE_CXX_FLAGS "${{CMAKE_CXX_FLAGS\}} -Wall -Wextra -Werror -Wpedantric")
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

    if let Err(error) = create_file(".clang-tidy", CLANG_TIDY_CONTENT) {
        println!("{}", error);
        return Err(error);
    }

    return Ok(());
}
