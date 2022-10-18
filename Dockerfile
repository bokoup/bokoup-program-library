FROM rust as build-env
COPY token-metadata/program /token-metadata/program
COPY api-tx /app
WORKDIR /app

RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/bpl-api-tx /
EXPOSE 8080
CMD ["./bpl-api-tx"]
