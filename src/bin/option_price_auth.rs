//For use with rapidapi
#[macro_use]
extern crate rocket;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Value};
use std::env;
const OPTION_SCALE: f64 = 10.0;
const DENSITY_SCALE: f64 = 5.0;
use rocket::tokio::task;
use utils::{auth, calibration_maps, constants, constraints, pricing_maps};
#[get("/<model>/parameters/parameter_ranges")]
pub async fn parameters(_key: auth::ApiKey, model: &str) -> Value {
    match model {
        constants::HESTON_NAME => json!(constraints::HESTON_CONSTRAINTS),
        constants::CGMY_NAME => json!(constraints::CGMY_CONSTRAINTS),
        constants::MERTON_NAME => json!(constraints::MERTON_CONSTRAINTS),
        _ => json!(constraints::PARAMETER_CONSTRAINTS),
    }
}

#[post(
    "/<_model>/calculator/<option_type>/<sensitivity>?<include_implied_volatility>",
    data = "<parameters>"
)]
pub async fn calculator(
    _key: auth::ApiKey,
    _model: &str,
    option_type: &str,
    sensitivity: &str,
    parameters: Json<constraints::OptionParameters>,
    include_implied_volatility: Option<bool>,
) -> Result<Json<Vec<pricing_maps::GraphElement>>, constraints::ParameterError> {
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
    let result = task::spawn_blocking(move || {
        pricing_maps::get_option_results_as_json(
            fn_indicator,
            include_iv,
            &cf_parameters,
            OPTION_SCALE,
            num_u,
            asset_unwrap,
            maturity,
            rate,
            &strikes_unwrap,
        )
    })
    .await??;
    //.unwrap()?;
    Ok(Json(result))
}
#[post("/<model>/calibrator/call", data = "<calibration_parameters>")]
pub fn calibrator(
    _key: auth::ApiKey,
    model: &str,
    calibration_parameters: Json<constraints::CalibrationParameters>,
) -> Result<Json<constraints::CalibrationResponse>, constraints::ParameterError> {
    //let calibration_parameters = calibration_parameters?;
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
    Ok(Json(results))
}

#[post("/<_model>/density", data = "<parameters>")]
pub fn density(
    _key: auth::ApiKey,
    _model: &str,
    parameters: Json<constraints::OptionParameters>,
) -> Result<Json<Vec<pricing_maps::GraphElement>>, constraints::ParameterError> {
    //let parameters = parameters?;
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

    Ok(Json(results))
}

#[post("/<_model>/riskmetric", data = "<parameters>")]
pub fn risk_metric(
    _key: auth::ApiKey,
    _model: &str,
    parameters: Json<constraints::OptionParameters>,
) -> Result<Json<cf_dist_utils::RiskMetric>, constraints::ParameterError> {
    //let parameters = parameters?;
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

    Ok(Json(results))
}
/*
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let port_str = env::var("PORT")?;
    let port = port_str.parse::<u16>()?;
    let mount_point = env::var("MAJOR_VERSION")?;
    let config = Config::build().address("0.0.0.0").port(port).finalize()?;
    rocket::custom(config)
        .mount(
            format!("/{}", mount_point.as_str()).as_str(),
            routes![parameters, calculator, calibrator, density, risk_metric],
        )
        .launch();

    Ok(())
}*/

#[launch]
fn rocket() -> _ {
    let mount_point = env::var("MAJOR_VERSION").unwrap();
    rocket::build().mount(
        format!("/{}", mount_point.as_str()).as_str(),
        routes![parameters, calculator, calibrator, density, risk_metric],
    )
}
