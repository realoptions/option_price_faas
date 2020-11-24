use crate::constants::{CGMY, HESTON, MERTON};
use crate::constraints::{
    CFParameters, CGMYParameters, CalibrationResponse, ErrorType, HestonParameters,
    MertonParameters, ParameterError, CGMY_CONSTRAINTS, HESTON_CONSTRAINTS, MERTON_CONSTRAINTS,
};
use fang_oost_option::option_calibration::OptionDataMaturity;

use nlopt::Nlopt;
use num_complex::Complex;
pub fn get_model_indicators(option_type: &str) -> Result<i32, ParameterError> {
    match option_type {
        "cgmy" => Ok(CGMY),
        "heston" => Ok(HESTON),
        "merton" => Ok(MERTON),
        _ => Err(ParameterError::new(&ErrorType::FunctionError(
            option_type.to_string(),
        ))),
    }
}
pub fn get_num_iter(option_type: &str) -> Result<usize, ParameterError> {
    match option_type {
        "cgmy" => Ok(8),
        "heston" => Ok(4),
        "merton" => Ok(1),
        _ => Err(ParameterError::new(&ErrorType::FunctionError(
            option_type.to_string(),
        ))),
    }
}

const CGMY_SIGMA: f64 = 0.0;
const CGMY_PARAMETER_NUM: usize = 4;
const CGMY_LOWER_BOUNDS: [f64; CGMY_PARAMETER_NUM] = [
    CGMY_CONSTRAINTS.c.lower,
    CGMY_CONSTRAINTS.g.lower,
    CGMY_CONSTRAINTS.m.lower,
    CGMY_CONSTRAINTS.y.lower,
];
const CGMY_UPPER_BOUNDS: [f64; CGMY_PARAMETER_NUM] = [
    CGMY_CONSTRAINTS.c.upper,
    CGMY_CONSTRAINTS.g.upper,
    CGMY_CONSTRAINTS.m.upper,
    CGMY_CONSTRAINTS.y.upper,
];
const HESTON_PARAMETER_NUM: usize = 5;
const HESTON_LOWER_BOUNDS: [f64; HESTON_PARAMETER_NUM] = [
    HESTON_CONSTRAINTS.sigma.lower,
    HESTON_CONSTRAINTS.v0.lower,
    HESTON_CONSTRAINTS.speed.lower,
    HESTON_CONSTRAINTS.eta_v.lower,
    HESTON_CONSTRAINTS.rho.lower,
];
const HESTON_UPPER_BOUNDS: [f64; HESTON_PARAMETER_NUM] = [
    HESTON_CONSTRAINTS.sigma.upper,
    HESTON_CONSTRAINTS.v0.upper,
    HESTON_CONSTRAINTS.speed.upper,
    HESTON_CONSTRAINTS.eta_v.upper,
    HESTON_CONSTRAINTS.rho.upper,
];
const MERTON_PARAMETER_NUM: usize = 4;
const MERTON_LOWER_BOUNDS: [f64; MERTON_PARAMETER_NUM] = [
    MERTON_CONSTRAINTS.lambda.lower,
    MERTON_CONSTRAINTS.mu_l.lower,
    MERTON_CONSTRAINTS.sig_l.lower,
    MERTON_CONSTRAINTS.sigma.lower,
];
const MERTON_UPPER_BOUNDS: [f64; MERTON_PARAMETER_NUM] = [
    MERTON_CONSTRAINTS.lambda.upper,
    MERTON_CONSTRAINTS.mu_l.upper,
    MERTON_CONSTRAINTS.sig_l.upper,
    MERTON_CONSTRAINTS.sigma.upper,
];

fn get_dx(lower: f64, upper: f64, num_steps: usize) -> f64 {
    (upper - lower) / (num_steps as f64)
}

fn get_midpoint(lower: f64, upper: f64) -> f64 {
    (upper + lower) / 2.0
}

fn get_element(lower: f64, dx: f64, index: usize) -> f64 {
    lower + dx * (index as f64)
}

fn get_cgmy_calibration(
    rate: f64,
    num_steps: usize,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<Vec<f64>>,
) {
    let mut init: Vec<f64> = CGMY_UPPER_BOUNDS
        .iter()
        .zip(CGMY_LOWER_BOUNDS.iter())
        .map(|(upper, lower)| get_midpoint(*lower, *upper))
        .collect();
    let dc = get_dx(
        CGMY_CONSTRAINTS.y.lower,
        CGMY_CONSTRAINTS.y.upper,
        num_steps,
    );
    let mut results: Vec<Vec<f64>> = vec![];
    for i in 1..num_steps {
        init[3] = get_element(CGMY_CONSTRAINTS.y.lower, dc, i);
        results.push(init.clone());
    }

    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [c, g, m, y] => {
                (cf_functions::cgmy::cgmy_log_risk_neutral_cf(u, *c, *g, *m, *y, rate, CGMY_SIGMA)
                    * maturity)
                    .exp()
            }
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        results,
    )
}

fn get_merton_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<Vec<f64>>,
) {
    let init: Vec<f64> = MERTON_UPPER_BOUNDS
        .iter()
        .zip(MERTON_LOWER_BOUNDS.iter())
        .map(|(upper, lower)| get_midpoint(*lower, *upper))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [lambda, mu_l, sig_l, sigma] => (cf_functions::merton::merton_log_risk_neutral_cf(
                u, *lambda, *mu_l, *sig_l, rate, *sigma,
            ) * maturity)
                .exp(),
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        vec![init],
    )
}

fn get_heston_calibration(
    rate: f64,
    num_steps: usize,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<Vec<f64>>,
) {
    let mut init: Vec<f64> = HESTON_UPPER_BOUNDS
        .iter()
        .zip(HESTON_LOWER_BOUNDS.iter())
        .map(|(upper, lower)| get_midpoint(*lower, *upper))
        .collect();
    let dsigma = get_dx(
        HESTON_CONSTRAINTS.sigma.lower,
        HESTON_CONSTRAINTS.sigma.upper,
        num_steps,
    );
    let mut results: Vec<Vec<f64>> = vec![];
    for i in 1..num_steps {
        init[0] = get_element(HESTON_CONSTRAINTS.sigma.lower, dsigma, i);
        results.push(init.clone());
    }
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [sigma, v0, speed, eta_v, rho] => {
                cf_functions::gauss::heston_cf(maturity, rate, *sigma, *v0, *speed, *eta_v, *rho)(u)
            }
            _ => {
                Complex::<f64>::new(0.0, 0.0) //can never get here
            }
        },
        results,
    )
}
//strategy: iterate over "problematic" paramter (for heston this is sigma, for CGMY its "c")
fn optimize<T>(
    mut starting_points: Vec<Vec<f64>>, //consume starting_points
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    obj_fn: T,
) -> Result<(Vec<f64>, f64), ParameterError>
where
    T: Fn(&[f64]) -> f64 + Sync,
{
    let mut optim = Nlopt::<_, ()>::new(
        nlopt::Algorithm::Neldermead,
        lower_bounds.len(),
        |x: &[f64], _grad: Option<&mut [f64]>, _user_data: &mut ()| obj_fn(x),
        nlopt::Target::Minimize,
        (),
    );

    optim.set_upper_bounds(&upper_bounds)?;
    optim.set_lower_bounds(&lower_bounds)?;
    optim.set_xtol_rel(f64::EPSILON)?;

    let mut result = f64::INFINITY;
    let mut best_index: usize = 0;
    for (index, mut starting_point) in starting_points.iter_mut().enumerate() {
        let (_, local_result) = optim
            .optimize(&mut starting_point)
            .map_err(|(err, _v)| err)?;

        if local_result < result {
            best_index = index;
            result = local_result;
        }
    }
    Ok((starting_points.swap_remove(best_index), result))
}
fn cgmy_max_strike(asset: f64, option_scale: f64) -> impl Fn(&[f64], f64) -> f64 + Sync {
    move |params: &[f64], maturity: f64| {
        let c = params[0];
        let g = params[1];
        let m = params[2];
        let y = params[3];
        //let sigma = params[4];

        let vol = cf_functions::cgmy::cgmy_diffusion_vol(CGMY_SIGMA, c, g, m, y, maturity);
        crate::pricing_maps::get_max_strike(asset, option_scale, vol)
    }
}
fn merton_max_strike(asset: f64, option_scale: f64) -> impl Fn(&[f64], f64) -> f64 + Sync {
    move |params: &[f64], maturity: f64| {
        let lambda = params[0];
        let mu_l = params[1];
        let sig_l = params[2];
        let sigma = params[3];
        let vol = cf_functions::merton::jump_diffusion_vol(sigma, lambda, mu_l, sig_l, maturity);
        crate::pricing_maps::get_max_strike(asset, option_scale, vol)
    }
}
fn heston_max_strike(asset: f64, option_scale: f64) -> impl Fn(&[f64], f64) -> f64 + Sync {
    move |params: &[f64], maturity: f64| {
        let sigma = params[0];
        let vol = sigma * maturity.sqrt();
        crate::pricing_maps::get_max_strike(asset, option_scale, vol)
    }
}
pub fn get_option_calibration_results_as_json(
    model_choice: i32,
    option_data: &[OptionDataMaturity],
    option_scale: f64,
    num_iter: usize,
    num_u: usize,
    asset: f64,
    rate: f64,
) -> Result<CalibrationResponse, ParameterError> {
    match model_choice {
        CGMY => {
            let (cf_inst, starting_points) = get_cgmy_calibration(rate, num_iter);
            let get_max_strike = cgmy_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            let (results, fn_value) = optimize(
                starting_points,
                &CGMY_LOWER_BOUNDS,
                &CGMY_UPPER_BOUNDS,
                &obj_fn,
            )?;
            match &results[..] {
                [c, g, m, y] => Ok(CalibrationResponse {
                    parameters: CFParameters::CGMY(CGMYParameters {
                        c: *c,
                        g: *g,
                        m: *m,
                        y: *y,
                        sigma: CGMY_SIGMA,
                        v0: 1.0,
                        speed: 0.0,
                        eta_v: 0.0,
                        rho: 0.0,
                    }),
                    final_cost_value: fn_value,
                }),
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        MERTON => {
            let (cf_inst, starting_points) = get_merton_calibration(rate);
            let get_max_strike = merton_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            let (results, fn_value) = optimize(
                starting_points,
                &MERTON_LOWER_BOUNDS,
                &MERTON_UPPER_BOUNDS,
                &obj_fn,
            )?;
            match &results[..] {
                [lambda, mu_l, sig_l, sigma] => Ok(CalibrationResponse {
                    parameters: CFParameters::Merton(MertonParameters {
                        lambda: *lambda,
                        mu_l: *mu_l,
                        sig_l: *sig_l,
                        sigma: *sigma,
                        v0: 1.0,
                        speed: 0.0,
                        eta_v: 0.0,
                        rho: 0.0,
                    }),
                    final_cost_value: fn_value,
                }),
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        HESTON => {
            let (cf_inst, starting_points) = get_heston_calibration(rate, num_iter);
            let get_max_strike = heston_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            let (results, fn_value) = optimize(
                starting_points,
                &HESTON_LOWER_BOUNDS,
                &HESTON_UPPER_BOUNDS,
                &obj_fn,
            )?;
            match &results[..] {
                [sigma, v0, speed, eta_v, rho] => Ok(CalibrationResponse {
                    parameters: CFParameters::Heston(HestonParameters {
                        sigma: *sigma,
                        v0: *v0,
                        speed: *speed,
                        eta_v: *eta_v,
                        rho: *rho,
                    }),
                    final_cost_value: fn_value,
                }),
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        _ => Err(ParameterError::new(&ErrorType::FunctionError(format!(
            "{}",
            model_choice
        )))),
    }
}

#[cfg(test)]
mod tests {
    use crate::calibration_maps::*;
    use fang_oost_option::option_calibration::OptionData;
    #[test]
    fn test_heston() {
        let stock = 178.46;
        let rate = 0.0;
        let maturity = 1.0;
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;

        let strikes = vec![
            95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
            210.0, 240.0, 250.0,
        ];
        let num_u = 256;
        let option_scale = 10.0;
        let heston_parameters =
            crate::constraints::CFParameters::Heston(crate::constraints::HestonParameters {
                sigma: b.sqrt(),
                v0: v0,
                speed: a,
                eta_v: c,
                rho,
            });
        let results = crate::pricing_maps::get_option_results_as_json(
            crate::constants::CALL_PRICE,
            false,
            &heston_parameters,
            option_scale,
            num_u,
            stock,
            maturity,
            rate,
            &strikes,
        )
        .unwrap();

        let option_data: Vec<OptionData> = results
            .iter()
            .map(
                |crate::pricing_maps::GraphElement {
                     at_point, value, ..
                 }| {
                    OptionData {
                        price: *value,
                        strike: *at_point,
                    }
                },
            )
            .collect();

        let option_data = vec![OptionDataMaturity {
            maturity,
            option_data,
        }];

        let result = get_option_calibration_results_as_json(
            HESTON,
            &option_data,
            option_scale,
            4,
            num_u,
            stock,
            rate,
        )
        .unwrap();
        let params = match result.parameters {
            CFParameters::Heston(params) => Ok(params),
            _ => Err("bad result"),
        };
        let params = params.unwrap();
        assert!(params.v0 > 0.0);
        assert!(params.sigma > 0.0);
        assert!(params.speed > 0.0);
        assert!(params.eta_v > 0.0);
        assert!(params.rho > -1.0);
        assert!(params.rho < 1.0);
        println!("sigma: {}", params.sigma);
        println!("v0: {}", params.v0);
        println!("speed: {}", params.speed);
        println!("eta_v: {}", params.eta_v);
        println!("rho: {}", params.rho);
    }
    #[test]
    fn test_merton() {
        let stock = 178.46;
        let rate = 0.0;
        let maturity = 1.0;
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;

        let strikes = vec![
            95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
            210.0, 240.0, 250.0,
        ];
        let num_u = 256;
        let option_scale = 10.0;
        let heston_parameters =
            crate::constraints::CFParameters::Heston(crate::constraints::HestonParameters {
                sigma: b.sqrt(),
                v0: v0,
                speed: a,
                eta_v: c,
                rho,
            });
        let results = crate::pricing_maps::get_option_results_as_json(
            crate::constants::CALL_PRICE,
            false,
            &heston_parameters,
            option_scale,
            num_u,
            stock,
            maturity,
            rate,
            &strikes,
        )
        .unwrap();

        let option_data: Vec<OptionData> = results
            .iter()
            .map(
                |crate::pricing_maps::GraphElement {
                     at_point, value, ..
                 }| {
                    OptionData {
                        price: *value,
                        strike: *at_point,
                    }
                },
            )
            .collect();

        let option_data = vec![OptionDataMaturity {
            maturity,
            option_data,
        }];

        let result = get_option_calibration_results_as_json(
            MERTON,
            &option_data,
            option_scale,
            4,
            num_u,
            stock,
            rate,
        )
        .unwrap();
        let params = match result.parameters {
            CFParameters::Merton(params) => Ok(params),
            _ => Err("bad result"),
        };
        let params = params.unwrap();
        assert!(params.lambda > 0.0);
        assert!(params.sig_l > 0.0);
        assert!(params.sigma > 0.0);
        println!("lambda: {}", params.lambda);
        println!("mu_l: {}", params.mu_l);
        println!("sig_l: {}", params.sig_l);
        println!("sigma: {}", params.sigma);
        println!("v0: {}", params.v0);
        println!("speed: {}", params.speed);
        println!("eta_v: {}", params.eta_v);
        println!("rho: {}", params.rho);
    }
    #[test]
    fn test_cgmy() {
        let stock = 178.46;
        let rate = 0.0;
        let maturity = 1.0;
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;

        let strikes = vec![
            95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
            210.0, 240.0, 250.0,
        ];
        let num_u = 256;
        let option_scale = 10.0;
        let heston_parameters =
            crate::constraints::CFParameters::Heston(crate::constraints::HestonParameters {
                sigma: b.sqrt(),
                v0: v0,
                speed: a,
                eta_v: c,
                rho,
            });
        let results = crate::pricing_maps::get_option_results_as_json(
            crate::constants::CALL_PRICE,
            false,
            &heston_parameters,
            option_scale,
            num_u,
            stock,
            maturity,
            rate,
            &strikes,
        )
        .unwrap();

        let option_data: Vec<OptionData> = results
            .iter()
            .map(
                |crate::pricing_maps::GraphElement {
                     at_point, value, ..
                 }| {
                    OptionData {
                        price: *value,
                        strike: *at_point,
                    }
                },
            )
            .collect();

        let option_data = vec![OptionDataMaturity {
            maturity,
            option_data,
        }];

        let result = get_option_calibration_results_as_json(
            CGMY,
            &option_data,
            option_scale,
            4,
            num_u,
            stock,
            rate,
        )
        .unwrap();
        let params = match result.parameters {
            CFParameters::CGMY(params) => Ok(params),
            _ => Err("bad result"),
        };
        let params = params.unwrap();
        assert!(params.c > 0.0);
        assert!(params.g > 0.0);
        assert!(params.m > 0.0);
        //assert!(params.sigma > 0.0);
        println!("c: {}", params.c);
        println!("g: {}", params.g);
        println!("m: {}", params.m);
        println!("y: {}", params.y);
        println!("sigma: {}", params.sigma);
        println!("v0: {}", params.v0);
        println!("speed: {}", params.speed);
        println!("eta_v: {}", params.eta_v);
        println!("rho: {}", params.rho);
    }
    #[test]
    fn test_cgmy_recover_exact() {
        let stock = 178.46;
        let rate = 0.0;
        let maturity = 1.0;

        let strikes = vec![
            95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
            210.0, 240.0, 250.0,
        ];
        let num_u = 256;
        let option_scale = 10.0;
        let cgmy_parameters = CFParameters::CGMY(CGMYParameters {
            sigma: 0.0,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 1.5,
            speed: 0.0,
            v0: 1.0,
            eta_v: 0.0,
            rho: 0.0,
        });
        let results = crate::pricing_maps::get_option_results_as_json(
            crate::constants::CALL_PRICE,
            false,
            &cgmy_parameters,
            option_scale,
            num_u,
            stock,
            maturity,
            rate,
            &strikes,
        )
        .unwrap();

        let option_data: Vec<OptionData> = results
            .iter()
            .map(
                |crate::pricing_maps::GraphElement {
                     at_point, value, ..
                 }| {
                    OptionData {
                        price: *value,
                        strike: *at_point,
                    }
                },
            )
            .collect();

        let option_data = vec![OptionDataMaturity {
            maturity,
            option_data,
        }];
        let (cf_inst, _starting_points) = get_cgmy_calibration(rate, 1);
        let get_max_strike = cgmy_max_strike(stock, option_scale);
        let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
            num_u,
            stock,
            &option_data,
            rate,
            &get_max_strike,
            &cf_inst,
        );
        let cgmy_params_unwrap = (match cgmy_parameters {
            CFParameters::CGMY(params) => Ok(params),
            _ => Err("Cant get here"),
        })
        .unwrap();
        let values = vec![
            cgmy_params_unwrap.c,
            cgmy_params_unwrap.g,
            cgmy_params_unwrap.m,
            cgmy_params_unwrap.y,
            //cgmy_params_unwrap.sigma,
        ];
        let obj_value = obj_fn(&values);

        assert_eq!(obj_value, 0.0);
    }
    #[test]
    fn test_cgmy_init_values() {
        let (_cf, init_values) = get_cgmy_calibration(0.0, 4);
        assert!(init_values[0][3] != init_values[1][3]);
        assert!(init_values[2][3] != init_values[1][3]);
    }
    #[test]
    fn test_heston_init_values() {
        let (_cf, init_values) = get_heston_calibration(0.0, 4);
        assert_eq!(init_values[0][0], 0.25);
        assert_eq!(init_values[1][0], 0.5);
        assert_eq!(init_values[2][0], 0.75);
    }
}
