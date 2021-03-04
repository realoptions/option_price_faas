sed -i 's/${VERSION_MAJOR}/v2/' ./docs/openapi_gcp.yml
sed -i 's/${HOST}/localhost/' ./docs/openapi_gcp.yml
sed -i 's/${PROJECT_ID}/123/' ./docs/openapi_gcp.yml
sed -i 's/${VISIBLE_HOST}/localhost/' ./docs/openapi_gcp.yml
PORT=9000 MAJOR_VERSION=v2 nohup  ./target/release/option_price &
npx dredd ./docs/openapi_gcp.yml http://localhost:9000