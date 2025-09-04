# Creme Brulee

This branch [portfolio-blog](https://github.com/h4rldev/creme-brulee/tree/portfolio-blog) is my personal portfolio blog built with Rust, [Axum](https://github.com/tokio-rs/axum) and [SeaORM](https://github.com/SeaQL/sea-orm).

## Features

- Markdown blog with editing, and drafts
- Markdown guestbook
- Admin dashboard
- Authentication
- Database migrations
- Rudimentary statistics for blog posts to see if people read them
- Rudimentary site visit stastistics, to catch bad actors

## Installation

TBD

## Building

- Initialize an empty sqlite database
  
```sh
sqlite3 mydatabase.db "VACUUM;"
```

- Make a `.env` file, example is in `.env.example`

- Run DB migrations

```sh
cargo run -p migration -- up
```

- Build the project

```sh
cargo build --release
```

After building, the binary will be in `target/release/creme-brulee`.
Be sure that the database file is in current directory from where you run the binary.

## Running

Run the binary once to create creme-brulee.toml, which contains some configuration for the server.

```sh
./target/release/creme-brulee
```

The server will start on port 8080 by default.


## License

This project is licensed under the BSD 3-Clause License - see the [LICENSE](LICENSE) file for details.



