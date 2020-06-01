GATEWAY_CONFIG=$(gcloud endpoints configs list --service ${GATEWAY_SERVICE} --limit 1 \
          | grep $(date +'%Y-%m-%d') | head -n1 | awk '{print $1;}')
curl --fail -o "service.json" -H "Authorization: Bearer $(gcloud auth print-access-token)" \
    "https://servicemanagement.googleapis.com/v1/services/${GATEWAY_SERVICE}/configs/${GATEWAY_CONFIG}?view=FULL" 
docker build . -f docker/gateway.Dockerfile --tag gcr.io/${PROJECT_ID}/${GATEWAY_SERVICE}_endpoints-runtime-serverless:${GITHUB_SHA}
docker push gcr.io/${PROJECT_ID}/${GATEWAY_SERVICE}_endpoints-runtime-serverless:${GITHUB_SHA}
rm server.json
gcloud run deploy ${CLOUD_RUN_SERVICE} \
          --image=gcr.io/${PROJECT_ID}/${GATEWAY_SERVICE}_endpoints-runtime-serverless:${GITHUB_SHA} \
          --set-env-vars=ESPv2_ARGS=--cors_preset=basic \
          --allow-unauthenticated \
          --platform managed \
          --project $PROJECT_ID \
          --region $RUN_REGION 