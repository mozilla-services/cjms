# cjms
Micro-service supporting VPN activities

## Developer docs

### Pre-requisites

* Rust - https://www.rust-lang.org/tools/install

Coming soon:
* Postgres

### Run and test

Optionally set environment variables with tha HOST and PORT

`cargo run`

Will start server running at http://127.0.0.1:8080

`cargo test`

Will run the tests. Integration tests go under tests folder, unit tests go into associated files under src.

### Deployment

Service is deployed using docker containers. 

Install docker and look at github actions for the docker build script.