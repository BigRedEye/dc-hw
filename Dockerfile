FROM rust:1.41.1-stretch AS build
COPY . .
RUN cargo build --release


FROM debian:buster-slim
RUN apt-get update && apt-get -y install libpq5
COPY --from=build /target/release/online-store-skeleton .
CMD ["./online-store-skeleton"]
