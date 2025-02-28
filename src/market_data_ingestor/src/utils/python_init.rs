use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use log::error;
use pyo3::PyErr;
use pyo3::Python;
use pyo3::exceptions::PyValueError;
use pyo3::types::PyAnyMethods;
use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Deserialize)]
pub struct Config {
    pub python_venv_path: String,
}

pub fn read_config(config_path: &str) -> Result<Config, Box<dyn Error + Send + Sync>> {
    let config_content = match fs::read_to_string(config_path) {
        Ok(content) => content,
        Err(e) => {
            error!("Failed to read config file: {:?}", e);
            return Err(e.into());
        }
    };

    let config: Config = match toml::from_str(&config_content) {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to parse config file: {:?}", e);
            return Err(e.into());
        }
    };

    Ok(config)
}

static INIT: OnceLock<Result<(), Box<dyn Error + Send + Sync>>> = OnceLock::new(); // <--- Track result

pub fn init_python(config_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read and parse the TOML config file.
    let config = read_config(config_path)?;

    let result = INIT.get_or_init(|| {
        let result = try_init_python(&config);
        if let Err(e) = &result {
            error!("Failed to initialize Python: {:?}", e);
        }
        result
    });

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e.as_ref().to_owned())),
    }
}

fn try_init_python(config: &Config) -> Result<(), Box<dyn Error + Send + Sync>> {
    // verify_shell_environment()?;
    // Initialize Python with venv
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let venv_path = Path::new(&config.python_venv_path);
        let lib_dir = venv_path.join("lib");

        // Set up virtual environment path first
        let site_packages_path = find_site_packages(&lib_dir)?;

        let sys = py.import("sys").expect("Cannot import sys module");

        let sys_path = sys.getattr("path").expect("Cannot get sys.path");

        sys_path
            .call_method1("insert", (0, site_packages_path))
            .expect("Failed to insert site-packages path");

        // Get environment variables in Rust
        // Verify environment variables exist in Rust process
        let api_key = std::env::var("APCA_API_KEY_ID").map_err(|e| {
            let msg = format!(
                "APCA_API_KEY_ID not found in environment. \
                Make sure to source your zsh config!\n\
                Original error: {}",
                e
            );
            PyErr::new::<PyValueError, _>(msg)
        })?;

        let secret_key = std::env::var("APCA_API_SECRET_KEY").map_err(|e| {
            let msg = format!(
                "APCA_API_SECRET_KEY not found in environment. \
                Did you reload your shell after adding to .zshenv?\n\
                Original error: {}",
                e
            );
            PyErr::new::<PyValueError, _>(msg)
        })?;

        // println!("env retrived by Rust.");
        // // Add this in try_init_python before setting Python environment variables:
        // println!("Rust ENV APCA_KEY: {:?}", std::env::var("APCA_API_KEY_ID"));
        // println!(
        //     "Rust ENV SECRET: {:?}",
        //     std::env::var("APCA_API_SECRET_KEY")
        // );
        // Set them in Python's environment
        let os = py.import("os")?;
        let environ = os.getattr("environ")?;
        environ.set_item("APCA_API_KEY_ID", api_key)?;
        environ.set_item("APCA_API_SECRET_KEY", secret_key)?;
        println!("env set to pyo3 instance.");

        // Prevent deadlock by importing modules upfront
        py.import("alpaca.data.timeframe").map_err(|e| {
            error!("Failed to import alpaca.data.timeframe: {:?}", e);
            e
        })?;
        py.import("alpaca.data.requests").map_err(|e| {
            error!("Failed to import alpaca.data.requests: {:?}", e);
            e
        })?;

        // Optionally check pydantic_core if needed
        // Update the pydantic_core import check
        py.import("pydantic_core").map_err(|e| {
            error!("Failed to import pydantic_core: {:?}", e);
            error!("Python path: {:?}", sys_path.call_method0("__str__"));
            e
        })?;

        Ok(())
    })
}

fn find_site_packages(lib_dir: &Path) -> Result<PathBuf, Box<dyn Error + Send + Sync>> {
    for entry in fs::read_dir(lib_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                if folder_name.starts_with("python") {
                    let candidate = path.join("site-packages");
                    if candidate.exists() {
                        return Ok(candidate);
                    }
                }
            }
        }
    }
    Err("No valid site-packages folder found".into())
}

pub fn verify_shell_environment() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Current environment variables:");
    for (k, v) in std::env::vars() {
        println!("- {}={}", k, v);
    }

    let required_vars = ["APCA_API_KEY_ID", "APCA_API_SECRET_KEY"];

    for var in required_vars {
        std::env::var(var).map_err(|_e| {
            format!(
                "Missing {} in environment.\n\
                TROUBLESHOOTING:\n\
                1. Ensure variables are exported in ~/.zshenv\n\
                2. Run 'source ~/.zshenv'\n\
                3. Verify with 'echo ${}'",
                var, var
            )
        })?;
    }

    Ok(())
}
