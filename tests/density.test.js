'use strict'
const { location, timeout } = require('./binaryLocation.json')
const fetch = require('node-fetch')
const { spawn } = require('child_process')
jest.setTimeout(timeout)
let server
beforeAll(() => {
    server = spawn(location, [], { env: { PORT: '8080' } })
})
afterAll(() => {
    server.kill()
});
describe('density', () => {
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
            'http://localhost:8080/v2/heston/density',
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return Promise.all([
                expect(Array.isArray(response)),
                expect(response[0].value).toBeDefined(),
                expect(response[0].at_point).toBeDefined(),
            ])
        })
    })
    it('returns error if not all parameters included', () => {
        const body = {
            num_u: 8,
            maturity: 0.5,
            cf_parameters: { sigma: 0.5, speed: 0.1, v0: 0.2, eta_v: 0.1, rho: -0.5 }
        }
        return fetch(
            'http://localhost:8080/v2/heston/density',
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.err).toEqual("parse error missing field `rate` at line 1 column 100, received {\"num_u\":8,\"maturity\":0.5,\"cf_parameters\":{\"sigma\":0.5,\"speed\":0.1,\"v0\":0.2,\"eta_v\":0.1,\"rho\":-0.5}}")
        })

    })
})