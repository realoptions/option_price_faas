'use strict'
const fetch = require('node-fetch')
const { location } = require('./binaryLocation.json')
const body = require('./example_calibration_3.json')
const { spawn } = require('child_process')
const port = '8090'
const version = 'v2'

const server = spawn(location, [], { env: { PORT: port, MAJOR_VERSION: version } })
const get_price = (port, model) => {
    return fetch(
        `http://127.0.0.1:${port}/v2/${model}/calibrator/call`,
        { method: 'POST', body: JSON.stringify(body), headers: { 'Content-Type': 'application/json' }, }
    ).then(res => res.json()).then(response => {
        console.log("This is model " + model)
        console.log(response.parameters)
        console.log(response.final_cost_value)
    })
}
setTimeout(() => {
    const models = ["heston", "merton", "cgmy"]
    Promise.all(models.map(model => get_price(port, model))).finally(() => server.kill())
}, 1000)//wait for server to launch





