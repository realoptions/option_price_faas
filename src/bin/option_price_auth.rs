//For use with rapidapi
#[macro_use]
extern crate rocket;
use rocket::serde::json::Json;
use rocket::serde::json::{json, Error as JsonError, Value};
use std::env;
const OPTION_SCALE: f64 = 10.0;
const DENSITY_SCALE: f64 = 5.0;
use rocket::tokio::task;
use utils::{auth, constants, constraints, pricing_maps};
#[get("/<model>/parameters/parameter_ranges")]
pub async fn parameters(_key: auth::ApiKey, model: &str) -> Value {
    match model {
        constants::HESTON_NAME => json!(constraints::HESTON_CONSTRAINTS),
        constants::CGMY_NAME => json!(constraints::CGMY_CONSTRAINTS),
        constants::CGMYSE_NAME => json!(constraints::CGMYSE_CONSTRAINTS),
        constants::MERTON_NAME => json!(constraints::MERTON_CONSTRAINTS),
        _ => json!(constraints::PARAMETER_CONSTRAINTS),
    }
}

#[post(
    "/<_>/calculator/<option_type>/<sensitivity>?<include_implied_volatility>",
    data = "<parameters>"
)]
pub async fn calculator(
    _key: auth::ApiKey,
    option_type: &str,
    sensitivity: &str,
    parameters: Result<Json<constraints::OptionParameters>, JsonError<'_>>,
    include_implied_volatility: Option<bool>,
) -> Result<Json<Vec<pricing_maps::GraphElement>>, constraints::ParameterError> {
    let fn_indicator = pricing_maps::get_fn_indicators(option_type, sensitivity)?;
    let parameters = parameters?;
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
    let results = task::spawn_blocking(move || {
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
    Ok(Json(results))
}

#[post("/<_>/density", data = "<parameters>")]
pub async fn density(
    _key: auth::ApiKey,
    parameters: Result<Json<constraints::OptionParameters>, JsonError<'_>>,
) -> Result<Json<Vec<pricing_maps::GraphElement>>, constraints::ParameterError> {
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

    let results = task::spawn_blocking(move || {
        pricing_maps::get_density_results_as_json(
            &cf_parameters,
            DENSITY_SCALE,
            num_u,
            maturity,
            rate,
        )
    })
    .await??;

    Ok(Json(results))
}

#[post("/<_>/riskmetric", data = "<parameters>")]
pub async fn risk_metric(
    _key: auth::ApiKey,
    parameters: Result<Json<constraints::OptionParameters>, JsonError<'_>>,
) -> Result<Json<cf_dist_utils::RiskMetric>, constraints::ParameterError> {
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
    let results = task::spawn_blocking(move || {
        pricing_maps::get_risk_measure_results_as_json(
            &cf_parameters,
            DENSITY_SCALE,
            num_u,
            maturity,
            rate,
            quantile_unwrap,
        )
    })
    .await??;

    Ok(Json(results))
}
#[launch]
fn rocket() -> _ {
    let mount_point = env::var("MAJOR_VERSION").unwrap();
    rocket::build().mount(
        format!("/{}", mount_point.as_str()).as_str(),
        routes![parameters, calculator, density, risk_metric],
    )
}
