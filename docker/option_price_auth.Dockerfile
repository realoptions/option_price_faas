FROM ekidd/rust-musl-builder:nightly-2019-02-08 AS build

RUN curl -L https://github.com/stevengj/nlopt/archive/v2.5.0.tar.gz -o nlopt.tar.gz
RUN tar -zxvf nlopt.tar.gz 
RUN rm nlopt.tar.gz
RUN mkdir install
RUN cd install
RUN cmake -DCMAKE_INSTALL_PREFIX=../install -DBUILD_SHARED_LIBS=OFF .
RUN make
RUN make install

RUN rustup target add x86_64-unknown-linux-musl
#RUN apt-get update
#RUN apt-get install -y cmake
#RUN apt-get install -y musl-tools  musl-gcc
#RUN apt-get install -y gcc-multilib
#RUN apt-get install -y build-essential
#WORKDIR /usr/src/optionprice
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo install --target x86_64-unknown-linux-musl --path .
ARG MAJOR_VERSION

FROM scratch

# Service must listen to $PORT environment variable.
# This default value facilitates local development.
ENV PORT 8080
ENV MAJOR_VERSION=$MAJOR_VERSION

COPY --from=build /usr/src/optionprice/target/x86_64-unknown-linux-musl/release/option_price_auth .
USER 1000
# Run the web service on container startup.
CMD ["/option_price_auth"]