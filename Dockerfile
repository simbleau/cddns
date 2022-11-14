FROM rustlang/rust:nightly
RUN cargo install cddns
ENTRYPOINT ["cddns"]
CMD ["inventory", "watch"]