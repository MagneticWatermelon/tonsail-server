# tonsail-server

## Pre-requisites

You'll need to install:

- [Rust](https://www.rust-lang.org/tools/install)
- [Docker](https://docs.docker.com/get-docker/)

## Optional

- [dotenv-cli](https://github.com/entropitor/dotenv-cli) for multiple .env files

## How to run/test

### In order to get the instances up and running either run

```bash
./scripts/init.sh
```

### or run individually

Launch a MySQL database via Docker:

```bash
./scripts/run_mysql.sh
```

Launch a QuestDB database via Docker:

```bash
./scripts/run_questdb.sh
```

Launch a Redis instance via Docker:

```bash
./scripts/run_redis.sh
```

### Rename .local.env to .env

```bash
mv .local.env .env
```

### Generate Prisma Client

```bash
cargo prisma generate
```

### Push Prisma schema to MySQL instance

```bash
cargo prisma db push
```

### Test with `cargo`

```bash
APP_ENVIRONMENT=test cargo test
```

### Run with `cargo`

```bash
cargo run
```

## Note

When using multiple .env files, run Prisma related commands by providing a .env file
manually via dotenv-cli like this:

```bash
dotenv -e .local.env -- cargo prisma db push
```

Related [Prisma Docs](https://www.prisma.io/docs/guides/development-environment/environment-variables/using-multiple-env-files)
