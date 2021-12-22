# cjms
Micro-service supporting VPN activities

## Developer docs

### Pre-requisites

* Rust - https://www.rust-lang.org/tools/install

Coming soon:
* Postgres

### Run and test

Set environment variables needed by [./src/env.rs](./src/env.rs)

`cargo run`

Will start server running.

`cargo test --test-threads=1`

Will run the tests. Integration tests go under tests folder, unit tests go into associated files under src.
The `--test-threads=` is needed for the environment variables tests.

### Deployment

Service is deployed using docker containers. 

Install docker.

To build, for example

`docker build -t cjms:latest .`

To run, set environment variables (can be done with a file) and forward ports e.g.

`docker run -e HOST=0.0.0.0 -e PORT=8484 -p 8484:8484 cjms:latest`
