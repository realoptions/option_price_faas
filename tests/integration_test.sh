sed -i 's/${VERSION_MAJOR}/v2/' ./docs/gcp_auth.yml
sed -i 's/${HOST}/localhost/' ./docs/gcp_auth.yml
sed -i 's/${PROJECT_ID}/123/' ./docs/gcp_auth.yml
sed -i 's/${VISIBLE_HOST}/localhost/' ./docs/gcp_auth.yml
npx swagger-cli bundle -r -o ./openapi_gcp.yml -t yaml ./docs/gcp_auth.yml
ROCKET_PORT=9000 ROCKET_ADDRESS=0.0.0.0 MAJOR_VERSION=v2 nohup  ./target/release/option_price &
npx dredd ./openapi_gcp.yml http://localhost:9000