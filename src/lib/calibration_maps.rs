use crate::constants::{CGMY, HESTON, MERTON};
use crate::constraints::{
    CFParameters, CGMYConstraints, CGMYParameters, ErrorType, HestonConstraints, HestonParameters,
    MertonConstraints, MertonParameters, ParameterError, CGMY_CONSTRAINTS, HESTON_CONSTRAINTS,
    MERTON_CONSTRAINTS,
};
use argmin::prelude::*;
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin::solver::quasinewton::LBFGS;
use fang_oost_option::option_calibration::OptionDataMaturity;
use finitediff::FiniteDiff;
use num_complex::Complex;
use rand::{distributions::Distribution, distributions::Uniform, SeedableRng, StdRng};

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

    fn gradient(&self, param: &Vec<f64>) -> Result<Vec<f64>, Error> {
        let ofn = self.obj_fn;
        Ok((*param).central_diff(&|x| ofn(&x)))
    }
}

fn get_cgmy_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
) {
    let init_params = CGMY_CONSTRAINTS
        .to_vector()
        .into_iter()
        .map(|v| convert_constraints_to_mid(&v))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [c, g, m, y, sigma, v0, speed, eta_v, rho] => cf_functions::cgmy::cgmy_time_change_cf(
                maturity, rate, *c, *g, *m, *y, *sigma, *v0, *speed, *eta_v, *rho,
            )(u),
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        init_params,
    )
}

fn get_merton_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
) {
    let init_params = MERTON_CONSTRAINTS
        .to_vector()
        .into_iter()
        .map(|v| convert_constraints_to_mid(&v))
        .collect();
    (
        move |u: &Complex<f64>, maturity: f64, params: &[f64]| match params {
            [lambda, mu_l, sig_l, sigma, v0, speed, eta_v, rho] => {
                cf_functions::merton::merton_time_change_cf(
                    maturity, rate, *lambda, *mu_l, *sig_l, *sigma, *v0, *speed, *eta_v, *rho,
                )(u)
            }
            _ => {
                //can never get here
                Complex::<f64>::new(0.0, 0.0)
            }
        },
        init_params,
    )
}

fn get_heston_calibration(
    rate: f64,
) -> (
    impl Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    Vec<f64>,
) {
    let init_params = HESTON_CONSTRAINTS
        .to_vector()
        .into_iter()
        .map(|v| convert_constraints_to_mid(&v))
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
        init_params,
    )
}

fn convert_constraints_to_mid(constraint: &crate::constraints::ConstraintsSchema) -> f64 {
    (constraint.upper + constraint.lower) * 0.5
}
fn get_obj_fn_mse<'a, 'b: 'a, T>(
    option_datum: &'b [OptionDataMaturity],
    num_u: usize,
    asset: f64,
    max_strike: f64,
    rate: f64,
    cf_fn: T,
) -> impl Fn(&[f64]) -> f64 + 'a
where
    T: Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + 'b + Sync,
{
    move |params| {
        fang_oost_option::option_calibration::obj_fn_real(
            num_u,
            asset,
            &option_datum,
            max_strike,
            rate,
            &params,
            &cf_fn,
        )
    }
}
struct UnifRnd {
    rng_seed: StdRng,
    uniform: Uniform<f64>,
}

impl UnifRnd {
    fn new(seed: [u8; 32]) -> UnifRnd {
        UnifRnd {
            rng_seed: SeedableRng::from_seed(seed),
            uniform: Uniform::new(0.0f64, 1.0),
        }
    }
}
fn get_over_region(lower: f64, upper: f64, rand: f64) -> f64 {
    lower + (upper - lower) * rand
}
fn generate_cgmy_params(rnd: &mut UnifRnd, constraints: &CGMYConstraints) -> Vec<f64> {
    constraints
        .to_vector()
        .into_iter()
        .map(|constraint| {
            get_over_region(
                constraint.lower,
                constraint.upper,
                rnd.uniform.sample(&mut rnd.rng_seed),
            )
        })
        .collect()
}
fn generate_heston_params(rnd: &mut UnifRnd, constraints: &HestonConstraints) -> Vec<f64> {
    constraints
        .to_vector()
        .into_iter()
        .map(|constraint| {
            get_over_region(
                constraint.lower,
                constraint.upper,
                rnd.uniform.sample(&mut rnd.rng_seed),
            )
        })
        .collect()
}
fn generate_merton_params(rnd: &mut UnifRnd, constraints: &MertonConstraints) -> Vec<f64> {
    constraints
        .to_vector()
        .into_iter()
        .map(|constraint| {
            get_over_region(
                constraint.lower,
                constraint.upper,
                rnd.uniform.sample(&mut rnd.rng_seed),
            )
        })
        .collect()
}

const MAX_ACCEPTABLE_COST_FUNCTION_VALUE: f64 = 0.000001;
fn optimize<S, U>(
    num_u: usize,
    asset: f64,
    option_data: &[OptionDataMaturity],
    mut init_params: Vec<f64>,
    max_strike: f64,
    rate: f64,
    max_iter: u64,
    cf_inst: S,
    mut generate_new_params: U,
) -> Result<Vec<f64>, ParameterError>
where
    S: Fn(&Complex<f64>, f64, &[f64]) -> Complex<f64> + Sync,
    U: FnMut() -> Vec<f64> + Sync,
{
    let obj_fn = get_obj_fn_mse(&option_data, num_u, asset, max_strike, rate, &cf_inst);
    let mut all_good = false;
    for _ in 0..100 {
        let obj_fn = ObjFn { obj_fn: &obj_fn };
        let linesearch = MoreThuenteLineSearch::new();
        let solver = LBFGS::new(linesearch, 7);
        let res = Executor::new(obj_fn, solver, init_params)
            .max_iters(max_iter)
            .run();
        match res {
            Ok(result) => {
                if result.state.get_best_cost() > MAX_ACCEPTABLE_COST_FUNCTION_VALUE {
                    init_params = generate_new_params();
                } else {
                    init_params = result.state.get_best_param();
                    all_good = true;
                    break;
                }
            }
            Err(_) => {
                init_params = generate_new_params();
            }
        };
    }
    if all_good {
        Ok(init_params)
    } else {
        Err(ParameterError::new(&ErrorType::NoConvergence()))
    }
}
pub fn get_option_calibration_results_as_json(
    model_choice: i32,
    option_data: &[OptionDataMaturity],
    calibration_scale: f64,
    max_iter: u64,
    num_u: usize,
    asset: f64,
    rate: f64,
    seed: [u8; 32],
) -> Result<CFParameters, ParameterError> {
    let max_strike = asset * calibration_scale;
    let mut rnd = UnifRnd::new(seed);
    match model_choice {
        CGMY => {
            let (cf_inst, init_params) = get_cgmy_calibration(rate);
            let results = optimize(
                num_u,
                asset,
                &option_data,
                init_params,
                max_strike,
                rate,
                max_iter,
                &cf_inst,
                || generate_cgmy_params(&mut rnd, &CGMY_CONSTRAINTS),
            )?;
            match &results[..] {
                [c, g, m, y, sigma, v0, speed, eta_v, rho] => {
                    Ok(CFParameters::CGMY(CGMYParameters {
                        c: *c,
                        g: *g,
                        m: *m,
                        y: *y,
                        sigma: *sigma,
                        v0: *v0,
                        speed: *speed,
                        eta_v: *eta_v,
                        rho: *rho,
                    }))
                }
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        MERTON => {
            let (cf_inst, init_params) = get_merton_calibration(rate);
            let results = optimize(
                num_u,
                asset,
                &option_data,
                init_params,
                max_strike,
                rate,
                max_iter,
                &cf_inst,
                || generate_merton_params(&mut rnd, &MERTON_CONSTRAINTS),
            )?;
            match &results[..] {
                [lambda, mu_l, sig_l, sigma, v0, speed, eta_v, rho] => {
                    Ok(CFParameters::Merton(MertonParameters {
                        lambda: *lambda,
                        mu_l: *mu_l,
                        sig_l: *sig_l,
                        sigma: *sigma,
                        v0: *v0,
                        speed: *speed,
                        eta_v: *eta_v,
                        rho: *rho,
                    }))
                }
                _ => Err(ParameterError::new(&ErrorType::OutOfBounds(
                    "Calibration".to_string(),
                ))),
            }
        }
        HESTON => {
            let (cf_inst, init_params) = get_heston_calibration(rate);
            let results = optimize(
                num_u,
                asset,
                &option_data,
                init_params,
                max_strike,
                rate,
                max_iter,
                &cf_inst,
                || generate_heston_params(&mut rnd, &HESTON_CONSTRAINTS),
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
    use crate::constants::CALIBRATION_SCALAR;
    use crate::constraints::MERTON_CONSTRAINTS;
    use approx::*;
    use fang_oost_option::option_calibration::OptionData;
    use fang_oost_option::option_pricing;
    use rand::{distributions::Distribution, distributions::Uniform, SeedableRng, StdRng};
    fn get_rng_seed(seed: [u8; 32]) -> StdRng {
        SeedableRng::from_seed(seed)
    }
    fn get_over_region(lower: f64, upper: f64, rand: f64) -> f64 {
        lower + (upper - lower) * rand
    }
    #[test]
    fn test_many_inputs_merton() {
        let seed: [u8; 32] = [2; 32];
        let mut rng_seed = get_rng_seed(seed);
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
        let num_total: usize = 100;
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

            let option_data: Vec<OptionData> = opt_prices
                .iter()
                .zip(&strikes)
                .map(|(price, strike)| OptionData {
                    price: *price,
                    strike: *strike,
                })
                .collect();
            /*let iv: Vec<f64> = option_data
            .iter()
            .map(|OptionData { price, strike }| {
                black_scholes::call_iv(*price, asset, *strike, rate, maturity).unwrap()
            })
            .collect();*/
            let option_data = vec![OptionDataMaturity {
                maturity,
                option_data,
            }];

            let result = get_option_calibration_results_as_json(
                MERTON,
                &option_data,
                CALIBRATION_SCALAR,
                200,
                num_u,
                asset,
                rate,
                [42; 32],
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
    }
}
