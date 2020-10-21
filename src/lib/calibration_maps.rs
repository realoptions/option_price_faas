use crate::constants::{CGMY, HESTON, MERTON};
use crate::constraints::{
    CFParameters, CGMYParameters, ErrorType, HestonParameters, MertonParameters, ParameterError,
    CGMY_CONSTRAINTS, HESTON_CONSTRAINTS, MERTON_CONSTRAINTS,
};
use argmin::prelude::*;
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin::solver::quasinewton::LBFGS;
use fang_oost_option::option_calibration::OptionDataMaturity;
use finitediff::FiniteDiff;
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
/** needed for calibration */
struct ObjFn<'a> {
    obj_fn: &'a (dyn Fn(&[f64]) -> f64 + Sync),
}

impl ArgminOp for ObjFn<'_> {
    type Param = Vec<f64>;
    type Output = f64;
    type Hessian = Vec<f64>;
    type Jacobian = ();
    type Float = f64;
    fn apply(&self, param: &Vec<f64>) -> Result<f64, Error> {
        let ofn = self.obj_fn;
        Ok(ofn(&param))
    }
    //this function only needed for lbfgs
    fn gradient(&self, param: &Vec<f64>) -> Result<Vec<f64>, Error> {
        let ofn = self.obj_fn;
        Ok((*param).central_diff(&|x| ofn(&x)))
    }
}

fn get_cgmy_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<cuckoo::UpperLower>,
) {
    let bounds = CGMY_CONSTRAINTS
        .to_vector()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| index < &5)
        .map(|(_, v)| convert_constraints_to_cuckoo_ul(&v))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [c, g, m, y, sigma, /*v0, speed, eta_v, rho*/] => (cf_functions::cgmy::cgmy_log_risk_neutral_cf(u,
                *c, *g, *m, *y, rate, *sigma /*, *v0, *speed, *eta_v, *rho,*/
            )*maturity).exp(),
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        bounds,
    )
}

fn get_merton_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<cuckoo::UpperLower>,
) {
    let bounds = MERTON_CONSTRAINTS
        .to_vector()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| index < &4)
        .map(|(_, v)| convert_constraints_to_cuckoo_ul(&v))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [lambda, mu_l, sig_l, sigma/*, v0, speed, eta_v, rho*/] => {
                (cf_functions::merton::merton_log_risk_neutral_cf(
                    u, *lambda, *mu_l, *sig_l, rate, *sigma, /*, *v0, *speed, *eta_v, *rho,*/
                ) * maturity)
                    .exp()
            }
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        bounds,
    )
}

fn get_heston_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<cuckoo::UpperLower>,
) {
    let bounds = HESTON_CONSTRAINTS
        .to_vector()
        .into_iter()
        .map(|v| convert_constraints_to_cuckoo_ul(&v))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [sigma, v0, speed, eta_v, rho] => {
                cf_functions::gauss::heston_cf(maturity, rate, *sigma, *v0, *speed, *eta_v, *rho)(u)
            }
            _ => {
                Complex::<f64>::new(0.0, 0.0) //can never get here
            }
        },
        bounds,
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
fn get_obj_fn_mse<'a, 'b: 'a, T, S>(
    option_datum: &'b [OptionDataMaturity],
    num_u: usize,
    asset: f64,
    rate: f64,
    get_max_strike: S,
    cf_fn: T,
) -> impl Fn(&[f64]) -> f64 + 'a
where
    T: Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + 'b + Sync,
    S: Fn(&[f64], f64) -> f64 + 'b + Sync,
{
    move |params| {
        fang_oost_option::option_calibration::obj_fn_real(
            num_u,
            asset,
            &option_datum,
            rate,
            &params,
            &get_max_strike,
            &cf_fn,
        )
    }
}

const NEST_SIZE: usize = 25;
const NUM_SIMS: usize = 1500; //this is super large, will likely never get there
const TOL: f64 = 0.00001; //doesn't need to be very accurate; just needs to get ballpark

fn optimize<T, S>(
    num_u: usize,
    asset: f64,
    option_data: &[OptionDataMaturity],
    ul: &[cuckoo::UpperLower],
    rate: f64,
    max_iter: u64,
    get_max_strike: T,
    cf_inst: S,
) -> Result<Vec<f64>, ParameterError>
where
    S: Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    T: Fn(&[f64], f64) -> f64 + Sync,
{
    let obj_fn = get_obj_fn_mse(&option_data, num_u, asset, rate, &get_max_strike, &cf_inst);
    //loop {
    let (optimal_parameters, _) = cuckoo::optimize(&obj_fn, ul, NEST_SIZE, NUM_SIMS, TOL, || {
        cuckoo::get_rng_system_seed()
    })?;

    let obj_fn = ObjFn { obj_fn: &obj_fn };
    let linesearch = MoreThuenteLineSearch::new();
    // m between 3 and 20 yield "good results" according to
    // http://www.apmath.spbu.ru/cnsa/pdf/monograf/Numerical_Optimization2006.pdf
    let solver = LBFGS::new(linesearch, 10).with_tol_cost(f64::EPSILON);
    let res = Executor::new(obj_fn, solver, optimal_parameters)
        .add_observer(ArgminSlogLogger::term(), ObserverMode::Always)
        .max_iters(max_iter)
        .run()?;
    //if res.state.iter != max_iter {
    Ok(res.state.best_param)
    //}
    //}
}
pub fn get_option_calibration_results_as_json(
    model_choice: i32,
    option_data: &[OptionDataMaturity],
    option_scale: f64,
    max_iter: u64,
    num_u: usize,
    asset: f64,
    rate: f64,
) -> Result<CFParameters, ParameterError> {
    match model_choice {
        CGMY => {
            let (cf_inst, bounds) = get_cgmy_calibration(rate);
            let get_max_strike = |params: &[f64], maturity| {
                let c = params[0];
                let g = params[1];
                let m = params[2];
                let y = params[3];
                let sigma = params[4];

                let vol = cf_functions::cgmy::cgmy_diffusion_vol(sigma, c, g, m, y, maturity);
                crate::pricing_maps::get_max_strike(asset, option_scale, vol)
            };
            let results = optimize(
                num_u,
                asset,
                &option_data,
                &bounds,
                rate,
                max_iter,
                &get_max_strike,
                &cf_inst,
            )?;
            match &results[..] {
                [c, g, m, y, sigma/*, v0, speed, eta_v, rho*/] => {
                    Ok(CFParameters::CGMY(CGMYParameters {
                        c: *c,
                        g: *g,
                        m: *m,
                        y: *y,
                        sigma: *sigma,
                        v0: 1.0,
                        speed: 0.0,
                        eta_v: 0.0,
                        rho: 0.0,
                    }))
                }
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        MERTON => {
            let (cf_inst, bounds) = get_merton_calibration(rate);
            let get_max_strike = |params: &[f64], maturity| {
                let lambda = params[0];
                let mu_l = params[1];
                let sig_l = params[2];
                let sigma = params[3];
                let vol =
                    cf_functions::merton::jump_diffusion_vol(sigma, lambda, mu_l, sig_l, maturity);
                crate::pricing_maps::get_max_strike(asset, option_scale, vol)
            };
            let results = optimize(
                num_u,
                asset,
                &option_data,
                &bounds,
                rate,
                max_iter,
                &get_max_strike,
                &cf_inst,
            )?;
            match &results[..] {
                [lambda, mu_l, sig_l, sigma/*, v0, speed, eta_v, rho*/] => {
                    Ok(CFParameters::Merton(MertonParameters {
                        lambda: *lambda,
                        mu_l: *mu_l,
                        sig_l: *sig_l,
                        sigma: *sigma,
                        v0: 1.0,
                        speed: 0.0,
                        eta_v: 0.0,
                        rho: 0.0,
                    }))
                }
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        HESTON => {
            let (cf_inst, bounds) = get_heston_calibration(rate);
            let get_max_strike = |params: &[f64], maturity: f64| {
                let sigma = params[0];
                let vol = sigma * maturity.sqrt();
                crate::pricing_maps::get_max_strike(asset, option_scale, vol)
            };
            let results = optimize(
                num_u,
                asset,
                &option_data,
                &bounds,
                rate,
                max_iter,
                &get_max_strike,
                &cf_inst,
            )?;
            match &results[..] {
                [sigma, v0, speed, eta_v, rho] => Ok(CFParameters::Heston(HestonParameters {
                    sigma: *sigma,
                    v0: *v0,
                    speed: *speed,
                    eta_v: *eta_v,
                    rho: *rho,
                })),
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
    //use crate::constraints::MERTON_CONSTRAINTS;
    use approx::*;
    use fang_oost_option::option_calibration::OptionData;
    //use fang_oost_option::option_pricing;
    //use rand::{distributions::Distribution, distributions::Uniform, SeedableRng, StdRng};
    //fn get_rng_seed(seed: [u8; 32]) -> StdRng {
    //    SeedableRng::from_seed(seed)
    //}
    ////fn get_over_region(lower: f64, upper: f64, rand: f64) -> f64 {
    //    lower + (upper - lower) * rand
    //}
    /*#[test]
    fn test_many_inputs_merton() {
        let seed: [u8; 32] = [2; 32];
        let mut rng_seed = get_rng_seed(seed);
        let uniform = Uniform::new(0.0f64, 1.0);
        let asset = 178.46;
        let num_u = 256;
        let option_scale = 10.0;
        let strikes = vec![
            95.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0, 210.0,
            240.0, 250.0,
        ];
        let maturity = 0.86;
        let rate = 0.02;
        let num_total: usize = 10;
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
            let vol = cf_functions::merton::jump_diffusion_vol(
                sigma_sim, lambda_sim, mu_l_sim, sig_l_sim, maturity,
            );
            let max_strike = crate::pricing_maps::get_max_strike(asset, option_scale, vol);
            let opt_prices = option_pricing::fang_oost_call_price(
                num_u, asset, &strikes, max_strike, rate, maturity, &inst_cf,
            );

            let option_data: Vec<OptionData> = opt_prices
                .iter()
                .zip(&strikes)
                .map(|(price, strike)| OptionData {
                    price: *price,
                    strike: *strike,
                })
                .collect();
            let option_data = vec![OptionDataMaturity {
                maturity,
                option_data,
            }];

            let result = get_option_calibration_results_as_json(
                MERTON,
                &option_data,
                option_scale,
                200,
                num_u,
                asset,
                rate,
            );
            match result {
                Ok(res) => {
                    let params = match res {
                        CFParameters::Merton(params) => Ok(params),
                        _ => Err("bad result"),
                    };
                    let params = params.unwrap();
                    assert_abs_diff_eq!(params.lambda, lambda_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.mu_l, mu_l_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.sig_l, sig_l_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.sigma, sigma_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.v0, v0_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.speed, speed_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.eta_v, eta_v_sim, epsilon = 0.01);
                    assert_abs_diff_eq!(params.rho, rho_sim, epsilon = 0.01);
                }
                Err(_) => {
                    num_bad = num_bad + 1;
                }
            }
        });
        let bad_rate = (num_bad as f64) / (num_total as f64);
        println!("Bad rate: {}", bad_rate);
        assert_eq!(bad_rate, 0.0);
    }*/
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
        );
        match result {
            Ok(res) => {
                let params = match res {
                    CFParameters::Heston(params) => Ok(params),
                    _ => Err("bad result"),
                };
                let params = params.unwrap();
                assert_abs_diff_eq!(params.sigma, b.sqrt(), epsilon = 0.01);
                assert_abs_diff_eq!(params.v0, v0, epsilon = 0.01);
                assert_abs_diff_eq!(params.speed, a, epsilon = 0.01);
                assert_abs_diff_eq!(params.eta_v, c, epsilon = 0.01);
                assert_abs_diff_eq!(params.rho, rho, epsilon = 0.01);
            }
            Err(e) => panic!(e),
        }
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
        );
        match result {
            Ok(res) => {
                let params = match res {
                    CFParameters::Merton(params) => Ok(params),
                    _ => Err("bad result"),
                };
                let params = params.unwrap();
                println!("lambda: {}", params.lambda);
                println!("mu_l: {}", params.mu_l);
                println!("sig_l: {}", params.sig_l);
                println!("sigma: {}", params.sigma);
                println!("v0: {}", params.v0);
                println!("speed: {}", params.speed);
                println!("eta_v: {}", params.eta_v);
                println!("rho: {}", params.rho);
                //if gets here, then its a win (it converged)
            }
            Err(e) => {
                println!("error!! {}", e);
                panic!(e)
            }
        }
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
        );
        match result {
            Ok(res) => {
                let params = match res {
                    CFParameters::CGMY(params) => Ok(params),
                    _ => Err("bad result"),
                };
                let params = params.unwrap();
                println!("c: {}", params.c);
                println!("g: {}", params.g);
                println!("m: {}", params.m);
                println!("y: {}", params.y);
                println!("sigma: {}", params.sigma);
                println!("v0: {}", params.v0);
                println!("speed: {}", params.speed);
                println!("eta_v: {}", params.eta_v);
                println!("rho: {}", params.rho);
                //if gets here, then its a win (it converged)
            }
            Err(e) => {
                println!("error!! {}", e);
                panic!(e)
            }
        }
    }
    /*#[test]
    fn test_heston_exact() {
        let stock = 178.46;
        let rate = 0.0;
        let maturity = 1.0;
        let b: f64 = 0.0398;
        let a = 1.5768;
        let c = 0.5751;
        let rho = -0.5711;
        let v0 = 0.0175;
        let sigma = b.sqrt();
        let speed = a;
        let eta_v = c;
        let strikes = vec![
            95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
            210.0, 240.0, 250.0,
        ];
        let num_u = 256;
        let option_scale = 10.0;
        let heston_parameters =
            crate::constraints::CFParameters::Heston(crate::constraints::HestonParameters {
                sigma,
                v0: v0,
                speed,
                eta_v,
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
                 }| OptionData {
                    price: *value,
                    strike: *at_point,
                },
            )
            .collect();

        let option_data = vec![OptionDataMaturity {
            maturity,
            option_data,
        }];
        let mut rnd = UnifRnd::new([42; 32]);
        let (cf_inst, _, bounds) = get_heston_calibration(rate);
        let get_max_strike = |params: &[f64], maturity: f64| {
            let sigma = params[0];
            let vol = sigma * maturity.sqrt();
            crate::pricing_maps::get_max_strike(stock, option_scale, vol)
        };
        let result = optimize(
            num_u,
            stock,
            &option_data,
            &bounds,
            rate,
            200,
            &get_max_strike,
            &cf_inst,
        )
        .unwrap();
        assert_abs_diff_eq!(result[0], sigma, epsilon = 0.01);
        assert_abs_diff_eq!(result[1], v0, epsilon = 0.01);
        assert_abs_diff_eq!(result[2], speed, epsilon = 0.01);
        assert_abs_diff_eq!(result[3], eta_v, epsilon = 0.01);
        assert_abs_diff_eq!(result[4], rho, epsilon = 0.01);
    }*/
}
