ARG PACKAGE=helloworld-server
FROM rustlang/rust:nightly as chef
#FROM docker.io/rustlang/rust:nightly@sha256:4cd78ba00833f16b45c185811d4285200100e0bb39c475da11117b56f8adc629 as chef
RUN apt-get clean
RUN apt-get update
RUN apt-get install -y protobuf-compiler libprotobuf-dev
#ARG CHEF_TAG=0.1.62
#RUN cargo install cargo-chef --locked --version $CHEF_TAG
RUN cargo install cargo-chef
WORKDIR build

FROM chef as planner
ARG PACKAGE
COPY . .
#RUN cargo chef prepare --recipe-path recipe.json --bin translit(__PACKAGEPATH__,`/',`_')
RUN cargo chef prepare --recipe-path recipe.json --bin .  # yes, seems to need package path here

FROM chef as builder
ARG PACKAGE
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --bin $PACKAGE
COPY . .
RUN VERSION=dev cargo build --bin $PACKAGE

FROM gcr.io/distroless/cc-debian11 as runtime
ARG PACKAGE
COPY --from=builder /build/target/debug/$PACKAGE /app
CMD ["/app"]

