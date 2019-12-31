'use strict'
const request = require('request')
const { location, timeout } = require('./binaryLocation.json')
const command = `./${location}`
const { spawn } = require('child_process')
const param1 = require('./parameter1.json')
const param2 = require('./parameter2.json')
const error = require('./pricerError.json')
jest.setTimeout(timeout)
let server
beforeAll(() => {
    server = spawn(command, [], { env: { PORT: '8080' } })
})

afterAll(() => {
    server.kill()
})
describe('option prices', () => {
    it('returns array of value and points', done => {
        request.post({ url: 'http://localhost:8080/v1/heston/calculator/put/price', body: JSON.parse(param1.body), json: true }, (err, response) => {
            if (err) {
                throw (err)
            }
            expect(Array.isArray(response.body))
            expect(response.body[0].value).toBeDefined()
            expect(response.body[0].at_point).toBeDefined()
            done()
        })
    })
    it('returns array of value, points, and iv', done => {
        request.post({ url: 'http://localhost:8080/v1/heston/calculator/put/price?includeImpliedVolatility=true', body: JSON.parse(param2.body), json: true }, (err, response) => {
            if (err) {
                throw (err)
            }
            expect(Array.isArray(response.body))
            expect(response.body[0].value).toBeDefined()
            expect(response.body[0].at_point).toBeDefined()
            expect(response.body[0].iv).toBeDefined()
            expect(response.body[0].iv).toBeTruthy()
            done()
        })
    })
    it('returns error if not all parameters included', done => {
        request.post({ url: 'http://localhost:8080/v1/heston/calculator/put/price?includeImpliedVolatility=true', body: JSON.parse(error.body), json: true }, (err, response) => {
            if (err) {
                throw (err)
            }
            expect(response.body).toBeDefined()
            expect(response.body.err).toEqual("Parameter strikes does not exist.")
            done()
        })
    })
})

