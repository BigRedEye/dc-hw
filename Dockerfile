FROM rust:1.43.1-stretch AS build
ARG CRATE
COPY . .
RUN cargo fetch
RUN cargo build --bin ${CRATE}
RUN cp /target/debug/${CRATE} /app


FROM debian:buster-slim
RUN apt-get update && apt-get -y install libpq5
RUN apt-get install -y ca-certificates
RUN update-ca-certificates
COPY --from=build /app .
CMD ["./app"]
