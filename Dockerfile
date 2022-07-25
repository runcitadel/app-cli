FROM rust:1.62.1-slim-bullseye as build-env
WORKDIR /app
COPY . /app
RUN cargo build --bin app-cli --release --features=cli

FROM gcr.io/distroless/cc
COPY --from=build-env /app/target/release/app-cli /

CMD ["/app-cli"]
