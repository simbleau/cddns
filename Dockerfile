FROM rustlang/rust:nightly-bullseye AS build
# Build binary
COPY . /build/
WORKDIR /build
RUN cargo build --release

FROM debian:bullseye-slim AS app
# Copy build
COPY --from=build /build/target/release/cddns /cddns

# Need certificates for secure requests
RUN apt update -y
RUN apt install ca-certificates

# Run
WORKDIR /
ENTRYPOINT ["/cddns"]
CMD ["inventory", "watch"]