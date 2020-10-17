use rocket::response::Responder;
use rocket_contrib::json::{JsonError, JsonValue};
use serde_derive::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

pub enum ErrorType {
    OutOfBounds(String),
    NoExist(String),
    FunctionError(String),
    NoConvergence(),
    ValueAtRiskError(String),
    JsonError(String),
    OptimizationError(String),
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
                ErrorType::OptimizationError(message)=>format!("{}", message)
            }}),
        }
    }
}

impl From<cf_dist_utils::ValueAtRiskError> for ParameterError {
    fn from(error: cf_dist_utils::ValueAtRiskError) -> ParameterError {
        ParameterError::new(&ErrorType::ValueAtRiskError(error.to_string()))
    }
}
impl From<argmin::core::Error> for ParameterError {
    fn from(error: argmin::core::Error) -> ParameterError {
        ParameterError::new(&ErrorType::OptimizationError(error.to_string()))
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
    pub description: String,
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
impl CGMYParameters {
    fn to_vector(&self) -> Vec<(f64, &str)> {
        vec![
            (self.c, "c"),
            (self.g, "g"),
            (self.m, "m"),
            (self.y, "y"),
            (self.sigma, "sigma"),
            (self.v0, "v0"),
            (self.speed, "speed"),
            (self.eta_v, "eta_v"),
            (self.rho, "rho"),
        ]
    }
}
impl HestonParameters {
    fn to_vector(&self) -> Vec<(f64, &str)> {
        vec![
            (self.sigma, "sigma"),
            (self.v0, "v0"),
            (self.speed, "speed"),
            (self.eta_v, "eta_v"),
            (self.rho, "rho"),
        ]
    }
}
impl MertonParameters {
    fn to_vector(&self) -> Vec<(f64, &str)> {
        vec![
            (self.lambda, "lambda"),
            (self.mu_l, "mu_l"),
            (self.sig_l, "sig_l"),
            (self.sigma, "sigma"),
            (self.v0, "v0"),
            (self.speed, "speed"),
            (self.eta_v, "eta_v"),
            (self.rho, "rho"),
        ]
    }
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
    pub strikes: Option<Vec<f64>>,
    pub quantile: Option<f64>,
    pub num_u: usize, //raised to the power of two.  if this is 8, then there will be 2^8=256 discrete "u"
    pub cf_parameters: CFParameters,
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

impl CGMYConstraints {
    pub fn to_vector(&self) -> Vec<ConstraintsSchema> {
        vec![
            self.c, self.g, self.m, self.y, self.sigma, self.v0, self.speed, self.eta_v, self.rho,
        ]
    }
}
impl HestonConstraints {
    pub fn to_vector(&self) -> Vec<ConstraintsSchema> {
        vec![self.sigma, self.v0, self.speed, self.eta_v, self.rho]
    }
}
impl MertonConstraints {
    pub fn to_vector(&self) -> Vec<ConstraintsSchema> {
        vec![
            self.lambda,
            self.mu_l,
            self.sig_l,
            self.sigma,
            self.v0,
            self.speed,
            self.eta_v,
            self.rho,
        ]
    }
}
pub fn get_constraints() -> ParameterConstraints {
    ParameterConstraints {
        rate: ConstraintsSchema {
            lower: 0.0,
            upper: 0.4,
            types: "float".to_string(),
            description: "Annualized risk-free interest rate".to_string(),
        },
        asset: ConstraintsSchema {
            lower: 0.0,
            upper: 1000000.0,
            types: "float".to_string(),
            description: "Underlying asset".to_string(),
        },
        maturity: ConstraintsSchema {
            lower: 0.0,
            upper: 1000000.0,
            types: "float".to_string(),
            description: "Time in years till option expiration".to_string(),
        },
        num_u: ConstraintsSchema {
            lower: 5.0,
            upper: 10.0,
            types: "int".to_string(),
            description: "Exponent for the precision of the numeric inversion.  For example, 8 represents 2^8=256.".to_string()
        },
        quantile: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Quantile of (risk-neutral) distribution of the underlying asset.  For example, 0.05 would map to a 95% VaR.".to_string()
        },
    }
}

pub fn get_merton_constraints() -> MertonConstraints {
    MertonConstraints {
        lambda: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
            description: "Annualized frequency of jumps for the asset process".to_string(),
        },
        mu_l: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Mean jump size".to_string(),
        },
        sig_l: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
            description: "Volatility of jump size".to_string(),
        },
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Volatility of diffusion component of asset process".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.2,
            upper: 1.8,
            types: "float".to_string(),
            description: "Initial value of the time-change diffusion".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Rate at which time-change diffusion reverts to mean".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Volatility of time-change diffusion".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Correlation between asset and time-change diffusions".to_string(),
        },
    }
}
pub fn get_cgmy_constraints() -> CGMYConstraints {
    CGMYConstraints {
        c: ConstraintsSchema {
            lower: 0.0,
            upper: 2.0,
            types: "float".to_string(),
            description: "Parameter C from CGMY, controls overall level of jump frequency"
                .to_string(),
        },
        g: ConstraintsSchema {
            lower: 0.0,
            upper: 20.0,
            types: "float".to_string(),
            description:
                "Parameter G from CGMY, controls rate of decay for left side of asset distribution"
                    .to_string(),
        },
        m: ConstraintsSchema {
            lower: 0.0,
            upper: 20.0,
            types: "float".to_string(),
            description:
                "Parameter M from CGMY, controls rate of decay for right side of asset distribution"
                    .to_string(),
        },
        y: ConstraintsSchema {
            lower: -1.0,
            upper: 2.0,
            types: "float".to_string(),
            description: "Parameter Y from CGMY, characterizes fine structure of jumps".to_string(),
        },
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Volatility of diffusion component of asset process".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.2,
            upper: 1.8,
            types: "float".to_string(),
            description: "Initial value of the time-change diffusion".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Rate at which time-change diffusion reverts to mean".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Volatility of time-change diffusion".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Correlation between asset and time-change diffusions".to_string(),
        },
    }
}
pub fn get_heston_constraints() -> HestonConstraints {
    HestonConstraints {
        sigma: ConstraintsSchema {
            lower: 0.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Square root of mean of variance process".to_string(),
        },
        v0: ConstraintsSchema {
            lower: 0.001,
            upper: 1.5,
            types: "float".to_string(),
            description: "Square root of initial value of the instantaneous variance".to_string(),
        },
        speed: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Rate at which variance reverts to mean".to_string(),
        },
        eta_v: ConstraintsSchema {
            lower: 0.0,
            upper: 3.0,
            types: "float".to_string(),
            description: "Vol of vol: volatility of instantaneous variance".to_string(),
        },
        rho: ConstraintsSchema {
            lower: -1.0,
            upper: 1.0,
            types: "float".to_string(),
            description: "Correlation between asset and variance diffusions".to_string(),
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
    for ((param, name), constraint) in parameters.to_vector().iter().zip(constraints.to_vector()) {
        check_constraint(*param, &constraint, name)?;
    }
    Ok(())
}
pub fn check_merton_parameters<'a>(
    parameters: &MertonParameters,
    constraints: &MertonConstraints,
) -> Result<(), ParameterError> {
    for ((param, name), constraint) in parameters.to_vector().iter().zip(constraints.to_vector()) {
        check_constraint(*param, &constraint, name)?;
    }
    Ok(())
}
pub fn check_cgmy_parameters<'a>(
    parameters: &CGMYParameters,
    constraints: &CGMYConstraints,
) -> Result<(), ParameterError> {
    for ((param, name), constraint) in parameters.to_vector().iter().zip(constraints.to_vector()) {
        check_constraint(*param, &constraint, name)?;
    }
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
            description: "hello".to_string(),
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
            description: "hello".to_string(),
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
            description: "hello".to_string(),
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
