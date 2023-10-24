FROM node:18-alpine AS node_deps

WORKDIR /app

# Copy the package.json and yarn.lock files to the container
COPY ./dashboard/package.json ./dashboard/yarn.lock ./

# Install dependencies
RUN yarn install --frozen-lockfile

# Now copy all the sources so we can compile
FROM node:18-alpine AS node_builder
WORKDIR /app
COPY ./dashboard .
COPY --from=node_deps /app/node_modules ./node_modules

# Build the webapp
RUN yarn build --mode production

FROM rust:1 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
WORKDIR /app/server

FROM chef AS planner

# Copy needed directories
COPY ./server/src /app/server/src
COPY ./server/Cargo.lock /app/server/Cargo.lock
COPY ./server/Cargo.toml /app/server/Cargo.toml

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Install DEV dependencies and others.
RUN apt-get update -y && \
    apt-get install -y net-tools build-essential python3 python3-pip valgrind

COPY --from=planner /app/server/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Copy needed directories
COPY ./server/src /app/server/src
COPY ./server/Cargo.lock /app/server/Cargo.lock
COPY ./server/Cargo.toml /app/server/Cargo.toml

# Build the binary
RUN cargo build --release

FROM debian:bookworm-slim as keiko

ARG DOJO_VERSION

RUN if [ -z "$DOJO_VERSION" ]; then echo "VERSION argument is required" && exit 1; fi

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
    jq \
    git-all \
    build-essential \
    curl
RUN apt-get autoremove && apt-get clean

# Get Rust
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

#Install Scarb
RUN curl --proto '=https' --tlsv1.2 -sSf https://docs.swmansion.com/scarb/install.sh --output install.sh
RUN chmod +x ./install.sh
RUN export PATH=$HOME/.local/bin:$PATH && ./install.sh
RUN echo 'export PATH=$HOME/.local/bin:$PATH' >> $HOME/.bashrc
ENV PATH="/root/.local/bin:${PATH}"

# Install dojo
SHELL ["/bin/bash", "-c"]
RUN curl -L https://install.dojoengine.org | bash
RUN source ~/.bashrc
ENV PATH="/root/.dojo/bin:${PATH}"
RUN dojoup -v $DOJO_VERSION

# TODO copy the dojo_examples, build them

WORKDIR /opt

# Now the actual keiko

COPY --from=builder /app/server/target/release/server .
COPY ./server/static ./static
COPY ./server/contracts ./contracts
COPY --from=node_builder /app/dist ./static/keiko

CMD ["/opt/server"]
