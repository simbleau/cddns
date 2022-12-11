FROM rust AS build

COPY . /build/
WORKDIR /build
RUN cargo build --release

FROM debian:buster-slim AS app
COPY --from=build /build/target/release/cddns /cddns

ENTRYPOINT ["cddns"]
CMD ["inventory", "watch"]