use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Fast exit if feature not enabled (Cargo exposes features as CARGO_FEATURE_* env vars)
    let python_feature_enabled = std::env::var("CARGO_FEATURE_ALPACA_PYTHON_SDK").is_ok();
    if !python_feature_enabled {
        println!("cargo:warning=alpaca-python-sdk feature not enabled; skipping Python setup.");
        return;
    }

    // Allow CI / users to force skip (set in workflow: MARKET_DATA_INGESTOR_SKIP_PYTHON_SETUP=1)
    if std::env::var("MARKET_DATA_INGESTOR_SKIP_PYTHON_SETUP").is_ok()
        || std::env::var("CI").is_ok()
    {
        println!(
            "cargo:warning=Skipping Python setup (MARKET_DATA_INGESTOR_SKIP_PYTHON_SETUP or CI set)."
        );
        return;
    }

    // 1. Try to find 'uv'; if absent, warn and skip instead of panic.
    let uv_ok = Command::new("uv")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !uv_ok {
        println!(
            "cargo:warning='uv' not found; skipping virtualenv creation (set it up manually if needed)."
        );
        return;
    }

    // --- 2. Set up the Python virtual environment if it doesn't exist ---
    // The path is relative to the crate root where this build.rs script lives.
    let python_project_path = Path::new("../../python");
    let venv_path = python_project_path.join(".venv");

    if !venv_path.exists() {
        println!("cargo:warning=Python virtual environment not found. Creating it with 'uv'...");

        // Create the virtual environment using `uv init`.
        let venv_status = Command::new("uv")
            .arg("venv")
            .arg(&venv_path)
            .status()
            .expect("Failed to execute 'uv venv'.");

        if !venv_status.success() {
            panic!("'uv venv' command failed. Could not create the virtual environment.");
        }

        // Install dependencies into the new virtual environment.
        let pip_status = Command::new("uv")
            .arg("pip")
            .arg("install")
            .arg("alpaca-py")
            .arg("polars[pandas]") // Needed for pl.from_pandas()
            .current_dir(python_project_path)
            .status()
            .expect("Failed to execute 'uv pip install'.");

        if !pip_status.success() {
            panic!("'uv pip install' failed. Could not install Python dependencies.");
        }
        println!(
            "cargo:warning=Python virtual environment created and dependencies installed successfully."
        );
    }

    // --- 3. Configure PyO3 to use the virtual environment's Python interpreter ---
    let python_executable = if cfg!(windows) {
        venv_path.join("Scripts").join("python.exe")
    } else {
        venv_path.join("bin").join("python")
    };

    if python_executable.exists() {
        unsafe {
            env::set_var("PYO3_PYTHON", python_executable.to_str().unwrap());
        }
    } else {
        panic!("Could not find Python executable in virtual environment at: {python_executable:?}",);
    }

    // --- 4. Let pyo3-build-config handle the rest ---
    // It inspects the interpreter from PYO3_PYTHON and emits the correct linker flags.
    pyo3_build_config::use_pyo3_cfgs();

    // Rerun this script only if it changes.
    println!("cargo:rerun-if-changed=build.rs");
}
