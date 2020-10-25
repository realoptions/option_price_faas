'use strict'
const fetch = require('node-fetch')
const { location } = require('./binaryLocation.json')
const body = require('./example_calibration.json')
const { spawn } = require('child_process')
let server
const port = '8090'
const version = 'v2'
beforeAll((done) => {
    server = spawn(location, [], { env: { PORT: port, MAJOR_VERSION: version } })
    setTimeout(done, 1000) //wait for server to launch
})

afterAll(() => {
    server.kill()
})

describe('option prices', () => {
    it('returns parameters for heston', () => {
        return fetch(
            `http://localhost:${port}/v2/heston/calibrator/call`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.parameters.eta_v).toBeDefined()
        })

    })
    it('returns parameters for cgmy', () => {
        return fetch(
            `http://localhost:${port}/v2/cgmy/calibrator/call`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.parameters.c).toBeDefined()
        })

    })
    it('returns parameters for merton', () => {
        return fetch(
            `http://localhost:${port}/v2/merton/calibrator/call`,
            { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
        ).then(res => res.json()).then(response => {
            return expect(response.parameters.lambda).toBeDefined()
        })

    })
})