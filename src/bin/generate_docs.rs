use fang_oost_option::option_calibration::{OptionData, OptionDataMaturity};
#[macro_use]
extern crate rocket_contrib;
use std::fs::File;
use std::io::prelude::*;
use utils::calibration_maps::get_option_calibration_results_as_json;
use utils::constants::{CALL_PRICE, CGMY, HESTON, MERTON};
use utils::constraints::{CFParameters, CGMYParameters, HestonParameters, MertonParameters};
use utils::pricing_maps;
fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let print_choice: i32 = args[1].parse().unwrap();
    match print_choice {
        HESTON => {
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
            let heston_parameters = CFParameters::Heston(HestonParameters {
                sigma: b.sqrt(),
                v0: v0,
                speed: a,
                eta_v: c,
                rho,
            });
            let results = crate::pricing_maps::get_option_results_as_json(
                CALL_PRICE,
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
            println!("this is cost function heston: {}", result.final_cost_value);
            let json_results = json!(params);

            let json_actual = json!(heston_parameters);

            let mut file = File::create("./techdoc/techdoc_heston.json")?;
            file.write_all(json_results.to_string().as_bytes())?;
            let mut file = File::create("./techdoc/techdoc_heston_actual.json")?;
            file.write_all(json_actual.to_string().as_bytes())?;
            Ok(())
        }
        MERTON => {
            let stock = 178.46;
            let rate = 0.0;
            let maturity = 1.0;

            let sig_l = 0.05_f64.sqrt();
            let mu_l = -sig_l.powi(2) * 0.5;

            let strikes = vec![
                95.0, 100.0, 130.0, 150.0, 160.0, 165.0, 170.0, 175.0, 185.0, 190.0, 195.0, 200.0,
                210.0, 240.0, 250.0,
            ];
            let num_u = 256;
            let option_scale = 10.0;
            let merton_parameters = CFParameters::Merton(MertonParameters {
                sigma: sig_l,
                lambda: 1.0,
                mu_l,
                sig_l,
                speed: 0.0,
                v0: 1.0,
                eta_v: 0.0,
                rho: 0.0,
            });
            let results = crate::pricing_maps::get_option_results_as_json(
                CALL_PRICE,
                false,
                &merton_parameters,
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
            println!("this is cost function merton: {}", result.final_cost_value);
            let json_results = json!(params);

            let json_actual = json!(merton_parameters);

            let mut file = File::create("./techdoc/techdoc_merton.json")?;
            file.write_all(json_results.to_string().as_bytes())?;
            let mut file = File::create("./techdoc/techdoc_merton_actual.json")?;
            file.write_all(json_actual.to_string().as_bytes())?;
            Ok(())
        }
        CGMY => {
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
                y: 1.5, //was 1.98, but that is close to the edge (2)
                speed: 0.0,
                v0: 1.0,
                eta_v: 0.0,
                rho: 0.0,
            });
            let results = crate::pricing_maps::get_option_results_as_json(
                CALL_PRICE,
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
            println!("this is cost function cgmy: {}", result.final_cost_value);
            let params = params.unwrap();
            let json_results = json!(params);

            let json_actual = json!(cgmy_parameters);

            let mut file = File::create("./techdoc/techdoc_cgmy.json")?;
            file.write_all(json_results.to_string().as_bytes())?;
            let mut file = File::create("./techdoc/techdoc_cgmy_actual.json")?;
            file.write_all(json_actual.to_string().as_bytes())?;
            Ok(())
        }

        _ => {
            println!("Should not get here");
            Ok(())
        }
    }
}
