#[cfg(feature = "alpaca-python-sdk")]
use pyo3::{Bound, BoundObject, FromPyObject, IntoPyObject, PyAny, Python, types::PyAnyMethods};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeFrameError {
    #[error("Invalid amount for {:?}: {}", unit, message)]
    InvalidAmount {
        unit: TimeFrameUnit,
        message: String,
    },

    #[error("Invalid input: {}", message)]
    InvalidInput { message: String },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TimeFrameUnit {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimeFrame {
    pub amount: u32,
    pub unit: TimeFrameUnit,
}

impl TimeFrame {
    pub fn new(amount: u32, unit: TimeFrameUnit) -> Self {
        Self { amount, unit }
    }
}

#[cfg(feature = "alpaca-python-sdk")]
impl<'py> IntoPyObject<'py> for TimeFrameUnit {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let timeframe_mod = py.import("alpaca.data.timeframe")?;
        let unit_enum = timeframe_mod.getattr("TimeFrameUnit")?;
        match self {
            TimeFrameUnit::Minute => Ok(unit_enum.getattr("Minute")?.into_bound()),
            TimeFrameUnit::Hour => Ok(unit_enum.getattr("Hour")?.into_bound()),
            TimeFrameUnit::Day => Ok(unit_enum.getattr("Day")?.into_bound()),
            TimeFrameUnit::Week => Ok(unit_enum.getattr("Week")?.into_bound()),
            TimeFrameUnit::Month => Ok(unit_enum.getattr("Month")?.into_bound()),
        }
    }
}

#[cfg(feature = "alpaca-python-sdk")]
impl<'py> IntoPyObject<'py> for TimeFrame {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let timeframe_cls = py.import("alpaca.data.timeframe")?.getattr("TimeFrame")?;

        let unit = self.unit.into_pyobject(py)?;
        timeframe_cls.call1((self.amount, unit))
    }
}

#[cfg(feature = "alpaca-python-sdk")]
impl<'source> FromPyObject<'source> for TimeFrame {
    fn extract_bound(ob: &Bound<'source, PyAny>) -> pyo3::PyResult<Self> {
        let amount: u32 = ob.getattr("amount")?.extract()?;
        // The Python TimeFrameUnit has a 'value' property that gives us the string representation
        let unit_str: String = ob.getattr("unit_value")?.getattr("value")?.extract()?;

        let unit = match unit_str.as_str() {
            "Min" => TimeFrameUnit::Minute,
            "Hour" => TimeFrameUnit::Hour,
            "Day" => TimeFrameUnit::Day,
            "Week" => TimeFrameUnit::Week,
            "Month" => TimeFrameUnit::Month,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Invalid TimeFrame unit {}",
                    unit_str.as_str()
                )));
            }
        };

        Ok(Self { amount, unit })
    }
}

#[cfg(all(test, feature = "alpaca-python-sdk"))]
mod test {

    use super::*;
    use log::error;

    mod python_conversion_tests {
        use super::*;
        use pyo3::Python;
        use serial_test::serial;

        use crate::utils::init_python;

        const CONFIG_PATH: &str =
            "/home/hanbo/repo/stock_trading_bot/src/configs/data_ingestor.toml";

        #[test]
        #[serial]
        #[ignore]
        fn test_timeframe_to_python() {
            init_python(CONFIG_PATH).unwrap();
            Python::with_gil(|py| {
                // Print Python version
                let sys = py.import("sys").expect("Cannot import sys module");
                let version = sys.getattr("version").expect("Cannot get Python version");
                println!("Python version: {version}");

                // Print PYTHONPATH
                let sys_path = sys.getattr("path").expect("Cannot get sys.path");
                println!("Python path: {}", sys_path.str().unwrap());

                // Try importing pydantic with more debug info
                match py.import("pydantic") {
                    Ok(_) => println!("Successfully imported pydantic"),
                    Err(e) => println!("Failed to import pydantic: {e}"),
                }

                // Try importing pydantic_core with more debug info
                match py.import("pydantic_core") {
                    Ok(_) => println!("Successfully imported pydantic_core"),
                    Err(e) => println!("Failed to import pydantic_core: {e}"),
                }

                let timeframe = TimeFrame::new(5, TimeFrameUnit::Minute);
                let py_timeframe = timeframe
                    .into_pyobject(py)
                    .map_err(|e| {
                        error!("Failed to import pydantic_core: {e}");
                        e
                    })
                    .unwrap();

                assert!(py_timeframe.call_method0("__str__").is_ok());

                // Check amount
                assert_eq!(
                    py_timeframe
                        .getattr("amount_value")
                        .unwrap()
                        .extract::<u32>()
                        .unwrap(),
                    5
                );

                // Check unit is Minute
                let unit = py_timeframe.getattr("unit_value").unwrap();
                assert_eq!(unit.to_string(), "TimeFrameUnit.Minute");

                // Check string representation
                assert_eq!(
                    py_timeframe
                        .call_method0("__str__")
                        .unwrap()
                        .extract::<String>()
                        .unwrap(),
                    "5Min"
                );
            });
        }

        #[test]
        #[serial]
        #[ignore]
        fn test_timeframe_from_python() {
            init_python(CONFIG_PATH).unwrap();
            Python::with_gil(|py| {
                // Create a Python TimeFrame object
                let timeframe_mod = py.import("alpaca.data.timeframe").unwrap();
                let timeframe_unit = timeframe_mod.getattr("TimeFrameUnit").unwrap();
                let minute_cls = timeframe_unit.getattr("Minute").unwrap();
                let py_timeframe = timeframe_mod
                    .getattr("TimeFrame")
                    .unwrap()
                    .call1((5, minute_cls))
                    .unwrap();

                // Convert it to Rust
                let rust_timeframe: TimeFrame = py_timeframe.extract().unwrap();

                // Verify the conversion
                match rust_timeframe {
                    TimeFrame {
                        amount,
                        unit: TimeFrameUnit::Minute,
                    } => assert_eq!(amount, 5),
                    _ => panic!("Incorrect TimeFrame conversion from Python"),
                }
            });
        }
    }
}
