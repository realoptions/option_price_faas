use crate::constants::{CGMY, HESTON, MERTON};
use crate::constraints::{
    CFParameters, CGMYParameters, CalibrationResponse, ErrorType, HestonParameters,
    MertonParameters, ParameterError, CGMY_CONSTRAINTS, HESTON_CONSTRAINTS, MERTON_CONSTRAINTS,
};
use fang_oost_option::option_calibration::OptionDataMaturity;
use finitediff::FiniteDiff;
use liblbfgs::lbfgs;
use nlopt::Nlopt;
use num_complex::Complex;
use rand::distributions::Distribution;
use rand::distributions::Uniform;
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

const CGMY_SIGMA: f64 = 0.0;
fn get_cgmy_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
    Vec<f64>,
) {
    let mut lower_bounds: Vec<f64> = vec![];
    let mut upper_bounds: Vec<f64> = vec![];

    for (_, crate::constraints::ConstraintsSchema { lower, upper, .. }) in CGMY_CONSTRAINTS
        .to_vector()
        .iter()
        .enumerate()
        .filter(|(index, _)| index < &4)
    {
        lower_bounds.push(*lower);
        upper_bounds.push(*upper);
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
        lower_bounds,
        upper_bounds,
    )
}

fn get_merton_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
    Vec<f64>,
) {
    let mut lower_bounds: Vec<f64> = vec![];
    let mut upper_bounds: Vec<f64> = vec![];

    for (_, crate::constraints::ConstraintsSchema { lower, upper, .. }) in MERTON_CONSTRAINTS
        .to_vector()
        .iter()
        .enumerate()
        .filter(|(index, _)| index < &4)
    {
        lower_bounds.push(*lower);
        upper_bounds.push(*upper);
    }
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
        lower_bounds,
        upper_bounds,
    )
}

fn get_heston_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
    Vec<f64>,
) {
    let mut lower_bounds: Vec<f64> = vec![];
    let mut upper_bounds: Vec<f64> = vec![];

    for crate::constraints::ConstraintsSchema { lower, upper, .. } in
        HESTON_CONSTRAINTS.to_vector().iter()
    {
        lower_bounds.push(*lower);
        upper_bounds.push(*upper);
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
        lower_bounds,
        upper_bounds,
    )
}

fn convert_constraints_to_cuckoo_ul(
    constraint: &crate::constraints::ConstraintsSchema,
) -> cuckoo::UpperLower {
    cuckoo::UpperLower {
        upper: constraint.upper,
        lower: constraint.lower,
    }
}

const NEST_SIZE: usize = 25;
const NUM_SIMS: usize = 500;

//it may not get to this accuracy before NUM_SIMS, but the
//more accurate we can make it the better the LBFGS performs
const TOL: f64 = 0.0000001;

fn optimize<T>(
    ul: &[cuckoo::UpperLower],
    max_iter: usize,
    obj_fn: T,
) -> Result<(Vec<f64>, f64), ParameterError>
where
    T: Fn(&[f64]) -> f64 + Sync,
{
    let (mut optimal_parameters, _) =
        cuckoo::optimize(&obj_fn, ul, NEST_SIZE, NUM_SIMS, TOL, || {
            cuckoo::get_rng_system_seed()
        })?;

    let evaluate = |x: &[f64], gx: &mut [f64]| {
        for (((index, gradient), parameter), upper_lower) in x
            .to_vec()
            .central_diff(&|x| obj_fn(&x))
            .iter()
            .enumerate()
            .zip(x)
            .zip(ul)
        {
            if parameter >= &upper_lower.upper || parameter <= &upper_lower.lower {
                // forcing a negative gradient if near the edges to
                // push it back into the correct space
                gx[index] = -1.0;
            } else {
                gx[index] = *gradient;
            }
        }
        Ok(obj_fn(x))
    };
    let mut fn_val = 0.0;
    let _result = lbfgs()
        .with_max_iterations(max_iter)
        .with_epsilon(f64::EPSILON)
        .with_gradient_only()
        .minimize(
            &mut optimal_parameters, // input variables
            evaluate,                // define how to evaluate function
            |prgr| {
                fn_val = prgr.fx;
                false
            },
        );
    Ok((optimal_parameters, fn_val))
}

fn optimize_cgmy<T>(
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    max_iter: usize,
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

    optim.set_upper_bounds(&upper_bounds).unwrap();
    optim.set_lower_bounds(&lower_bounds).unwrap();
    optim.set_xtol_rel(f64::EPSILON).unwrap();

    let mut init: Vec<f64> = upper_bounds
        .iter()
        .zip(lower_bounds)
        .map(|(upper, lower)| (upper + lower) / 2.0)
        .collect();
    let (_, mut result) = optim.optimize(&mut init).unwrap();
    let uniform = Uniform::new(0.0f64, 1.0);
    let mut local_vec = init.clone();
    let mut rng = cuckoo::get_rng_system_seed();
    (0..max_iter).for_each(|_i| {
        for ((local_element, lower), upper) in
            local_vec.iter_mut().zip(lower_bounds).zip(upper_bounds)
        {
            *local_element = uniform.sample(&mut rng) * (upper - lower) + lower;
        }
        let (_, local_result) = optim.optimize(&mut local_vec).unwrap();
        if local_result < result {
            for (init_el, local_element) in init.iter_mut().zip(&local_vec) {
                *init_el = *local_element;
            }
            result = local_result;
        }
    });

    Ok((init, result))
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
    max_iter: usize,
    num_u: usize,
    asset: f64,
    rate: f64,
) -> Result<CalibrationResponse, ParameterError> {
    match model_choice {
        CGMY => {
            let (cf_inst, lower_bounds, upper_bounds) = get_cgmy_calibration(rate);
            let get_max_strike = cgmy_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            //let (results, fn_value) = optimize(&bounds, max_iter, &obj_fn)?;
            let (results, fn_value) =
                optimize_cgmy(&lower_bounds, &upper_bounds, max_iter, &obj_fn)?;
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
            let (cf_inst, lower_bounds, upper_bounds) = get_merton_calibration(rate);
            let get_max_strike = merton_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            let (results, fn_value) =
                optimize_cgmy(&lower_bounds, &upper_bounds, max_iter, &obj_fn)?;
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
            let (cf_inst, lower_bounds, upper_bounds) = get_heston_calibration(rate);
            let get_max_strike = heston_max_strike(asset, option_scale);
            let obj_fn = fang_oost_option::option_calibration::obj_fn_real(
                num_u,
                asset,
                &option_data,
                rate,
                &get_max_strike,
                &cf_inst,
            );
            let (results, fn_value) =
                optimize_cgmy(&lower_bounds, &upper_bounds, max_iter, &obj_fn)?;
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
            400,
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
                    println!("price: {}", value);
                    println!("strike: {}", at_point);
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
            400,
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
                    println!("price: {}", value);
                    println!("strike: {}", at_point);
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
            400,
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
        let (cf_inst, _lower_bounds, _upper_bounds) = get_cgmy_calibration(rate);
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
}
