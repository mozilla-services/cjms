FROM rust:1.57 as build
WORKDIR /app
COPY . /app
RUN ls -hal
RUN cargo build --release
RUN ls -hal
RUN if [ ! -f "version.yaml" ]; then ./target/release/make_version_file; fi
RUN cat version.yaml

# Note: If you need to debug this image add ":debug" to the end of the next line
# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debug-images
FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/web /
COPY --from=build /app/version.yaml /
CMD ["./web"]
