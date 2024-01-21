# syntax=docker/dockerfile:1.6
FROM rust:1.75-bookworm as build

WORKDIR /usr/src/gitlab-components-docs
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

ENV RUST_LOG=info

COPY --from=build /usr/local/cargo/bin/gitlab-components-docs /usr/local/bin/gitlab-components-docs
WORKDIR /gitlab
ENTRYPOINT ["gitlab-components-docs"]
