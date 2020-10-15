/** needed for calibration */
struct CGMYCalibration {
    init_params: Vec<f64>,
    rate: f64,
}

impl ArgminOp for CGMYCalibration {
    type Param = Vec<f64>;
    type Output = f64;
    type Hessian = Vec<f64>;
    type Jacobian = ();
    type Float = f64;
    fn apply(&self, param: &Vec<f64>) -> Result<f64, Error> {
        let [c, g, m, y, sigma, v0, speed, eta_v, rho] = params;
        Ok(cf_functions::cgmy::cgmy_time_change_cf(
            maturity, self.rate, *c, *g, *m, *y, *sigma, *v0, *speed, *eta_v, *rho,
        )(u))
    }

    fn gradient(&self, param: &Vec<f64>) -> Result<Vec<f64>, Error> {
        let ofn = self.obj_fn;
        Ok((*param).central_diff(&|x| ofn(&x)))
    }
}
impl CGMYCalibration {
    fn new(&self) {
        let constraints = crate::constraints::get_cgmy_constraints();
        self.init_params = vec![
            convert_constraints_to_mid(&constraints.c),
            convert_constraints_to_mid(&constraints.g),
            convert_constraints_to_mid(&constraints.m),
            convert_constraints_to_mid(&constraints.y),
            convert_constraints_to_mid(&constraints.sigma),
            convert_constraints_to_mid(&constraints.v0),
            convert_constraints_to_mid(&constraints.speed),
            convert_constraints_to_mid(&constraints.eta_v),
            convert_constraints_to_mid(&constraints.rho),
        ];
    }
}
