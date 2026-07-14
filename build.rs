use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=src/wrapper.h");
    println!("cargo:rerun-if-changed=build.rs");

    let mut builder = bindgen::Builder::default().header("src/wrapper.h");

    if cfg!(feature = "vendor") {
        let include_dir = build_vendored_lexbor();
        builder = builder.clang_arg(format!("-I{}", include_dir.display()));
    } else {
        println!("cargo:rustc-link-lib=lexbor");
    }

    // libclang does not always discover GCC's builtin include directory (for
    // example, when libclang is installed without the clang driver). Lexbor's
    // headers eventually include standard C headers such as <stddef.h>, so
    // pass the compiler's include directory to bindgen explicitly when it is
    // available.
    builder = add_compiler_include_dir(builder);

    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn add_compiler_include_dir(builder: bindgen::Builder) -> bindgen::Builder {
    println!("cargo:rerun-if-env-changed=CC");

    let compiler = env::var_os("CC").unwrap_or_else(|| OsString::from("cc"));
    let output = match Command::new(&compiler)
        .arg("-print-file-name=include")
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return builder,
    };

    let include_dir = String::from_utf8_lossy(&output.stdout);
    let include_dir = Path::new(include_dir.trim());
    if include_dir.is_dir() {
        builder.clang_arg(format!("-I{}", include_dir.display()))
    } else {
        builder
    }
}

fn build_vendored_lexbor() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let source_dir = out_dir.join("lexbor-src");
    let build_dir = out_dir.join("lexbor-build");
    let install_dir = out_dir.join("lexbor-install");

    if !source_dir.exists() {
        run(Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg("https://github.com/lexbor/lexbor.git")
            .arg(&source_dir));
    }

    run(Command::new("cmake")
        .arg("-S")
        .arg(&source_dir)
        .arg("-B")
        .arg(&build_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", install_dir.display()))
        .arg("-DLEXBOR_BUILD_SHARED=OFF")
        .arg("-DLEXBOR_BUILD_STATIC=ON")
        .arg("-DLEXBOR_BUILD_EXAMPLES=OFF")
        .arg("-DLEXBOR_BUILD_TESTS=OFF"));
    run(Command::new("cmake")
        .arg("--build")
        .arg(&build_dir)
        .arg("--target")
        .arg("install"));

    let lib_dir = find_lib_dir(&install_dir);
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    link_vendored_library(&lib_dir);

    let include_dir = install_dir.join("include");
    if !include_dir.exists() {
        panic!(
            "vendored lexbor install did not create include directory at {}",
            include_dir.display()
        );
    }

    include_dir
}

fn find_lib_dir(install_dir: &Path) -> PathBuf {
    for name in ["lib", "lib64"] {
        let lib_dir = install_dir.join(name);
        if lib_dir.exists() {
            return lib_dir;
        }
    }

    panic!(
        "vendored lexbor install did not create a lib directory under {}",
        install_dir.display()
    );
}

fn link_vendored_library(lib_dir: &Path) {
    let entries = fs::read_dir(lib_dir)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", lib_dir.display()));
    let libraries: Vec<String> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    if libraries.iter().any(|name| name == "liblexbor_static.a") {
        println!("cargo:rustc-link-lib=static=lexbor_static");
    } else if libraries.iter().any(|name| name == "liblexbor.a") {
        println!("cargo:rustc-link-lib=static=lexbor");
    } else if libraries
        .iter()
        .any(|name| name == "liblexbor.dylib" || name == "liblexbor.so")
    {
        println!("cargo:rustc-link-lib=lexbor");
    } else {
        panic!(
            "vendored lexbor install did not produce a recognizable lexbor library in {}",
            lib_dir.display()
        );
    }
}

fn run(command: &mut Command) {
    let program = command.get_program().to_string_lossy().into_owned();
    let status = command.status().unwrap_or_else(|err| {
        panic!(
            "failed to run {:?}: {err}. Make sure `{program}` is installed and available on PATH.",
            command
        )
    });

    if !status.success() {
        panic!("{:?} failed with status {status}", command);
    }
}
