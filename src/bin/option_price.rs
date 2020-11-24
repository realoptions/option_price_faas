#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
use rocket::config::{Config, Environment};
use rocket::http::RawStr;
use rocket_contrib::json::{Json, JsonError, JsonValue};
use std::env;
const OPTION_SCALE: f64 = 10.0;
const DENSITY_SCALE: f64 = 5.0;
use utils::{calibration_maps, constraints, pricing_maps};

#[get("/<model>/parameters/parameter_ranges")]
pub fn parameters(model: &RawStr) -> JsonValue {
    match model.as_str() {
        "heston" => json!(constraints::HESTON_CONSTRAINTS),
        "cgmy" => json!(constraints::CGMY_CONSTRAINTS),
        "merton" => json!(constraints::MERTON_CONSTRAINTS),
        _ => json!(constraints::PARAMETER_CONSTRAINTS),
    }
}

#[post(
    "/<_model>/calculator/<option_type>/<sensitivity>?<include_implied_volatility>",
    data = "<parameters>"
)]
pub fn calculator(
    _model: &RawStr,
    option_type: &RawStr,
    sensitivity: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
    include_implied_volatility: Option<bool>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    let fn_indicator = pricing_maps::get_fn_indicators(option_type, sensitivity)?;
    constraints::check_parameters(&parameters, &constraints::PARAMETER_CONSTRAINTS)?;
    let constraints::OptionParameters {
        maturity,
        rate,
        asset,
        num_u: num_u_base,
        strikes,
        cf_parameters,
        ..
    } = parameters.into_inner(); //destructure

    let strikes_unwrap = strikes.ok_or(constraints::throw_no_exist_error("strikes"))?;
    let asset_unwrap = asset.ok_or(constraints::throw_no_exist_error("asset"))?;

    let num_u = (2 as usize).pow(num_u_base as u32);
    let include_iv = include_implied_volatility.unwrap_or(false);
    let results = pricing_maps::get_option_results_as_json(
        fn_indicator,
        include_iv,
        &cf_parameters,
        OPTION_SCALE,
        num_u,
        asset_unwrap,
        maturity,
        rate,
        &strikes_unwrap,
    )?;
    Ok(json!(results))
}

#[post("/<_model>/density", data = "<parameters>")]
pub fn density(
    _model: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    constraints::check_parameters(&parameters, &constraints::PARAMETER_CONSTRAINTS)?;

    let constraints::OptionParameters {
        maturity,
        rate,
        num_u: num_u_base,
        cf_parameters,
        ..
    } = parameters.into_inner(); //destructure

    let num_u = (2 as usize).pow(num_u_base as u32);

    let results = pricing_maps::get_density_results_as_json(
        &cf_parameters,
        DENSITY_SCALE,
        num_u,
        maturity,
        rate,
    )?;

    Ok(json!(results))
}
#[post("/<model>/calibrator/call", data = "<calibration_parameters>")]
pub fn calibrator(
    model: &RawStr,
    calibration_parameters: Result<Json<constraints::CalibrationParameters>, JsonError>,
) -> Result<JsonValue, constraints::ParameterError> {
    let calibration_parameters = calibration_parameters?;
    let constraints::CalibrationParameters {
        rate,
        asset,
        num_u: num_u_base,
        option_data,
    } = calibration_parameters.into_inner();
    let model_indicator = calibration_maps::get_model_indicators(model)?;
    let num_iter = calibration_maps::get_num_iter(model)?;
    let num_u = (2 as usize).pow(num_u_base as u32);
    let results = calibration_maps::get_option_calibration_results_as_json(
        model_indicator,
        &option_data,
        OPTION_SCALE,
        num_iter,
        num_u,
        asset,
        rate,
    )?;
    Ok(json!(results))
}
#[post("/<_model>/riskmetric", data = "<parameters>")]
pub fn risk_metric(
    _model: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    constraints::check_parameters(&parameters, &constraints::PARAMETER_CONSTRAINTS)?;

    let constraints::OptionParameters {
        maturity,
        rate,
        num_u: num_u_base,
        quantile,
        cf_parameters,
        ..
    } = parameters.into_inner(); //destructure

    let num_u = (2 as usize).pow(num_u_base as u32);
    let quantile_unwrap = quantile.ok_or(constraints::throw_no_exist_error("quantile"))?;
    let results = pricing_maps::get_risk_measure_results_as_json(
        &cf_parameters,
        DENSITY_SCALE,
        num_u,
        maturity,
        rate,
        quantile_unwrap,
    )?;

    Ok(json!(results))
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port_str = env::var("PORT")?;
    let mount_point = env::var("MAJOR_VERSION")?;
    let port = port_str.parse::<u16>()?;
    let config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .port(port)
        .finalize()?;
    rocket::custom(config)
        .mount(
            format!("/{}", mount_point.as_str()).as_str(),
            routes![parameters, calculator, calibrator, density, risk_metric],
        )
        .launch();

    Ok(())
}
