#!/bin/bash

IMAGE_NAME=us-west1-docker.pkg.dev/bokoup/demo/demo-api

docker build -t $IMAGE_NAME .
docker push $IMAGE_NAME
gcloud beta run deploy demo-api --image $IMAGE_NAME --platform managed --region us-west1 --allow-unauthenticated \
--min-instances 1 \
--update-env-vars RUST_LOG=DEBUG

# --service-account demo-bokoup@bokoup.iam.gserviceaccount.com \
# --update-secrets /service_account/service_account.json=SERVICE_ACCOUNT:1,/arweave_dev/arweave_dev.json=ARWEAVE_DEV:1,/arweave_main/arweave_main.json=ARWEAVE_MAIN:2 \
