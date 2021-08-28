| [Linux][lin-link] | [Codecov][cov-link] |
| :---------------: | :-----------------: |
| ![lin-badge]      | ![cov-badge]        |

[lin-badge]: https://github.com/realoptions/option_price_faas/workflows/test/badge.svg
[lin-link]:  https://github.com/realoptions/option_price_faas/actions
[cov-badge]: https://codecov.io/gh/realoptions/option_price_faas/branch/master/graph/badge.svg
[cov-link]:  https://codecov.io/gh/realoptions/option_price_faas

## Option Price FAAS

### API Documentation


Swagger docs at [Docs](https://developer.finside.org/).

Rapid API at https://rapidapi.com/realoptions-realoptions-default/api/finside1. 

### Pricer
These are a set of functions for pricing options when assets follow an extended Jump Diffusion process with stochastic time clock correlated with the diffusion portion of the asset process. See [Carr and Wu 2004](http://faculty.baruch.cuny.edu/lwu/papers/timechangeLevy_JFE2004.pdf) and [Huang and Wu 2004](https://pdfs.semanticscholar.org/0065/9b64e38e097f9df521ea5393ede9a2b6f824.pdf?_ga=2.75168529.2091536158.1531661727-680909490.1531661727).

### More documentation/design evidence

There are documents in the [technical documentation](./techdoc) folder.

To generate the needed json files, build the project `cargo build --release`, then run `node index`.  Open [OptionCalibration](./docs/OptionCalibration.rnw) in a Sweave/Latex editor (eg RStudio) and compile.

#RUSTFLAGS='-L my/lib/location' cargo build

### Run functions locally

`cargo build --release`

`ROCKET_PORT=8080 ROCKET_ADDRESS=0.0.0.0 MAJOR_VERSION=v2 ./target/release/option_price`

## Benchmarks

View benchmarks at https://realoptions.github.io/option_price_faas/report.
