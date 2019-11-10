# Change Log

## Unreleased

### Added
* Added support for converting to and from `uuid` crate v0.8

## v0.5.0-alpha.1 - 2019-10-14

### Changed

* The library now uses `std::futures::Future` and async/await syntax.
* Most methods now take `&self` rather than `&mut self`.
* The transaction API has changed to more closely resemble the synchronous API and is significantly more ergonomic.
* Methods now take `&[&(dyn ToSql + Sync)]` rather than `&[&dyn ToSql]` to allow futures to be `Send`.
* Methods are now "normal" async functions that no longer do work up-front.
* Statements are no longer required to be prepared explicitly before use. Methods taking `&Statement` can now also take
    `&str`, and will internally prepare the statement.
* `ToSql` now serializes its value into a `BytesMut` rather than `Vec<u8>`.
* Methods that previously returned `Stream`s now return `Vec<T>`. New `*_raw` methods still provide a `Stream`
    interface.

### Added

* Added the `channel_binding=disable/allow/require` configuration to control use of channel binding.
* Added the `Client::query_one` method to cover the common case of a query that returns exactly one row.

## v0.4.0-rc.3 - 2019-06-29

### Fixed

* Significantly improved the performance of `query` and `copy_in`.

### Changed

* The items of the stream passed to `copy_in` must be `'static`.

## v0.4.0-rc.2 - 2019-03-05

### Fixed

* Fixed Cargo features to actually enable the functionality they claim to.

## v0.4.0-rc.1 - 2019-03-05

### Changed

* The client API has been significantly overhauled. It now resembles `hyper`'s, with separate `Connection` and `Client`
    objects. See the crate-level documentation for more details.
* The TLS connection mode (e.g. `prefer`) is now part of the connection configuration rather than being passed in
    separately.
* The Cargo features enabling `ToSql` and `FromSql` implementations for external crates are now versioned. For example,
    `with-uuid` is now `with-uuid-0_7`. This enables us to add support for new major versions of the crates in parallel
    without breaking backwards compatibility.
* Upgraded from `tokio-core` to `tokio`.

### Added

* Connection string configuration now more fully mirrors libpq's syntax, and supports both URL-style and key-value style
    strings.
* `FromSql` implementations can now borrow from the data buffer. In particular, this means that you can deserialize
    values as `&str`. The `FromSqlOwned` trait can be used as a bound to restrict code to deserializing owned values.
* Added support for channel binding with SCRAM authentication.
* Added multi-host support in connection configuration.
* The client now supports query pipelining, which can be used as a latency hiding measure.
* While the crate uses `tokio` by default, the base API can be used with any asynchronous stream type on any reactor.
* Added support for simple query requests returning row data.

### Removed

* The `with-openssl` feature has been removed. Use the `tokio-postgres-openssl` crate instead.
* The `with-rustc_serialize` and `with-time` features have been removed. Use `serde` and `SystemTime` or `chrono`
    instead.

## Older

Look at the [release tags] for information about older releases.

[release tags]: https://github.com/sfackler/rust-postgres/releases
