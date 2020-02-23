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
})
describe('risk_measures', () => {
    it('returns constraints for cgmy', () => {
        return fetch(
            'http://localhost:8080/v2/cgmy/parameters/parameter_ranges',
            { method: 'GET', headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.c).toBeTruthy()
        })
    })
    it('returns constraints for heston', () => {
        return fetch(
            'http://localhost:8080/v2/heston/parameters/parameter_ranges',
            { method: 'GET', headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return Promise.all([
                expect(response.v0).toBeDefined(),
                expect(response.c).toBeUndefined(),
                expect(response.mu_l).toBeUndefined(),
                expect(response.v0).toBeTruthy(),
            ])
        })

    })
    it('returns constraints for merton', () => {
        return fetch(
            'http://localhost:8080/v2/merton/parameters/parameter_ranges',
            { method: 'GET', headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.mu_l).toBeTruthy()
        })

    })
    it('returns constraints for market', () => {
        return fetch(
            'http://localhost:8080/v2/market/parameters/parameter_ranges',
            { method: 'GET', headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.asset).toBeTruthy()
        })
    })

})
