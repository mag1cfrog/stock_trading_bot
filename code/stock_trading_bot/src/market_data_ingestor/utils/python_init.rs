use std::path::Path;
use std::sync::Once;
use pyo3::Python;
use pyo3::types::PyAnyMethods;

static INIT: Once = Once::new();

pub fn init_python() {
    INIT.call_once(|| {
        // Initialize Python with venv
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {

            let venv_path = Path::new("python/venv");
            let sys = py.import("sys").unwrap();
            let path = sys.getattr("path").unwrap();
            path.call_method1(
                "insert",
                (0, venv_path.join("lib/python3.12/site-packages")),
            )
            .unwrap();
        
            // Prevent deadlock by importing modules upfront
            py.import("alpaca.data.timeframe").unwrap();
            py.import("alpaca.data.requests").unwrap();
        });
    });
}