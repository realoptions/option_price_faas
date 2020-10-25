const yaml = require('js-yaml')
const fs = require('fs')
const SwaggerParser = require("@apidevtools/swagger-parser")
// Get document, or throw exception on error
const yamls = [
    './docs/openapi_gcp.yml',
    './docs/openapi_rapidapi.yml'
]
yamls.forEach(file => {
    const swagger = yaml.safeLoad(fs.readFileSync(file, 'utf8'))
    SwaggerParser.validate(swagger, (err, api) => {
        if (err) {
            throw Error(err)
        }
        else {
            console.log("API name: %s, Version: %s", api.info.title, api.info.version);
        }
    })
})


