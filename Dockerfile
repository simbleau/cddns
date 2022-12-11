FROM rustlang/rust:nightly-bullseye AS build
# Build binary
COPY . /build/
WORKDIR /build
RUN cargo build --release

FROM debian:bullseye-slim AS app
# Copy build
COPY --from=build /build/target/release/cddns /opt/bin/cddns

# Add cddns to PATH
ENV PATH="$PATH:/opt/bin"

# Need certificates for secure requests
RUN apt update -y
RUN apt install -y ca-certificates

# Run
WORKDIR /
ENTRYPOINT ["cddns"]
CMD ["inventory", "watch"]