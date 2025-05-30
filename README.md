# curto

[![Docker Image Size](https://img.shields.io/docker/image-size/rolvapneseth/curto?label=Docker%20image)](https://hub.docker.com/r/rolvapneseth/curto)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/Rolv-Apneseth/curto/docker.yml)](https://github.com/Rolv-Apneseth/world-wonders-api/actions/workflows/docker.yml)
[![License](https://img.shields.io/badge/License-AGPLv3-green.svg)](./LICENSE)

Open-source, self-hostable and fast URL-shortening API, written in Rust. Created mostly as a learning project.

## Features

- Short link creation and redirection.
- Custom shortened link IDs (optional).
- Shortened link expiration (optional).
- Only track the number of times shortened links are used, not information about users.
- Protections against attackers, such as rate-limiting (optional), request body limits and request timeouts.
- Auto-generated [OpenAPI](https://swagger.io/specification/) specification for all endpoints.
- [Scalar](https://scalar.com/) used to display the API's documentation and easily interact with it.
- Prometheus metrics for the API.
- Support for configuration via a `.env` file, or environment variables.
- [Docker image](https://hub.docker.com/r/rolvapneseth/curto) and [docker-compose.yml](./docker-compose.yml) for convenient self-hosting.

## Deployment

You can self-host using Docker, ideally with the provided [docker-compose.yml](./docker-compose.yml).

Simply copy that file to your system, and in the same directory run `docker compose up`. The documentation
should be available shortly at <http://0.0.0.0:7229/docs>.

### Build from source

While deploying with Docker is the preferred method of deployment, you can also build the project from source and
run it directly by following these steps:

1. Ensure there is a PostgreSQL database available to the application
2. Manually set environment variables or create a file called `.env` based on the provided [.env.example](./.env.example)
3. Build and run the application directly: `cargo run --release`

## Technologies

- [tokio](https://github.com/tokio-rs/tokio): Async runtime
- [axum](https://github.com/tokio-rs/axum): web app framework
- [utoipa](https://github.com/juhaku/utoipa): automatic documentation generation
- [sqlx](https://github.com/launchbadge/sqlx): interaction with the database, compile-time checked queries
- [PostgreSQL](https://www.postgresql.org/): relational database
- [Scalar](https://www.postgresql.org/): API documentation page

## Credits

- The authors and maintainers of all the technologies mentioned above, and more!
- The original idea to do this came from [this repo](https://github.com/oliverjumpertz/link-shortener/tree/main)

## License

[AGPL-3.0](./LICENSE)
