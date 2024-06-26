use crate::constants::{
    CALL_DELTA, CALL_GAMMA, CALL_PRICE, CALL_THETA, DENSITY, PUT_DELTA, PUT_GAMMA, PUT_PRICE,
    PUT_THETA, RISK_MEASURES,
};
use crate::constraints::{
    check_cgmy_parameters, check_cgmyse_parameters, check_heston_parameters,
    check_merton_parameters, throw_no_convergence_error, CFParameters, CGMYParameters,
    CGMYSEParameters, ErrorType, HestonParameters, MertonParameters, ParameterError,
    CGMYSE_CONSTRAINTS, CGMY_CONSTRAINTS, HESTON_CONSTRAINTS, MERTON_CONSTRAINTS,
};

use fang_oost_option::option_pricing;
use num_complex::Complex;
use rayon::prelude::*;
use serde_derive::{Deserialize, Serialize};

/// Gets indicators for which sensitivity
/// to retrieve
/// # Examples
///
/// ```
/// extern crate utils;
/// use utils::pricing_maps;
/// # fn main() {
/// let sensitivity = pricing_maps::get_fn_indicators(
///     "put",
///     "price"
/// ).unwrap();
/// # }
/// ```
pub fn get_fn_indicators(option_type: &str, sensitivity: &str) -> Result<i32, ParameterError> {
    let combine_types = format!("{}_{}", option_type, sensitivity);
    match combine_types.as_str() {
        "put_price" => Ok(PUT_PRICE),
        "call_price" => Ok(CALL_PRICE),
        "put_delta" => Ok(PUT_DELTA),
        "call_delta" => Ok(CALL_DELTA),
        "put_gamma" => Ok(PUT_GAMMA),
        "call_gamma" => Ok(CALL_GAMMA),
        "put_theta" => Ok(PUT_THETA),
        "call_theta" => Ok(CALL_THETA),
        "density_" => Ok(DENSITY),
        "riskmetric_" => Ok(RISK_MEASURES),
        _ => Err(ParameterError::new(&ErrorType::FunctionError(
            combine_types,
        ))),
    }
}

fn get_cgmy_cf(
    cf_parameters: &CGMYParameters,
    maturity: f64,
    rate: f64,
) -> Result<(impl Fn(&Complex<f64>) -> Complex<f64>, f64), ParameterError> {
    check_cgmy_parameters(&cf_parameters, &CGMY_CONSTRAINTS)?;
    let CGMYParameters {
        c,
        g,
        m,
        y,
        sigma,
        v0,
        speed,
        eta_v,
        rho,
    } = cf_parameters;
    let cf_inst = cf_functions::cgmy::cgmy_time_change_cf(
        maturity, rate, *c, *g, *m, *y, *sigma, *v0, *speed, *eta_v, *rho,
    );
    let vol = cf_functions::cgmy::cgmy_diffusion_vol(*sigma, *c, *g, *m, *y, maturity);
    Ok((cf_inst, vol))
}
fn get_cgmyse_cf(
    cf_parameters: &CGMYSEParameters,
    maturity: f64,
    rate: f64,
) -> Result<(impl Fn(&Complex<f64>) -> Complex<f64>, f64), ParameterError> {
    check_cgmyse_parameters(&cf_parameters, &CGMYSE_CONSTRAINTS)?;
    let num_steps = 256; //
    let CGMYSEParameters {
        c,
        g,
        m,
        y,
        sigma,
        v0,
        speed,
        eta_v,
        //rho,
    } = cf_parameters;
    let cf_inst = cf_functions::cgmy::cgmyse_time_change_cf(
        maturity, rate, *c, *g, *m, *y, *sigma, *v0, *speed, *eta_v, num_steps,
    );
    let vol = cf_functions::cgmy::cgmy_diffusion_vol(*sigma, *c, *g, *m, *y, maturity);
    Ok((cf_inst, vol))
}
fn get_merton_cf(
    cf_parameters: &MertonParameters,
    maturity: f64,
    rate: f64,
) -> Result<(impl Fn(&Complex<f64>) -> Complex<f64>, f64), ParameterError> {
    check_merton_parameters(&cf_parameters, &MERTON_CONSTRAINTS)?;
    let MertonParameters {
        lambda,
        mu_l,
        sig_l,
        sigma,
        v0,
        speed,
        eta_v,
        rho,
    } = cf_parameters;
    let cf_inst = cf_functions::merton::merton_time_change_cf(
        maturity, rate, *lambda, *mu_l, *sig_l, *sigma, *v0, *speed, *eta_v, *rho,
    );
    let vol = cf_functions::merton::jump_diffusion_vol(*sigma, *lambda, *mu_l, *sig_l, maturity);
    Ok((cf_inst, vol))
}

fn get_heston_cf(
    cf_parameters: &HestonParameters,
    maturity: f64,
    rate: f64,
) -> Result<(impl Fn(&Complex<f64>) -> Complex<f64>, f64), ParameterError> {
    check_heston_parameters(&cf_parameters, &HESTON_CONSTRAINTS)?;
    let HestonParameters {
        sigma,
        v0,
        speed,
        eta_v,
        rho,
    } = cf_parameters;
    let cf_inst = cf_functions::gauss::heston_cf(maturity, rate, *sigma, *v0, *speed, *eta_v, *rho);
    Ok((cf_inst, sigma * maturity.sqrt()))
}

pub(crate) fn get_max_strike(asset: f64, option_scale: f64, vol: f64) -> f64 {
    (option_scale * vol).exp() * asset
}
pub fn get_option_results_as_json(
    fn_choice: i32,
    include_iv: bool,
    cf_parameters: &CFParameters,
    option_scale: f64,
    num_u: usize,
    asset: f64,
    maturity: f64,
    rate: f64,
    strikes: &[f64],
) -> Result<Vec<GraphElement>, ParameterError> {
    match cf_parameters {
        CFParameters::CGMY(cf_params) => {
            let (cf_inst, vol) = get_cgmy_cf(cf_params, maturity, rate)?;
            let max_strike = get_max_strike(asset, option_scale, vol);
            get_option_results(
                fn_choice, include_iv, num_u, asset, rate, maturity, &strikes, max_strike, &cf_inst,
            )
        }
        CFParameters::CGMYSE(cf_params) => {
            let (cf_inst, vol) = get_cgmyse_cf(cf_params, maturity, rate)?;
            let max_strike = get_max_strike(asset, option_scale, vol);
            get_option_results(
                fn_choice, include_iv, num_u, asset, rate, maturity, &strikes, max_strike, &cf_inst,
            )
        }
        CFParameters::Merton(cf_params) => {
            let (cf_inst, vol) = get_merton_cf(cf_params, maturity, rate)?;
            let max_strike = get_max_strike(asset, option_scale, vol);
            get_option_results(
                fn_choice, include_iv, num_u, asset, rate, maturity, &strikes, max_strike, &cf_inst,
            )
        }
        CFParameters::Heston(cf_params) => {
            let (cf_inst, vol) = get_heston_cf(cf_params, maturity, rate)?;
            let max_strike = get_max_strike(asset, option_scale, vol);
            get_option_results(
                fn_choice, include_iv, num_u, asset, rate, maturity, &strikes, max_strike, &cf_inst,
            )
        }
    }
}

pub fn get_density_results_as_json(
    cf_parameters: &CFParameters,
    density_scale: f64,
    num_u: usize,
    maturity: f64,
    rate: f64,
) -> Result<Vec<GraphElement>, ParameterError> {
    match cf_parameters {
        CFParameters::CGMY(cf_params) => {
            let (cf_inst, vol) = get_cgmy_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            get_density_results(num_u, x_max_density, &cf_inst)
        }
        CFParameters::CGMYSE(cf_params) => {
            let (cf_inst, vol) = get_cgmyse_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            get_density_results(num_u, x_max_density, &cf_inst)
        }
        CFParameters::Merton(cf_params) => {
            let (cf_inst, vol) = get_merton_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            get_density_results(num_u, x_max_density, &cf_inst)
        }
        CFParameters::Heston(cf_params) => {
            let (cf_inst, vol) = get_heston_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            get_density_results(num_u, x_max_density, &cf_inst)
        }
    }
}

pub fn get_risk_measure_results_as_json(
    cf_parameters: &CFParameters,
    density_scale: f64,
    num_u: usize,
    maturity: f64,
    rate: f64,
    quantile: f64,
) -> Result<cf_dist_utils::RiskMetric, ParameterError> {
    match cf_parameters {
        CFParameters::CGMY(cf_params) => {
            let (cf_inst, vol) = get_cgmy_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            let result = get_risk_measure_results(num_u, x_max_density, quantile, &cf_inst)?;
            Ok(result)
        }
        CFParameters::CGMYSE(cf_params) => {
            let (cf_inst, vol) = get_cgmyse_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            let result = get_risk_measure_results(num_u, x_max_density, quantile, &cf_inst)?;
            Ok(result)
        }
        CFParameters::Merton(cf_params) => {
            let (cf_inst, vol) = get_merton_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            let result = get_risk_measure_results(num_u, x_max_density, quantile, &cf_inst)?;
            Ok(result)
        }
        CFParameters::Heston(cf_params) => {
            let (cf_inst, vol) = get_heston_cf(cf_params, maturity, rate)?;
            let x_max_density = vol * density_scale;
            let result = get_risk_measure_results(num_u, x_max_density, quantile, &cf_inst)?;
            Ok(result)
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GraphElement {
    pub at_point: f64,
    pub value: f64,
    #[serde(skip_serializing_if = "Option::is_none")] //skip when iv is not provided
    pub iv: Option<f64>,
}

fn density_as_json(
    values: impl IndexedParallelIterator<Item = fang_oost::GraphElement>,
) -> Vec<GraphElement> {
    values
        .map(|fang_oost::GraphElement { x, value }| GraphElement {
            at_point: x,
            value,
            iv: None,
        })
        .collect::<Vec<_>>()
}

fn graph_no_iv_as_json(
    values: impl IndexedParallelIterator<Item = fang_oost::GraphElement>,
) -> Vec<GraphElement> {
    values
        .map(|fang_oost::GraphElement { x, value }| GraphElement {
            at_point: x,
            value,
            iv: None,
        })
        .collect::<Vec<_>>()
}
fn graph_iv_as_json<T>(
    values: impl IndexedParallelIterator<Item = fang_oost::GraphElement>,
    iv_fn: T,
) -> Result<Vec<GraphElement>, ParameterError>
where
    T: Fn(f64, f64) -> Result<f64, f64> + std::marker::Sync + std::marker::Send,
{
    values
        .map(|fang_oost::GraphElement { x, value }| {
            iv_fn(value, x)
                .map(|iv| GraphElement {
                    at_point: x,
                    value,
                    iv: Some(iv),
                })
                .map_err(|_err| throw_no_convergence_error())
        })
        .collect()
}

fn call_iv_as_json(
    values: impl IndexedParallelIterator<Item = fang_oost::GraphElement>,
    asset: f64,
    rate: f64,
    maturity: f64,
) -> Result<Vec<GraphElement>, ParameterError> {
    graph_iv_as_json(values, &|price, strike| {
        black_scholes::call_iv(price, asset, strike, rate, maturity)
    })
}
fn put_iv_as_json(
    values: impl IndexedParallelIterator<Item = fang_oost::GraphElement>,
    asset: f64,
    rate: f64,
    maturity: f64,
) -> Result<Vec<GraphElement>, ParameterError> {
    graph_iv_as_json(values, &|price, strike| {
        black_scholes::put_iv(price, asset, strike, rate, maturity)
    })
}

const NUM_X: usize = 128;
fn adjust_density<T>(num_u: usize, x_max: f64, cf: T) -> Vec<GraphElement>
where
    T: Fn(&Complex<f64>) -> Complex<f64> + std::marker::Sync + std::marker::Send,
{
    let x_min = -x_max;
    density_as_json(cf_dist_utils::get_pdf(NUM_X, num_u, x_min, x_max, &cf))
}

fn get_option_results<S>(
    fn_choice: i32,
    include_iv: bool,
    num_u: usize,
    asset: f64,
    rate: f64,
    maturity: f64,
    strikes: &[f64],
    max_strike: f64,
    inst_cf: S,
) -> Result<Vec<GraphElement>, ParameterError>
where
    S: Fn(&Complex<f64>) -> Complex<f64> + std::marker::Sync + std::marker::Send,
{
    match fn_choice {
        CALL_PRICE => {
            let prices = option_pricing::fang_oost_call_price(
                num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
            );
            if include_iv {
                call_iv_as_json(prices, asset, rate, maturity)
            } else {
                Ok(graph_no_iv_as_json(prices))
            }
        }
        PUT_PRICE => {
            let prices = option_pricing::fang_oost_put_price(
                num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
            );
            if include_iv {
                put_iv_as_json(prices, asset, rate, maturity)
            } else {
                Ok(graph_no_iv_as_json(prices))
            }
        }
        CALL_DELTA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_call_delta(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        PUT_DELTA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_put_delta(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        CALL_GAMMA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_call_gamma(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        PUT_GAMMA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_put_gamma(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        CALL_THETA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_call_theta(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        PUT_THETA => Ok(graph_no_iv_as_json(option_pricing::fang_oost_put_theta(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        ))),
        _ => Err(ParameterError::new(&ErrorType::FunctionError(format!(
            "{}",
            fn_choice
        )))),
    }
}

fn get_density_results(
    num_u: usize,
    x_max_density: f64,
    inst_cf: &(impl Fn(&Complex<f64>) -> Complex<f64> + std::marker::Sync),
) -> Result<Vec<GraphElement>, ParameterError> {
    Ok(adjust_density(num_u, x_max_density, &inst_cf))
}
const MAX_SIMS: usize = 100;
const PRECISION: f64 = 0.0000001;

fn get_risk_measure_results(
    num_u: usize,
    x_max_density: f64,
    quantile: f64,
    inst_cf: &(impl Fn(&Complex<f64>) -> Complex<f64> + std::marker::Sync),
) -> Result<cf_dist_utils::RiskMetric, cf_dist_utils::ValueAtRiskError> {
    cf_dist_utils::get_expected_shortfall_and_value_at_risk(
        quantile,
        num_u,
        -x_max_density,
        x_max_density,
        MAX_SIMS,
        PRECISION,
        &inst_cf,
    )
}

#[cfg(test)]
mod tests {
    use crate::pricing_maps::*;
    use approx::*;
    use rand::{distributions::Distribution, distributions::Uniform, rngs::StdRng, SeedableRng};
    #[test]
    fn get_fn_indicators_gets_match() {
        let model = get_fn_indicators("put", "price").unwrap();
        assert_eq!(model, PUT_PRICE);
    }
    fn get_rng_seed(seed: u64) -> StdRng {
        SeedableRng::seed_from_u64(seed)
    }
    fn get_over_region(lower: f64, upper: f64, rand: f64) -> f64 {
        lower + (upper - lower) * rand
    }
    #[test]
    fn test_many_inputs() {
        let mut rng_seed = get_rng_seed(42);
        let uniform = Uniform::new(0.0f64, 1.0);
        let asset = 178.46;
        let num_u = 256;
        let strikes = vec![
            95.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0, 210.0,
            240.0, 250.0,
        ];
        let maturity = 0.86;
        let rate = 0.02;
        let max_strike = 5000.0;
        let num_total: usize = 10000;
        let mut num_bad: usize = 0;
        (0..num_total).for_each(|_| {
            let lambda_sim = get_over_region(
                MERTON_CONSTRAINTS.lambda.lower,
                MERTON_CONSTRAINTS.lambda.upper,
                uniform.sample(&mut rng_seed),
            );
            let mu_l_sim = get_over_region(
                MERTON_CONSTRAINTS.mu_l.lower,
                MERTON_CONSTRAINTS.mu_l.upper,
                uniform.sample(&mut rng_seed),
            );
            let sig_l_sim = get_over_region(
                MERTON_CONSTRAINTS.sig_l.lower,
                MERTON_CONSTRAINTS.sig_l.upper,
                uniform.sample(&mut rng_seed),
            );
            let sigma_sim = get_over_region(
                MERTON_CONSTRAINTS.sigma.lower,
                MERTON_CONSTRAINTS.sigma.upper,
                uniform.sample(&mut rng_seed),
            );
            let v0_sim = get_over_region(
                MERTON_CONSTRAINTS.v0.lower,
                MERTON_CONSTRAINTS.v0.upper,
                uniform.sample(&mut rng_seed),
            );
            let speed_sim = get_over_region(
                MERTON_CONSTRAINTS.speed.lower,
                MERTON_CONSTRAINTS.speed.upper,
                uniform.sample(&mut rng_seed),
            );
            let eta_v_sim = get_over_region(
                MERTON_CONSTRAINTS.eta_v.lower,
                MERTON_CONSTRAINTS.eta_v.upper,
                uniform.sample(&mut rng_seed),
            );
            let rho_sim = get_over_region(
                MERTON_CONSTRAINTS.rho.lower,
                MERTON_CONSTRAINTS.rho.upper,
                uniform.sample(&mut rng_seed),
            );

            let inst_cf = cf_functions::merton::merton_time_change_cf(
                maturity, rate, lambda_sim, mu_l_sim, sig_l_sim, sigma_sim, v0_sim, speed_sim,
                eta_v_sim, rho_sim,
            );
            let opt_prices = option_pricing::fang_oost_call_price(
                num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
            );

            let result = opt_prices.find_any(|fang_oost::GraphElement { value, .. }| {
                value.is_nan() || value.is_infinite()
            });

            match result {
                Some(_x) => num_bad += 1,
                _ => {}
            };
        });
        let bad_rate = (num_bad as f64) / (num_total as f64);
        println!("Bad rate: {}", bad_rate);
        assert_eq!(bad_rate, 0.0);
    }
    #[test]
    fn for_some_extreme_values() {
        let asset = 223.4000;
        let rate = 0.0247;
        let maturity = 0.7599;
        let eta_v = 1.3689;
        let lambda = 0.0327;
        let mu_l = -0.3571;
        let rho = -0.0936;
        let sig_l = 0.5876;
        let sigma = 0.2072;
        let speed = 0.87;
        let v0 = 1.2104;
        let max_strike = get_max_strike(
            asset,
            10.0,
            cf_functions::merton::jump_diffusion_vol(sigma, lambda, mu_l, sig_l, maturity),
        );

        let strikes = vec![
            85.0, 90.0, 100.0, 110.0, 120.0, 125.0, 130.0, 135.0, 140.0, 145.0, 150.0, 155.0,
            160.0, 165.0, 170.0, 175.0, 180.0, 185.0, 190.0, 195.0, 200.0, 205.0, 210.0, 215.0,
            220.0, 225.0, 230.0, 235.0, 240.0, 245.0, 250.0, 255.0, 260.0, 265.0, 270.0, 275.0,
            280.0, 285.0, 290.0, 295.0, 300.0, 310.0, 320.0, 330.0, 340.0,
        ];
        let inst_cf = cf_functions::merton::merton_time_change_cf(
            maturity, rate, lambda, mu_l, sig_l, sigma, v0, speed, eta_v, rho,
        );
        let num_u = 256;
        let prices = option_pricing::fang_oost_call_price(
            num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
        );
        let result = call_iv_as_json(prices, asset, rate, maturity);
        assert!(result.is_ok());
    }
    #[test]
    fn get_fn_indicators_no_match() {
        assert_eq!(
            get_fn_indicators(&"something".to_string(), &"somethingelse".to_string())
                .unwrap_err()
                .to_string(),
            "Function indicator something_somethingelse does not exist."
        );
    }
    #[test]
    fn test_cgmy_price_1() {
        //https://mpra.ub.uni-muenchen.de/8914/4/MPRA_paper_8914.pdf pg 18
        //S0 = 100, K = 100, r = 0.1, q = 0, C = 1, G = 5, M = 5, T = 1, Y=0.5
        let parameters = CGMYParameters {
            sigma: 0.0,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        };

        let strikes = vec![100.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.1;
        let asset = 100.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            true,
            &CFParameters::CGMY(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 19.812948843, epsilon = 0.00001);
    }
    #[test]
    fn test_cgmy_price_2() {
        //https://mpra.ub.uni-muenchen.de/8914/4/MPRA_paper_8914.pdf pg 18
        //S0 = 100, K = 100, r = 0.1, q = 0, C = 1, G = 5, M = 5, T = 1, Y=1.5
        let parameters = CGMYParameters {
            sigma: 0.0,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 1.5,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        };
        let strikes = vec![100.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.1;
        let asset = 100.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::CGMY(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 49.790905469, epsilon = 0.00001);
    }
    #[test]
    fn test_cgmy_price_3() {
        //https://mpra.ub.uni-muenchen.de/8914/4/MPRA_paper_8914.pdf pg 18
        //S0 = 100, K = 100, r = 0.1, q = 0, C = 1, G = 5, M = 5, T = 1, Y=1.98
        let parameters = CGMYParameters {
            sigma: 0.0,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 1.98,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        };
        let strikes = vec![100.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.1;
        let asset = 100.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::CGMY(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 99.999905510, epsilon = 0.00001);
    }
    #[test]
    fn test_merton_price() {
        //https://www.upo.es/personal/jfernav/papers/Jumps_JOD_.pdf pg 8
        let sig_l = 0.05_f64.sqrt();
        let mu_l = -sig_l.powi(2) * 0.5;
        let parameters = MertonParameters {
            sigma: sig_l,
            lambda: 1.0,
            mu_l,
            sig_l,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        };
        let strikes = vec![35.0];
        let num_u: usize = 256;
        let t = 0.5;
        let rate = 0.1;
        let asset = 38.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::Merton(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 5.9713, epsilon = 0.0001);
    }
    #[test]
    fn test_merton_price_subset_heston() {
        //https://mpra.ub.uni-muenchen.de/8914/4/MPRA_paper_8914.pdf pg 15
        let sig_l = 0.0;
        let mu_l = 0.0;
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;
        let parameters = MertonParameters {
            sigma: b.sqrt(),
            lambda: 0.0,
            mu_l,
            sig_l,
            speed: a,
            v0: v0 / b,
            eta_v: c / b.sqrt(),
            rho,
        };
        let strikes = vec![100.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.0;
        let asset = 100.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::Merton(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 5.78515545, epsilon = 0.0001);
    }
    #[test]
    fn test_heston() {
        //https://mpra.ub.uni-muenchen.de/8914/4/MPRA_paper_8914.pdf pg 15
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;
        let parameters = HestonParameters {
            sigma: b.sqrt(),
            speed: a,
            v0,
            eta_v: c,
            rho,
        };
        let strikes = vec![100.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.0;
        let asset = 100.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::Heston(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        assert_abs_diff_eq!(results[0].value, 5.78515545, epsilon = 0.0001);
    }
    #[test]
    fn test_monte_carlo() {
        // ../../techdoc/OptionCalculation.Rnw
        let parameters = MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        let strikes = vec![50.0];
        let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let results = get_option_results_as_json(
            CALL_PRICE,
            false,
            &CFParameters::Merton(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
        //MC price is 4.793274
        assert!(results[0].value > 4.781525);
        assert!(results[0].value < 4.805023);
    }
    #[test]
    fn test_risk_measures() {
        //https://github.com/phillyfan1138/levy-functions/issues/27
        let parameters = MertonParameters {
            sigma: 0.3183,
            lambda: 0.204516,
            mu_l: -0.302967,
            sig_l: 0.220094,
            speed: 2.6726,
            v0: 0.237187,
            eta_v: 0.0,
            rho: -0.182754,
        };
        let num_u: usize = 256;
        let t = 0.187689;
        let rate = 0.004;
        let quantile = 0.01;
        let results = get_risk_measure_results_as_json(
            &CFParameters::Merton(parameters),
            5.0,
            num_u,
            t,
            rate,
            quantile,
        )
        .unwrap();
        assert_abs_diff_eq!(results.value_at_risk, 0.261503, epsilon = 0.00001);
    }
    #[test]
    fn test_error_for_out_of_bounds_constant() {
        let sig_l = 0.05_f64.sqrt();
        let mu_l = -sig_l.powi(2) * 0.5;
        let parameters = MertonParameters {
            sigma: sig_l,
            lambda: 1.0,
            mu_l,
            sig_l,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        };
        let strikes = vec![35.0];
        let num_u: usize = 256;
        let t = 0.5;
        let rate = 0.1;
        let asset = 38.0;
        let integer_that_is_not_an_option = -1;
        let results = get_option_results_as_json(
            integer_that_is_not_an_option,
            false,
            &CFParameters::Merton(parameters),
            10.0,
            num_u,
            asset,
            t,
            rate,
            &strikes,
        );
        assert!(results.is_err());
        assert_eq!(
            results.unwrap_err().to_string(),
            "Function indicator -1 does not exist."
        );
    }
}
