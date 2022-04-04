FROM rust:1.57 as build
WORKDIR /app
COPY . /app
ARG CI
ARG GITHUB_SHA
ARG GITHUB_REF_NAME
RUN cargo build --release
RUN ./target/release/make_version_file
RUN cat version.yaml

# Note: If you need to debug this image add ":debug" to the end of the next line
# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debug-images
FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/web /
COPY --from=build /app/target/release/batch_refunds /
COPY --from=build /app/target/release/check_subscriptions /
COPY --from=build /app/target/release/check_refunds /
COPY --from=build /app/target/release/report_subscriptions /
COPY --from=build /app/version.yaml /
CMD ["./web"]
