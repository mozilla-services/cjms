# cjms

[![Coverage Status](https://coveralls.io/repos/github/mozilla-services/cjms/badge.svg)](https://coveralls.io/github/mozilla-services/cjms)
[![Security audit](https://github.com/mozilla-services/cjms/actions/workflows/scheduled-audit.yml/badge.svg)](https://github.com/mozilla-services/cjms/actions/workflows/scheduled-audit.yml)

Micro-service supporting VPN activities

# Pre-requisites

* Rust - https://www.rust-lang.org/tools/install

Coming soon:
* Postgres

### Run server


If your server is configured with environment variables, run

`cargo run`

More likely on a dev setup, copy `settings.yaml.example` to `settings.yaml` and update with your local settings values.
Then run the server passing in the settings file.

`cargo run settings.yaml`

### Run tests

`cargo test`

Will run the tests. Integration tests go under tests folder, unit tests go into associated files under src.

# Tips

## Git hooks

To save time in CI, add a pre-commit or pre-push git hook locally that runs, at least, clippy and fmt.

For example, in `.git/hooks/` copy `pre-push.sample` to `pre-push`. Update the file, so that the end of the file reads:

```
if ! cargo fmt -- --check; then
    exit 1
fi
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    exit 1
fi

exit 0
```
# Deployment

Service is deployed using docker containers.

Install docker.

To build, for example

`docker build -t cjms:latest .`

To run, set environment variables (can be done with a file) and forward ports e.g.

`docker run -e HOST=0.0.0.0 -e PORT=8484 -p 8484:8484 cjms:latest`

## Version numbers

### Pre 1.0

Version numbers will increment 0.1, 0.2 etc as pre-releases as we work towards launch.

### 1.0+

Version numbering will follow the guardian schema:
- version numbers will increase v1.1, v1.2 etc
- in emergency situations a minor release e.g. v1.2.1 will be made for a patch release
- release candidates will have a "b" suffix e.g. ahead of v1.1 release we will push
  v1.1.b1, v1.1.b2 to staging for QA to review

### 2.0+

A major version bump from 1.x to 2.x would happen in the case of a breaking change to APIs
we provide.

For CJMS, this is unlikely to happen.

## Release, branching, and merging

### Pre 1.0

All development will happen against main

### 1.0+

main should reflect the state of production code.

At the start of a new development cycle a release branch is made and development continues
against that branch. A draft PR is opened that proposes a merge of the whole release branch
into main. The supports the following workflow elements:
- The test suite reviews the complete set of changes in the release branch
- The release PR should only be merged after QA sign-off and the QA report should be linked in
  the commit
- If a patch release is needed, make a new release branch off main and when completed cherry
  pick into the active release branch

PRs should be squash merged into the release branch.

Release branch should be merged with a merge commit into main.
