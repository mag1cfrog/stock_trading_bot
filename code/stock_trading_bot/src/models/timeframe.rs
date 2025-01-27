use pyo3::{types::PyAnyMethods, Bound, BoundObject, FromPyObject, IntoPyObject, PyAny, Python};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimeFrameError {
    #[error("Invalid amount for {:?}: {}", unit, message)]
    InvalidAmount {
        unit: TimeFrameUnit,
        message: String,
    }
}

#[derive(Debug, Clone)]
pub enum TimeFrameUnit {
    Minute,
    Hour,
    Day,
    Week,
    Month
}

#[derive(Debug, Clone)]
pub struct TimeFrame {
    amount: u32,
    unit: TimeFrameUnit,
}

impl TimeFrame {
    pub fn new(amount: u32, unit: TimeFrameUnit) -> Result<Self, TimeFrameError> {
        Self::validate(amount, unit.clone())?;
        Ok(Self { amount, unit })
    }

    fn validate(amount: u32, unit: TimeFrameUnit) -> Result<(), TimeFrameError> {
        match unit {
            TimeFrameUnit::Minute if !(1..=59).contains(&amount) => Err(TimeFrameError::InvalidAmount {
                unit,
                message: "Second or Minute units can only be used with amounts between 1-59.".into(),
            }),
            TimeFrameUnit::Hour if !(1..=23).contains(&amount) => Err(TimeFrameError::InvalidAmount {
                unit,
                message: "Hour units can only be used with amounts 1-23".into(),
            }),
            TimeFrameUnit::Day | TimeFrameUnit::Week if amount != 1 => Err(TimeFrameError::InvalidAmount {
                unit,
                message: "Day and Week units can only be used with amount 1".into(),
            }),
            TimeFrameUnit::Month if ![1, 2, 3, 6, 12].contains(&amount) => Err(TimeFrameError::InvalidAmount {
                unit,
                message: "Month units can only be used with amount 1, 2, 3, 6 and 12".into(),
            }),
            _ => Ok(()),
        }
    }

    // Helper constructors 
    pub fn minutes(amounts: u32) -> Result<Self, TimeFrameError> {
        Self::new(amounts, TimeFrameUnit::Minute)
    }

    pub fn hours(amounts: u32) -> Result<Self, TimeFrameError> {
        Self::new(amounts, TimeFrameUnit::Hour)
    }

    pub fn day() -> Result<Self, TimeFrameError> {
        Self::new(1, TimeFrameUnit::Day)
    }

    pub fn week() -> Result<Self, TimeFrameError> {
        Self::new(1, TimeFrameUnit::Week)
    }

    pub fn months(amounts: u32) -> Result<Self, TimeFrameError> {
        Self::new(amounts, TimeFrameUnit::Month)
    }
    
}

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

impl<'py> IntoPyObject<'py> for TimeFrame {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = pyo3::PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let timeframe_cls = py
            .import("alpaca.data.timeframe")?
            .getattr("TimeFrame")?;

        let unit = self.unit.into_pyobject(py)?;
        timeframe_cls.call1((self.amount, unit))
    }
}

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
            _ => return Err(pyo3::exceptions::PyValueError::new_err(
                format!("Invalid TimeFrame unit {}", unit_str.as_str())
            )),
        };

        Ok(Self { amount, unit })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_valid_timeframe_creation() {
        assert!(TimeFrame::minutes(1).is_ok());
        assert!(TimeFrame::minutes(59).is_ok());
        assert!(TimeFrame::hours(1).is_ok());
        assert!(TimeFrame::hours(23).is_ok());
        assert!(TimeFrame::day().is_ok());
        assert!(TimeFrame::week().is_ok());
        assert!(TimeFrame::months(1).is_ok());
        assert!(TimeFrame::months(3).is_ok());
        assert!(TimeFrame::months(6).is_ok());
        assert!(TimeFrame::months(12).is_ok());
    }

    #[test]
    fn test_invalid_timeframe_creation() {
        // Minutes validation
        assert!(TimeFrame::minutes(0).is_err());
        assert!(TimeFrame::minutes(60).is_err());
        
        // Hours validation
        assert!(TimeFrame::hours(0).is_err());
        assert!(TimeFrame::hours(24).is_err());
        
        // Day/Week validation (only amount 1 allowed)
        assert!(TimeFrame::new(2, TimeFrameUnit::Day).is_err());
        assert!(TimeFrame::new(2, TimeFrameUnit::Week).is_err());
        
        // Months validation
        assert!(TimeFrame::months(4).is_err());
        assert!(TimeFrame::months(5).is_err());
        assert!(TimeFrame::months(7).is_err());
        assert!(TimeFrame::months(13).is_err());
    }

    #[test]
    fn test_timeframe_helper_constructors() {
        let minute_frame = TimeFrame::minutes(5).unwrap();
        let hour_frame = TimeFrame::hours(2).unwrap();
        let day_frame = TimeFrame::day().unwrap();
        let week_frame = TimeFrame::week().unwrap();
        let month_frame = TimeFrame::months(3).unwrap();

        // Test internal values
        match minute_frame {
            TimeFrame { amount, unit: TimeFrameUnit::Minute } => assert_eq!(amount, 5),
            _ => panic!("Unexpected TimeFrame structure"),
        }

        match hour_frame {
            TimeFrame { amount, unit: TimeFrameUnit::Hour } => assert_eq!(amount, 2),
            _ => panic!("Unexpected TimeFrame structure"),
        }

        match day_frame {
            TimeFrame { amount, unit: TimeFrameUnit::Day } => assert_eq!(amount, 1),
            _ => panic!("Unexpected TimeFrame structure"),
        }

        match week_frame {
            TimeFrame { amount, unit: TimeFrameUnit::Week } => assert_eq!(amount, 1),
            _ => panic!("Unexpected TimeFrame structure"),
        }

        match month_frame {
            TimeFrame { amount, unit: TimeFrameUnit::Month } => assert_eq!(amount, 3),
            _ => panic!("Unexpected TimeFrame structure"),
        }
    }

    mod python_conversion_tests {
        use super::*;
        use std::path::Path;
        use pyo3::Python;

        fn init_python() {
            // Initialize Python with venv
            pyo3::prepare_freethreaded_python();
            Python::with_gil(|py| {
                let venv_path = Path::new("python/venv");
                let sys = py.import("sys").unwrap();
                let path = sys.getattr("path").unwrap();
                path.call_method1(
                    "insert",
                    (0, venv_path.join("lib/python3.12/site-packages"))
                ).unwrap();
            });
        }

        #[test]
        fn test_timeframe_to_python() {
            init_python();
            Python::with_gil(|py| {
                let timeframe = TimeFrame::minutes(5).unwrap();
                let py_timeframe = timeframe.into_pyobject(py).unwrap();
                
                assert!(py_timeframe.call_method0("__str__").is_ok());
                
                // Check amount
                assert_eq!(
                    py_timeframe.getattr("amount_value").unwrap().extract::<u32>().unwrap(), 
                    5
                );
                
                // Check unit is Minute
                let unit = py_timeframe.getattr("unit_value").unwrap();
                assert_eq!(
                    unit.to_string(),
                    "TimeFrameUnit.Minute"
                );
                
                // Check string representation
                assert_eq!(
                    py_timeframe.call_method0("__str__").unwrap().extract::<String>().unwrap(),
                    "5Min"
                );
            });
        }

        #[test]
        fn test_timeframe_from_python() {
            init_python();
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
                    TimeFrame { amount, unit: TimeFrameUnit::Minute } => assert_eq!(amount, 5),
                    _ => panic!("Incorrect TimeFrame conversion from Python"),
                }
            });
        }
    }
}

