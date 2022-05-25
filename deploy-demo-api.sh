#!/bin/bash

IMAGE_NAME=us-west1-docker.pkg.dev/bokoup/demo/demo-api

docker build -t $IMAGE_NAME .
docker push $IMAGE_NAME
gcloud beta run deploy demo-api --image $IMAGE_NAME --platform managed --region us-west1 --allow-unauthenticated \
--min-instances 1 \
--update-env-vars RUST_LOG=DEBUG \
--service-account demo-bokoup@bokoup.iam.gserviceaccount.com \
--update-secrets /keys/promo_owner-keypair.json=DEMO_PROMO_OWNER:1