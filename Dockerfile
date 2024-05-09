# Cache the dependencies of the Dashboard
FROM node:18-bookworm-slim AS dashboard_deps

WORKDIR /app

# Copy the package.json and yarn.lock files to the container
COPY ./dashboard/package.json ./dashboard/yarn.lock ./

# Install dependencies
RUN yarn install --frozen-lockfile

# Now copy all the sources so we can compile
FROM node:18-bookworm-slim AS dashboard_builder
WORKDIR /app
COPY ./dashboard .
COPY --from=dashboard_deps /app/node_modules ./node_modules

# Build the webapp
RUN yarn build --mode production

FROM lukemathwalker/cargo-chef:0.1.66-rust-slim-bookworm AS chef
WORKDIR /app/server

FROM chef AS server_planner

# Copy needed directories
COPY ./server/src /app/server/src
COPY ./server/api /app/server/api
COPY ./server/Cargo.lock /app/server/Cargo.lock
COPY ./server/Cargo.toml /app/server/Cargo.toml

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS server_builder

COPY --from=server_planner /app/server/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Copy needed directories
COPY ./server/src /app/server/src
COPY ./server/api /app/server/api
COPY ./server/Cargo.lock /app/server/Cargo.lock
COPY ./server/Cargo.toml /app/server/Cargo.toml

# Build the binary
RUN cargo build --release

FROM node:20-bookworm as keiko

ARG DOJO_VERSION

RUN if [ -z "$DOJO_VERSION" ]; then echo "DOJO_VERSION argument is required" && exit 1; fi

# Install dependencies
RUN apt-get update && \
    apt-get install -y \
    jq \
    git-all \
    build-essential \
    nano \
    net-tools


RUN apt-get autoremove && apt-get clean
RUN npm i -g @import-meta-env/cli

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

# Install starkli
SHELL ["/bin/bash", "-c"]
RUN curl https://get.starkli.sh | bash
RUN source ~/.bashrc
ENV PATH="/root/.starkli/bin:${PATH}"
RUN starkliup

# TODO copy the dojo_examples, build them

WORKDIR /keiko

# Contracts
COPY ./server/contracts ./contracts

# Warm up the git cache for "sozo build"
#RUN cd contracts && sozo build

# Server
COPY --from=server_builder /app/server/target/release/keiko .
COPY ./server/static ./static

# Dashboard
COPY --from=dashboard_builder /app/dist ./static/keiko
COPY ./dashboard/.env.example ./.env.example

ENV PUBLIC_NODE_URL=http://localhost:5050
ENV PROD=true

CMD ["./keiko"]


