#[cfg(feature = "alpaca-python-sdk")]
use pyo3::{Bound, BoundObject, FromPyObject, IntoPyObject, PyAny, Python, types::PyAnyMethods};
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

#[derive(Debug, Clone)]
pub enum TimeFrameUnit {
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone)]
pub struct TimeFrame {
    pub amount: u32,
    pub unit: TimeFrameUnit,
}

impl TimeFrame {
    pub fn new(amount: u32, unit: TimeFrameUnit) -> Self {
        // Self::validate(amount, unit.clone())?;
        // Ok(Self { amount, unit })
        Self { amount, unit }
    }

    // fn validate(amount: u32, unit: TimeFrameUnit) -> Result<(), TimeFrameError> {
    //     match unit {
    //         TimeFrameUnit::Minute if !(1..=59).contains(&amount) => {
    //             Err(TimeFrameError::InvalidAmount {
    //                 unit,
    //                 message: "Second or Minute units can only be used with amounts between 1-59."
    //                     .into(),
    //             })
    //         }
    //         TimeFrameUnit::Hour if !(1..=23).contains(&amount) => {
    //             Err(TimeFrameError::InvalidAmount {
    //                 unit,
    //                 message: "Hour units can only be used with amounts 1-23".into(),
    //             })
    //         }
    //         TimeFrameUnit::Day | TimeFrameUnit::Week if amount != 1 => {
    //             Err(TimeFrameError::InvalidAmount {
    //                 unit,
    //                 message: "Day and Week units can only be used with amount 1".into(),
    //             })
    //         }
    //         TimeFrameUnit::Month if ![1, 2, 3, 6, 12].contains(&amount) => {
    //             Err(TimeFrameError::InvalidAmount {
    //                 unit,
    //                 message: "Month units can only be used with amount 1, 2, 3, 6 and 12".into(),
    //             })
    //         }
    //         _ => Ok(()),
    //     }
    // }
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

#[cfg(all(test, feature = "alpaca-python-sdk"))]
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
                println!("Python version: {}", version);

                // Print PYTHONPATH
                let sys_path = sys.getattr("path").expect("Cannot get sys.path");
                println!("Python path: {}", sys_path.str().unwrap());

                // Try importing pydantic with more debug info
                match py.import("pydantic") {
                    Ok(_) => println!("Successfully imported pydantic"),
                    Err(e) => println!("Failed to import pydantic: {:?}", e),
                }

                // Try importing pydantic_core with more debug info
                match py.import("pydantic_core") {
                    Ok(_) => println!("Successfully imported pydantic_core"),
                    Err(e) => println!("Failed to import pydantic_core: {:?}", e),
                }

                let timeframe = TimeFrame::new(5, TimeFrameUnit::Minute).unwrap();
                let py_timeframe = timeframe
                    .into_pyobject(py)
                    .map_err(|e| {
                        error!("Failed to import pydantic_core: {:?}", e);
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

    mod timeframe_creation_tests {
        use super::*;

        #[test]
        fn test_valid_minute_timeframe() {
            let tf = TimeFrame::new(5, TimeFrameUnit::Minute);
            assert!(tf.is_ok());
            let tf = tf.unwrap();
            assert_eq!(tf.amount, 5);
            assert!(matches!(tf.unit, TimeFrameUnit::Minute));
        }

        #[test]
        fn test_valid_hour_timeframe() {
            let tf = TimeFrame::new(6, TimeFrameUnit::Hour);
            assert!(tf.is_ok());
            let tf = tf.unwrap();
            assert_eq!(tf.amount, 6);
            assert!(matches!(tf.unit, TimeFrameUnit::Hour));
        }

        #[test]
        fn test_valid_day_timeframe() {
            let tf = TimeFrame::new(1, TimeFrameUnit::Day);
            assert!(tf.is_ok());
        }

        #[test]
        fn test_valid_month_timeframes() {
            for amount in [1, 2, 3, 6, 12] {
                let tf = TimeFrame::new(amount, TimeFrameUnit::Month);
                assert!(tf.is_ok(), "Month with amount {} should be valid", amount);
            }
        }

        #[test]
        fn test_invalid_minute_timeframe() {
            assert!(TimeFrame::new(0, TimeFrameUnit::Minute).is_err());
            assert!(TimeFrame::new(60, TimeFrameUnit::Minute).is_err());
        }

        #[test]
        fn test_invalid_hour_timeframe() {
            assert!(TimeFrame::new(0, TimeFrameUnit::Hour).is_err());
            assert!(TimeFrame::new(24, TimeFrameUnit::Hour).is_err());
        }

        #[test]
        fn test_invalid_day_timeframe() {
            assert!(TimeFrame::new(2, TimeFrameUnit::Day).is_err());
        }

        #[test]
        fn test_invalid_week_timeframe() {
            assert!(TimeFrame::new(2, TimeFrameUnit::Week).is_err());
        }

        #[test]
        fn test_invalid_month_timeframe() {
            for amount in [0, 4, 5, 7, 8, 9, 10, 11, 13] {
                assert!(
                    TimeFrame::new(amount, TimeFrameUnit::Month).is_err(),
                    "Month with amount {} should be invalid",
                    amount
                );
            }
        }

        #[test]
        fn test_error_messages() {
            match TimeFrame::new(60, TimeFrameUnit::Minute) {
                Err(TimeFrameError::InvalidAmount { unit, message }) => {
                    assert!(matches!(unit, TimeFrameUnit::Minute));
                    assert!(message.contains("Second or Minute"));
                }
                _ => panic!("Expected InvalidAmount error"),
            }

            match TimeFrame::new(24, TimeFrameUnit::Hour) {
                Err(TimeFrameError::InvalidAmount { unit, message }) => {
                    assert!(matches!(unit, TimeFrameUnit::Hour));
                    assert!(message.contains("Hour units"));
                }
                _ => panic!("Expected InvalidAmount error"),
            }
        }
    }
}
