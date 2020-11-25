FROM rustlang/rust:nightly-slim AS build
#RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update
RUN apt-get install -y cmake
RUN apt-get install -y musl-tools
RUN apt-get install -y build-essential
WORKDIR /usr/src/optionprice
COPY Cargo.toml Cargo.lock ./
COPY src ./src
#RUN cargo install --target x86_64-unknown-linux-musl --path .
RUN cargo install --path .
ARG MAJOR_VERSION

FROM rustlang/rust:nightly-slim

# Service must listen to $PORT environment variable.
# This default value facilitates local development.
ENV PORT 8080
ENV MAJOR_VERSION=$MAJOR_VERSION
#COPY --from=build /usr/src/optionprice/target/x86_64-unknown-linux-musl/release/option_price .
COPY --from=build /usr/src/optionprice/target/release/option_price .
USER 1000
# Run the web service on container startup.
CMD ["/option_price"]