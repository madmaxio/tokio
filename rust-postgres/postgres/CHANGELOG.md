# Change Log

## Unreleased

### Added
* Added support for converting to and from `uuid` crate v0.8

## v0.17.0-alpha.1 - 2019-10-14

### Changed

* Updated `tokio-postgres` to 0.5.0-alpha.1.

## v0.16.0-rc.2 - 2019-06-29

### Fixed

* Documentation fixes

## v0.16.0-rc.1 - 2019-04-06

### Changed

* `Connection` has been renamed to `Client`.
* The `Client` type is now a thin wrapper around the tokio-postgres nonblocking client. By default, this is handled
    transparently by spawning connections onto an internal tokio `Runtime`, but this can also be controlled explicitly.
* The `ConnectParams` type and `IntoConnectParams` trait have been replaced by a builder-style `Config` type.

    Before:
    ```rust
    let params = ConnectParams::builder()
        .user("postgres", None)
        .build(Host::Tcp("localhost".to_string()))
        .build();
    let conn = Connection::connect(params, &TlsMode::None)?;
    ```
    After:
    ```rust
    let client = Client::configure()
        .user("postgres")
        .host("localhost")
        .connect(NoTls)?;
    ```
* The TLS connection mode (e.g. `prefer`) is now part of the connection configuration instead of being passed in
    separately.

    Before:
    ```rust
    let conn = Connection::connect("postgres://postgres@localhost", &TlsMode::Prefer(connector))?;
    ```
    After:
    ```rust
    let client = Client::connect("postgres://postgres@localhost?sslmode=prefer", connector)?;
    ```
* `Client` and `Transaction` methods take `&mut self` rather than `&self`, and correct use of the active transaction is
    verified at compile time rather than runtime.
* `Row` no longer borrows any data.
* `Statement` is now a "token" which is passed into methods on `Client` and `Transaction` and does not borrow the
    client:

    Before:
    ```rust
    let statement = conn.prepare("SELECT * FROM foo WHERE bar = $1")?;
    let rows = statement.query(&[&1i32])?;
    ```
    After:
    ```rust
    let statement = client.prepare("SELECT * FROM foo WHERE bar = $1")?;
    let rows = client.query(&statement, &[1i32])?;
    ```
* `Statement::lazy_query` has been replaced with `Transaction::bind`, which returns a `Portal` type that can be used
    with `Transaction::query_portal`.
* `Statement::copy_in` and `Statement::copy_out` have been moved to `Client` and `Transaction`.
* `Client::copy_out` and `Transaction::copy_out` now return a `Read`er rather than consuming in a `Write`r.
* `Connection::batch_execute` and `Transaction::batch_execute` have been replaced with `Client::simple_query` and
    `Transaction::simple_query`.
* The Cargo features enabling `ToSql` and `FromSql` implementations for external crates are now versioned. For example,
    `with-uuid` is now `with-uuid-0_7`. This enables us to add support for new major versions of the crates in parallel
    without breaking backwards compatibility.

### Added

* Connection string configuration now more fully mirrors libpq's syntax, and supports both URL-style and key-value style
    strings.
* `FromSql` implementations can now borrow from the data buffer. In particular, this means that you can deserialize
    values as `&str`. The `FromSqlOwned` trait can be used as a bound to restrict code to deserializing owned values.
* Added support for channel binding with SCRAM authentication.
* Added multi-host support in connection configuration.
* Added support for simple query requests returning row data.
* Added variants of query methods which return fallible iterators of values and avoid fully buffering the response in
    memory.

### Removed

* The `with-openssl` and `with-native-tls` Cargo features have been removed. Use the `tokio-postgres-openssl` and
    `tokio-postgres-native-tls` crates instead.
* The `with-rustc_serialize` and `with-time` Cargo features have been removed. Use `serde` and `SystemTime` or `chrono`
    instead.
* The `Transaction::set_commit` and `Transaction::set_rollback` methods have been removed. The only way to commit a
    transaction is to explicitly consume it via `Transaction::commit`.
* The `Rows` type has been removed; methods now return `Vec<Row>` instead.
* `Connection::prepare_cache` has been removed, as `Statement` is now `'static` and can be more easily cached
    externally.
* Some other slightly more obscure features have been removed in the initial release. If you depended on them, please
    file an issue and we can find the right design to add them back!

## Older

Look at the [release tags] for information about older releases.

[release tags]: https://github.com/sfackler/rust-postgres/releases
