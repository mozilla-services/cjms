FROM rust:1.57 as build
WORKDIR /app
COPY . /app
RUN cargo build --release
ENV CI=CI
ENV GITHUB_SHA=GITHUB_SHA
ENV GITHUB_REF_NAME=GITHUB_REF_NAME
RUN ./target/release/make_version_file
RUN cat version.yaml

# Note: If you need to debug this image add ":debug" to the end of the next line
# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debug-images
FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/web /
COPY --from=build /app/version.yaml /
CMD ["./web"]
