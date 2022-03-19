#![feature(test)]
extern crate test;
use test::Bencher;
extern crate num_complex;
extern crate utils;
use std::fmt;
use utils::{constants, constraints, pricing_maps};
#[bench]
fn bench_option_price_merton_u_128(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            128,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_merton_u_256(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_merton_u_512(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            512,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_merton_u_1024(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            1024,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_heston_u_128(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            128,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_heston_u_256(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_heston_u_512(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            512,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_heston_u_1024(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            1024,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_cgmy_u_128(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            128,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_cgmy_u_256(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_cgmy_u_512(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            512,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_cgmy_u_1024(b: &mut Bencher) {
    b.iter(|| {
        let strikes = vec![50.0];
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            1024,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[derive(Debug)]
enum StrikeScenario {
    OneStrike,
    SevenStrike,
    SixtyStrike,
}

impl fmt::Display for StrikeScenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
fn switch_strikes(scenario: &StrikeScenario) -> Vec<f64> {
    match scenario {
        StrikeScenario::OneStrike => vec![50.0],
        StrikeScenario::SevenStrike => vec![20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0],
        StrikeScenario::SixtyStrike => {
            let mut strikes: Vec<f64> = vec![];
            for i in 20..80 {
                strikes.push(i as f64);
            }
            strikes
        }
    }
}

#[bench]
fn bench_option_price_merton_strike_one(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::OneStrike);
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_merton_strike_seven(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SevenStrike);
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_merton_strike_sixty(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SixtyStrike);
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::MertonParameters {
            sigma: 0.2,
            lambda: 0.5,
            mu_l: -0.05,
            sig_l: 0.1,
            speed: 0.3,
            v0: 0.9,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Merton(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_heston_one_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::OneStrike);
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}
#[bench]
fn bench_option_price_heston_seven_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SevenStrike);
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}
#[bench]
fn bench_option_price_heston_sixty_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SixtyStrike);
        //let num_u: usize = 256;
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = constraints::HestonParameters {
            sigma: 0.2,
            speed: 0.3,
            v0: 0.2,
            eta_v: 0.2,
            rho: -0.5,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::Heston(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}

#[bench]
fn bench_option_price_cgmy_one_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::OneStrike);
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}
#[bench]
fn bench_option_price_cgmy_seven_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SevenStrike);
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}
#[bench]
fn bench_option_price_cgmy_sixty_strike(b: &mut Bencher) {
    b.iter(|| {
        let strikes = switch_strikes(&StrikeScenario::SixtyStrike);
        let t = 1.0;
        let rate = 0.03;
        let asset = 50.0;
        let parameters = crate::constraints::CGMYParameters {
            sigma: 0.2,
            c: 1.0,
            g: 5.0,
            m: 5.0,
            y: 0.5,
            speed: 0.1,
            v0: 1.0,
            eta_v: 0.2,
            rho: -0.1,
        };
        pricing_maps::get_option_results_as_json(
            constants::CALL_PRICE,
            false,
            &constraints::CFParameters::CGMY(parameters),
            10.0,
            256,
            asset,
            t,
            rate,
            &strikes,
        )
        .unwrap();
    });
}
