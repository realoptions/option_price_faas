#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
use rocket::http::RawStr;
use rocket_contrib::json::{Json, JsonError, JsonValue};
use std::env;
use utils::{constraints, maps};
const OPTION_SCALE: f64 = 10.0;
const DENSITY_SCALE: f64 = 5.0;
use rocket::config::{Config, Environment};

#[get("/v2/<model>/parameters/parameter_ranges")]
fn parameters(model: &RawStr) -> JsonValue {
    match model.as_str() {
        "heston" => json!(constraints::get_heston_constraints()),
        "cgmy" => json!(constraints::get_cgmy_constraints()),
        "merton" => json!(constraints::get_merton_constraints()),
        _ => json!(constraints::get_constraints()),
    }
}

#[post(
    "/v2/<_model>/calculator/<option_type>/<sensitivity>?<include_implied_volatility>",
    data = "<parameters>"
)]
fn calculator(
    _model: &RawStr,
    option_type: &RawStr,
    sensitivity: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
    include_implied_volatility: Option<bool>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    let fn_indicator = maps::get_fn_indicators(option_type, sensitivity)?;
    constraints::check_parameters(&parameters, &constraints::get_constraints())?;
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
    let results = maps::get_option_results_as_json(
        fn_indicator,
        include_iv,
        &cf_parameters,
        OPTION_SCALE,
        num_u,
        asset_unwrap,
        maturity,
        rate,
        strikes_unwrap,
    )?;
    Ok(json!(results))
}

#[post("/v2/<_model>/density", data = "<parameters>")]
fn density(
    _model: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    constraints::check_parameters(&parameters, &constraints::get_constraints())?;

    let constraints::OptionParameters {
        maturity,
        rate,
        num_u: num_u_base,
        cf_parameters,
        ..
    } = parameters.into_inner(); //destructure

    let num_u = (2 as usize).pow(num_u_base as u32);

    let results =
        maps::get_density_results_as_json(&cf_parameters, DENSITY_SCALE, num_u, maturity, rate)?;

    Ok(json!(results))
}

#[post("/v2/<_model>/riskmetric", data = "<parameters>")]
fn risk_metric(
    _model: &RawStr,
    parameters: Result<Json<constraints::OptionParameters>, JsonError>,
) -> Result<JsonValue, constraints::ParameterError> {
    let parameters = parameters?;
    constraints::check_parameters(&parameters, &constraints::get_constraints())?;

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
    let results = maps::get_risk_measure_results_as_json(
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
    let port = port_str.parse::<u16>()?;
    let config = Config::build(Environment::Production)
        .address("0.0.0.0")
        .port(port)
        .finalize()?;
    rocket::custom(config)
        .mount("/", routes![parameters, calculator, density, risk_metric])
        .launch();

    Ok(())
}
