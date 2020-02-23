use rocket::response::Responder;
use rocket_contrib::json::{JsonError, JsonValue};
use serde_derive::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

pub enum ErrorType {
    OutOfBounds(String),
    NoExist(String),
    FunctionError(String),
    NoConvergence(),
    ValueAtRiskError(String),
    JsonError(String),
}
#[derive(Debug, PartialEq, Responder, Serialize)]
#[response(status = 400, content_type = "json")]
pub struct ParameterError {
    msg: JsonValue,
}

impl ParameterError {
    pub fn new(error_type: &ErrorType) -> Self {
        ParameterError {
            msg: json!({"err":match error_type {
                ErrorType::OutOfBounds(parameter) => {
                    format!("Parameter {} out of bounds.", parameter)
                }
                ErrorType::NoExist(parameter) => format!("Parameter {} does not exist.", parameter),
                ErrorType::FunctionError(parameter) => {
                    format!("Function indicator {} does not exist.", parameter)
                }
                ErrorType::NoConvergence() => format!("Root does not exist for implied volatility"),
                ErrorType::ValueAtRiskError(message) => format!("{}", message),
                ErrorType::JsonError(message) => format!("{}", message),
            }}),
        }
    }
}

impl From<cf_dist_utils::ValueAtRiskError> for ParameterError {
    fn from(error: cf_dist_utils::ValueAtRiskError) -> ParameterError {
        ParameterError::new(&ErrorType::ValueAtRiskError(error.to_string()))
    }
}
impl From<JsonError<'_>> for ParameterError {
    fn from(error: JsonError) -> ParameterError {
        let msg = match error {
            JsonError::Io(err) => err.to_string(),
            JsonError::Parse(v, err) => format!("parse error {}, received {}", err, v),
        };
        ParameterError::new(&ErrorType::JsonError(msg))
    }
}

impl fmt::Display for ParameterError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg.get("err").unwrap().as_str().unwrap())
    }
}
impl Error for ParameterError {
    fn description(&self) -> &str {
        self.msg.get("err").unwrap().as_str().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConstraintsSchema {
    pub lower: f64,
    pub upper: f64,
    pub types: String,
}
#[derive(Serialize, Deserialize)]
pub struct CGMYParameters {
    pub c: f64,
    pub g: f64,
    pub m: f64,
    pub y: f64,
    pub sigma: f64,
    pub v0: f64,
    pub speed: f64,
    pub eta_v: f64,
    pub rho: f64,
}
#[derive(Serialize, Deserialize)]
pub struct MertonParameters {
    pub lambda: f64,
    pub mu_l: f64,
    pub sig_l: f64,
    pub sigma: f64,
    pub v0: f64,
    pub speed: f64,
    pub eta_v: f64,
    pub rho: f64,
}
#[derive(Serialize, Deserialize)]
pub struct HestonParameters {
    pub sigma: f64,
    pub v0: f64,
    pub speed: f64,
    pub eta_v: f64,
    pub rho: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum CFParameters {
    Merton(MertonParameters),
    CGMY(CGMYParameters),
    Heston(HestonParameters),
}

#[derive(Serialize, Deserialize)]
pub struct OptionParameters {
    pub maturity: f64,
    pub rate: f64,
    pub asset: Option<f64>,
    pub strikes: Option<VecDeque<f64>>,
    pub quantile: Option<f64>,
    pub num_u: usize, //raised to the power of two.  if this is 8, then there will be 2^8=256 discrete "u"
    pub cf_parameters: CFParameters,
}

pub fn extend_strikes(mut strikes: VecDeque<f64>, asset: f64, x_max: f64) -> Vec<f64> {
    strikes.push_back((-x_max).exp() * asset);
    strikes.push_front(x_max.exp() * asset);
    Vec::from(strikes)
}

#[derive(Serialize, Deserialize)]
pub struct ParameterConstraints {
    pub rate: ConstraintsSchema,
    pub asset: ConstraintsSchema,
    pub maturity: ConstraintsSchema,
    pub num_u: ConstraintsSchema,
    pub quantile: ConstraintsSchema,
}

#[derive(Serialize, Deserialize)]
pub struct MertonConstraints {
    pub lambda: ConstraintsSchema,
    pub mu_l: ConstraintsSchema,
    pub sig_l: ConstraintsSchema,
    pub sigma: ConstraintsSchema,
    pub v0: ConstraintsSchema,
    pub speed: ConstraintsSchema,
    pub eta_v: ConstraintsSchema,
    pub rho: ConstraintsSchema,
}

#[derive(Serialize, Deserialize)]
pub struct CGMYConstraints {
    pub c: ConstraintsSchema,
    pub g: ConstraintsSchema,
    pub m: ConstraintsSchema,
    pub y: ConstraintsSchema,
    pub sigma: ConstraintsSchema,
    pub v0: ConstraintsSchema,
    pub speed: ConstraintsSchema,
    pub eta_v: ConstraintsSchema,
    pub rho: ConstraintsSchema,
}

#[derive(Serialize, Deserialize)]
pub struct HestonConstraints {
    pub sigma: ConstraintsSchema,
    pub v0: ConstraintsSchema,
    pub speed: ConstraintsSchema,
    pub eta_v: ConstraintsSchema,
    pub rho: ConstraintsSchema,
}

pub fn get_constraints() -> ParameterConstraints {
    ParameterConstraints {
        rate: ConstraintsSchema {
            lower: 0.0,
            upper: 0.4,
            types: "float".to_string(),
        },
        asset: ConstraintsSchema {
            lower: 0.0,
            upper: 1000000.0,
            types: "float".to_string(),
        },
        maturity: ConstraintsSchema {
            lower: 0.0,
            upper: 1000000.0,
            types: "float".to_string(),
        },
        num_u: ConstraintsSchema {
            lower: 5.0,
            upper: 10.0,
            types: "int".to_string(),
        },
        quantile: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        },
    }
}

pub fn get_merton_constraints() -> MertonConstraints {
    MertonConstraints {
        lambda: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
        },
        mu_l: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
        },
        sig_l: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
        },
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.2,
            upper: 1.8,
            types: "float".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
        },
    }
}
pub fn get_cgmy_constraints() -> CGMYConstraints {
    CGMYConstraints {
        c: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
        },
        g: ConstraintsSchema {
            lower: 0.0,
            upper: 20.0,
            types: "float".to_string(),
        },
        m: ConstraintsSchema {
            lower: 0.0,
            upper: 20.0,
            types: "float".to_string(),
        },
        y: ConstraintsSchema {
            lower: -1.0,
            upper: 2.0,
            types: "float".to_string(),
        },
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.2,
            upper: 1.8,
            types: "float".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
        },
    }
}

pub fn get_heston_constraints() -> HestonConstraints {
    HestonConstraints {
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.001,
            upper: 1.5,
            types: "float".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
        },
    }
}

fn check_constraint<'a>(
    parameter: f64,
    constraint: &'a ConstraintsSchema,
    parameter_name: &'a str,
) -> Result<(), ParameterError> {
    if parameter >= constraint.lower && parameter <= constraint.upper {
        Ok(())
    } else {
        Err(ParameterError::new(&ErrorType::OutOfBounds(
            parameter_name.to_string(),
        )))
    }
}
fn check_constraint_option<'a>(
    parameter: &Option<f64>,
    constraint: &'a ConstraintsSchema,
    parameter_name: &'a str,
) -> Result<(), ParameterError> {
    match parameter {
        Some(param) => check_constraint(*param, constraint, parameter_name),
        None => Ok(()),
    }
}

pub fn check_parameters<'a>(
    parameters: &OptionParameters,
    constraints: &ParameterConstraints,
) -> Result<(), ParameterError> {
    check_constraint_option(&parameters.asset, &constraints.asset, "asset")?;
    check_constraint(parameters.maturity, &constraints.maturity, "maturity")?;
    check_constraint(parameters.rate, &constraints.rate, "rate")?;
    check_constraint(parameters.num_u as f64, &constraints.num_u, "num_u")?;
    check_constraint_option(&parameters.quantile, &constraints.quantile, "quantile")?;
    Ok(())
}
pub fn check_heston_parameters<'a>(
    parameters: &HestonParameters,
    constraints: &HestonConstraints,
) -> Result<(), ParameterError> {
    check_constraint(parameters.sigma, &constraints.sigma, "sigma")?;
    check_constraint(parameters.v0, &constraints.v0, "v0")?;
    check_constraint(parameters.speed, &constraints.speed, "speed")?;
    check_constraint(parameters.eta_v, &constraints.eta_v, "eta_v")?;
    check_constraint(parameters.rho, &constraints.rho, "rho")?;
    Ok(())
}
pub fn check_merton_parameters<'a>(
    parameters: &MertonParameters,
    constraints: &MertonConstraints,
) -> Result<(), ParameterError> {
    check_constraint(parameters.lambda, &constraints.lambda, "lambda")?;
    check_constraint(parameters.mu_l, &constraints.mu_l, "mu_l")?;
    check_constraint(parameters.sig_l, &constraints.sig_l, "sig_l")?;
    check_constraint(parameters.sigma, &constraints.sigma, "sigma")?;
    check_constraint(parameters.v0, &constraints.v0, "v0")?;
    check_constraint(parameters.speed, &constraints.speed, "speed")?;
    check_constraint(parameters.eta_v, &constraints.eta_v, "eta_v")?;
    check_constraint(parameters.rho, &constraints.rho, "rho")?;
    Ok(())
}
pub fn check_cgmy_parameters<'a>(
    parameters: &CGMYParameters,
    constraints: &CGMYConstraints,
) -> Result<(), ParameterError> {
    check_constraint(parameters.c, &constraints.c, "c")?;
    check_constraint(parameters.g, &constraints.g, "g")?;
    check_constraint(parameters.m, &constraints.m, "m")?;
    check_constraint(parameters.y, &constraints.y, "y")?;
    check_constraint(parameters.sigma, &constraints.sigma, "sigma")?;
    check_constraint(parameters.v0, &constraints.v0, "v0")?;
    check_constraint(parameters.speed, &constraints.speed, "speed")?;
    check_constraint(parameters.eta_v, &constraints.eta_v, "eta_v")?;
    check_constraint(parameters.rho, &constraints.rho, "rho")?;
    Ok(())
}

pub fn throw_no_exist_error(parameter: &str) -> ParameterError {
    ParameterError::new(&ErrorType::NoExist(parameter.to_string()))
}
pub fn throw_no_convergence_error() -> ParameterError {
    ParameterError::new(&ErrorType::NoConvergence())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_throw_no_exist_error() {
        let err = throw_no_exist_error("hello");
        assert_eq!(err.to_string(), "Parameter hello does not exist.");
    }
    #[test]
    fn test_check_convergence_error() {
        let err = throw_no_convergence_error();
        assert_eq!(
            err.to_string(),
            "Root does not exist for implied volatility"
        );
    }
    #[test]
    fn test_check_constraint_option() {
        let constraint = ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        };
        let parameter = Some(0.5);
        let result = check_constraint_option(&parameter, &constraint, "hello");
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_constraint_option_failure() {
        let constraint = ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        };
        let parameter = None;
        let result = check_constraint_option(&parameter, &constraint, "hello");
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_constraint_option_failure_bounds() {
        let constraint = ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
        };
        let parameter = Some(5.0);
        let result = check_constraint_option(&parameter, &constraint, "hello");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parameter hello out of bounds.".to_string()
        );
    }
    #[test]
    fn test_check_parameters_ok() {
        let parameters = OptionParameters {
            rate: 0.05,
            asset: Some(50.0),
            strikes: None,
            quantile: None,
            num_u: 8,
            maturity: 1.0,
            cf_parameters: CFParameters::Heston(HestonParameters {
                sigma: 0.3,
                v0: 0.2,
                speed: 0.5,
                eta_v: 0.3,
                rho: -0.2,
            }),
        };
        let result = check_parameters(&parameters, &get_constraints());
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_parameters_err() {
        let parameters = OptionParameters {
            rate: -0.05,
            asset: Some(50.0),
            strikes: None,
            quantile: None,
            maturity: 1.0,
            num_u: 8,
            cf_parameters: CFParameters::Heston(HestonParameters {
                sigma: 0.3,
                v0: 0.2,
                speed: 0.5,
                eta_v: 0.3,
                rho: -0.2,
            }),
        };
        let result = check_parameters(&parameters, &get_constraints());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parameter rate out of bounds."
        );
    }
    #[test]
    fn test_check_heston_parameters_ok() {
        let parameters = HestonParameters {
            sigma: 0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_heston_parameters(&parameters, &get_heston_constraints());
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_heston_parameters_err() {
        let parameters = HestonParameters {
            sigma: -0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_heston_parameters(&parameters, &get_heston_constraints());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parameter sigma out of bounds."
        );
    }
    #[test]
    fn test_check_merton_parameters_ok() {
        let parameters = MertonParameters {
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.2,
            sigma: 0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_merton_parameters(&parameters, &get_merton_constraints());
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_merton_parameters_err() {
        let parameters = MertonParameters {
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.2,
            sigma: -0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_merton_parameters(&parameters, &get_merton_constraints());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parameter sigma out of bounds."
        );
    }
    #[test]
    fn test_check_cgmy_parameters_ok() {
        let parameters = CGMYParameters {
            c: 0.5,
            g: 3.0,
            m: 3.0,
            y: 0.2,
            sigma: 0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_cgmy_parameters(&parameters, &get_cgmy_constraints());
        assert!(result.is_ok());
    }
    #[test]
    fn test_check_cgmy_parameters_err() {
        let parameters = CGMYParameters {
            c: 0.5,
            g: 3.0,
            m: 3.0,
            y: 0.2,
            sigma: -0.3,
            v0: 0.2,
            speed: 0.5,
            eta_v: 0.3,
            rho: -0.2,
        };
        let result = check_cgmy_parameters(&parameters, &get_cgmy_constraints());
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Parameter sigma out of bounds."
        );
    }

    #[test]
    fn test_serialization_heston() {
        let json_str = r#"{
            "maturity": 0.5,
            "rate": 0.05,
            "num_u": 8,
            "cf_parameters":{
                "sigma":0.5,
                "speed":0.1,
                "v0":0.2,
                "eta_v":0.1,
                "rho":-0.5
            }
        }"#;
        let parameters: OptionParameters = serde_json::from_str(json_str).unwrap();
        match parameters.cf_parameters {
            CFParameters::Heston(cf_params) => {
                assert_eq!(cf_params.sigma, 0.5);
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn test_serialization_merton() {
        let json_str = r#"{
            "maturity": 0.5,
            "rate": 0.05,
            "num_u": 8,
            "cf_parameters":{
                "sigma":0.5,
                "speed":0.1,
                "v0":0.2,
                "eta_v":0.1,
                "rho":-0.5,
                "lambda": 0.5,
                "mu_l": -0.05,
                "sig_l": 0.3
            }
        }"#;
        let parameters: OptionParameters = serde_json::from_str(json_str).unwrap();
        match parameters.cf_parameters {
            CFParameters::Merton(cf_params) => {
                assert_eq!(cf_params.sigma, 0.5);
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn test_serialization_cgmy() {
        let json_str = r#"{
            "maturity": 0.5,
            "rate": 0.05,
            "num_u": 8,
            "cf_parameters":{
                "sigma":0.5,
                "speed":0.1,
                "v0":0.2,
                "eta_v":0.1,
                "rho":-0.5,
                "c": 0.5,
                "g": 3.0,
                "m": 4.0,
                "y":0.5
            }
        }"#;
        let parameters: OptionParameters = serde_json::from_str(json_str).unwrap();
        match parameters.cf_parameters {
            CFParameters::CGMY(cf_params) => {
                assert_eq!(cf_params.sigma, 0.5);
            }
            _ => assert!(false),
        }
    }
}
