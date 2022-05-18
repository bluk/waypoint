# Copyright 2022 Bryant Luk
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

FROM rust:slim as build

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:buster-slim

RUN mkdir app
WORKDIR /app

COPY --from=build /app/target/release/waypoint .

ARG SOURCE_COMMIT=unknown
ENV SOURCE_COMMIT $SOURCE_COMMIT

ARG IMAGE_TAG=unknown
ENV IMAGE_TAG $IMAGE_TAG

ARG PORT=6881
ENV PORT $PORT

EXPOSE $PORT

ARG IP_ADDR="0.0.0.0"
ENV IP_ADDR $IP_ADDR

ENTRYPOINT ["/app/waypoint"]