ARG MAJOR_VERSION
ARG BINARY

FROM rust:slim AS build
RUN apt-get update
RUN apt-get install -y cmake
RUN apt-get install -y musl-tools
RUN apt-get install -y build-essential
WORKDIR /usr/src/optionprice
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-gnu --path .

ARG MAJOR_VERSION
ARG BINARY

FROM gcr.io/distroless/cc-debian10
ARG MAJOR_VERSION
ARG BINARY
# Service must listen to $PORT environment variable.
# This default value facilitates local development.
# see https://rocket.rs/master/guide/configuration/#environment-variables
ENV ROCKET_PORT 8080 
ENV ROCKET_ADDRESS "0.0.0.0"
ENV MAJOR_VERSION=$MAJOR_VERSION
COPY --from=build --chown=1001:1001 /usr/src/optionprice/target/x86_64-unknown-linux-gnu/release/$BINARY ./optionprice
USER 1001
# Run the web service on container startup.
CMD ["./optionprice"]