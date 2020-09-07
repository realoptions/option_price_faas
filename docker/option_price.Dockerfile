FROM rustlang/rust:nightly-slim AS build
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /usr/src/optionprice
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .


FROM scratch

# Service must listen to $PORT environment variable.
# This default value facilitates local development.
ENV PORT 8080
COPY --from=build /usr/src/optionprice/target/x86_64-unknown-linux-musl/release/option_price .
USER 1000
# Run the web service on container startup.
CMD ["/option_price"]