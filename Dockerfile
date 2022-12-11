FROM rust

COPY . /tmp
WORKDIR /tmp
RUN cargo build --release
RUN cargo install --path .
WORKDIR /root
RUN rm -rf /tmp

ENTRYPOINT ["cddns"]
CMD ["inventory", "watch"]