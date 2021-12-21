FROM rust:1.57 as build
WORKDIR /app
COPY . /app
RUN cargo build --release

# Note: If you need to debug this image add ":debug" to the end of the next line
# https://github.com/GoogleContainerTools/distroless/blob/main/README.md#debug-images
FROM gcr.io/distroless/cc
COPY --from=build /app/target/release/cjms /
CMD ["./cjms"]