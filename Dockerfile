FROM rust as build-env
COPY token-metadata /token-metadata
COPY demo-api /app
WORKDIR /app

RUN cargo build --release

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/bpl-demo-api /
EXPOSE 8080
CMD ["./bpl-demo-api"]
