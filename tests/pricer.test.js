'use strict'
const fetch = require('node-fetch')
const { location, timeout } = require('./binaryLocation.json')
const { spawn } = require('child_process')
jest.setTimeout(timeout)
let server
const port = '9000'
const version = 'v2'
beforeAll((done) => {
    server = spawn(location, [], { env: { ROCKET_PORT: port, ROCKET_ADDRESS: "0.0.0.0", MAJOR_VERSION: version } })
    setTimeout(done, 1000) //wait for server to launch
})

afterAll(() => {
    server.kill()
})
describe('option prices', () => {
    it('returns array of value and points', () => {
        const body = {
            num_u: 8,
            rate: 0.1,
            maturity: 0.5,
            asset: 38,
            cf_parameters: { sigma: 0.5, speed: 0.1, v0: 0.2, eta_v: 0.1, rho: -0.5 },
            strikes: [100],
            quantile: 0.01
        }
        return fetch(
            `http://127.0.0.1:${port}/v2/heston/calculator/put/price`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return Promise.all([
                expect(Array.isArray(response)),
                expect(response[0].value).toBeDefined(),
                expect(response[0].at_point).toBeDefined()
            ])
        })

    })
    it('returns array of value, points, and iv', () => {
        const body = {
            num_u: 8,
            rate: 0.1,
            maturity: 0.5,
            asset: 38,
            cf_parameters: { sigma: 0.5, speed: 0.1, v0: 0.2, eta_v: 0.1, rho: -0.5 },
            strikes: [100],
            quantile: 0.01
        }
        return fetch(
            `http://127.0.0.1:${port}/v2/heston/calculator/put/price?include_implied_volatility=true`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return Promise.all([
                expect(Array.isArray(response)),
                expect(response[0].value).toBeDefined(),
                expect(response[0].at_point).toBeDefined(),
                expect(response[0].iv).toBeTruthy()
            ])
        })
    })
    it('returns error if not all parameters included', () => {
        const body = {
            num_u: 8,
            rate: 0.1,
            maturity: 0.5,
            asset: 38,
            cf_parameters: { sigma: 0.5, speed: 0.1, v0: 0.2, eta_v: 0.1, rho: -0.5 },
            quantile: 0.01
        }
        return fetch(
            `http://127.0.0.1:${port}/v2/heston/calculator/put/price?include_implied_volatility=true`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.err).toEqual("Parameter strikes does not exist.")
        })
    })
})