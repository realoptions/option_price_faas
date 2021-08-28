ARG MAJOR_VERSION
ARG BINARY

FROM rust:1.54-buster AS build
RUN apt-get update
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --target=x86_64-unknown-linux-musl --path .

FROM scratch 
ARG MAJOR_VERSION
ARG BINARY
# Service must listen to $PORT environment variable.
# This default value facilitates local development.
# see https://rocket.rs/master/guide/configuration/#environment-variables
ENV ROCKET_PORT 8080 
ENV ROCKET_ADDRESS "0.0.0.0"
ENV MAJOR_VERSION=$MAJOR_VERSION
COPY --from=build --chown=1001:1001 /usr/src/target/x86_64-unknown-linux-musl/release/$BINARY ./optionprice
# RUN chmod +x optionprice
USER 1001
# Run the web service on container startup.
CMD ["./optionprice"]