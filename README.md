# cjms

[![Coverage Status](https://coveralls.io/repos/github/mozilla-services/cjms/badge.svg)](https://coveralls.io/github/mozilla-services/cjms)
[![Security audit](https://github.com/mozilla-services/cjms/actions/workflows/scheduled-audit.yml/badge.svg)](https://github.com/mozilla-services/cjms/actions/workflows/scheduled-audit.yml)

Micro-service supporting VPN activities

# Development pre-requisites
#### Rust

https://www.rust-lang.org/tools/install

We use clippy and fmt which can be added with `rustup component add clippy rustfmt`.

Many optional utilities are useful when developing cjms and can be installed with `cargo install`. e.g. `cargo install cargo-edit` adds the ability to add dependencies to the project by running `cargo add <name of package>`. Consider the following additions:
* cargo-edit
* cargo-audit
* cargo-tarpaulin

I have found the VSCode extension `rust-analyzer` to give me the richest experience while developing.
#### Postgres

https://www.postgresql.org/docs/14/index.html

- Have postgres running on your machine.
- Know your database url `postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}`
- Install [sqlx-cli](https://github.com/launchbadge/sqlx/tree/master/sqlx-cli) `cargo install sqlx-cli`
- If needed create your database `sqlx database create --database-url="<database url>"`

When adding migrations, create reversible migrations using `sqlx migrate add -r <name>`.

# Run server

You can configure cjms and tests either with environment variables or a settings file.

To use a settings file, copy `settings.yaml.example` to `settings.yaml` and update with your local settings values.
Then run the server:

`cargo run`

If configuring with environment variables, all variables, listed in settings.yaml.example must be available.

# Run tests

`cargo test`

Integration tests go under tests folder, unit tests go into associated files under src.

Running the integration tests will cause lots of test databases with the prefix `<your_db_name>_test_<random id>` to be created in
your local database. If you want to remove them, you can use a bash script along the lines of:

```
for db in `psql -U cjms -c '\l' | grep cjms_test_ | cut -d '|' -f 1`; do psql -U cjms -c "drop database \"$db\" "; done
```

In this case my db_name is `cjms` and my user is `cjms` and my password is in the environment variables `export PGPASSWORD=<>`.

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
if ! cargo sqlx prepare --check -- --bin cjms; then
    exit 1
fi

exit 0
```

## Working with sqlx

### Writing new queries

It took me a long time to figure out the mechanics of working with sqlx. Here's the key points:
* We want to use the macro functions like `query_as!` so that we get compile time checks as we're developing.
* They require DATABASE_URL to be in the environment. Either exposed in the environment or in a .env file.
* This works by sqlx connecting to a live database to check the queries at compile time. So the database that's listed
in your DATABASE_URL must be available and migrated to the latest migrations as needed. This is not the same as the dynamically
generated test databases.
* However, in CI where the final compile will happen this won't be the case, so we use sqlx's offline mode. This works by running
a command that generates the necessary data into `sqlx-data.json`. This can then be used at compile time instead of the database
being available.
* The command `cargo sqlx prepare -- --bin cjms` will auto generate `sqlx-data.json` for you (make sure DATABASE_URL is available).
* But, it only runs against one target so sql queries in the integration tests do not work. So, I've established a pattern where
  all sql queries as functions in the relevant model and the integration tests call functions on the model. While this is somewhat
  polluting production code with test code it was the best tradeoff I could find.

The general steps that I took:
* Add a .env file with my database url
* Run `sqlx migrate run` and sometimes `sqlx migrate revert` (and then run) to get my database correctly migrated
* Once everythings working as expected, run `cargo sqlx prepare -- --bin cjms` to update `sqlx-data.json`. If you don't CI
  will fail at compile time so this should be fairly easy to spot.

Links to sqlx cli information including offline mode https://github.com/launchbadge/sqlx/blob/v0.5.11/sqlx-cli/README.md.

### Version compatibility

One thing that took me a while to figure out was I was using sqlx features like "time" or "uuid" but I was
getting error messages like `expected struct sqlx::types::time::OffsetDateTime, found struct time::OffsetDateTime`.
This was confusing because I was using the modules as documented and the whole point is that these types become
magically compatible.

In both cases the reason was because the versions of `time` and `uuid` that I had installed were ahead of what sqlx
currently supports. This meant that the, I think, trait implementations for them to interoperate weren't present.

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
