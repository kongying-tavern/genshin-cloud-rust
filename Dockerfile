# Preload dependencies,
# used to speed up repeated builds and reduce traffic consumption of libraries
FROM rust:1 as stage-deps

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-make

COPY ./Cargo.toml /home/Cargo.toml
RUN cargo new --lib --name _database /home/packages/database
COPY ./packages/database/Cargo.toml /home/packages/database/Cargo.toml
RUN cargo new --lib --name _functions /home/packages/functions
COPY ./packages/functions/Cargo.toml /home/packages/functions/Cargo.toml
RUN cargo new --name _utils /home/packages/utils
COPY ./packages/utils/Cargo.toml /home/packages/utils/Cargo.toml
RUN cargo new --name _router /home/packages/router
COPY ./packages/router/Cargo.toml /home/packages/router/Cargo.toml
RUN cargo new --name _utils /home/packages/utils
COPY ./packages/utils/Cargo.toml /home/packages/utils/Cargo.toml

WORKDIR /home
RUN cargo fetch

COPY ./packages/database /home/packages/database
COPY ./packages/functions /home/packages/functions
COPY ./packages/router /home/packages/router
COPY ./packages/utils /home/packages/utils

# Stage 1 for server build, used to compile server program
FROM stage-deps as stage-server-build1

WORKDIR /home
RUN cargo build --offline --package _router --release

# Stage 2 for server build, used to integrate the build result of client and generate the final image
FROM debian:bookworm as stage-server-build2

ENV ROOT_DIR=/home/res
WORKDIR /home
ENTRYPOINT [ "./a" ]
EXPOSE 80
