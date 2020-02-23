FROM rustlang/rust:nightly-slim

# Copy local code to the container image.
WORKDIR /usr/src/app
COPY src src
COPY benches benches
COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

# Install production dependencies and build a release artifact.
RUN cargo build --release

# Service must listen to $PORT environment variable.
# This default value facilitates local development.
ENV PORT 8080

# Run the web service on container startup.
CMD ["./target/release/option_price"]