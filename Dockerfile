FROM rust
RUN cargo install cddns
ENTRYPOINT ["cddns inventory watch"]