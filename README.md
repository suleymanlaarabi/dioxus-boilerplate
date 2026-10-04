# Boilerplate

Dioxus 0.7 fullstack with PostgreSQL, SQLx, Resend authentication emails, and internal RustFS file storage.

## Development

Copy `.env.example` to `.env` and set `RESEND`. `make up` starts only PostgreSQL and RustFS. `make dev` starts Dioxus at `http://localhost:8080` and uses that address in emails. Override both the port and email links with `make dev APP_PORT=8081`.

`RESEND_FROM=onboarding@resend.dev` only sends to your Resend account's own email address. Use a sender on a verified Resend domain for other recipients. Set `APP_URL` to your public origin when running the server outside `make dev`.

`make reset` deletes all development database records and stored files, then recreates the infrastructure from `infra/schema.sql`. Restart Dioxus afterwards. There are no migrations or SQLx offline caches. PostgreSQL must be running when compiling server queries.

`make fmt` formats Rust. `make lint` runs Clippy for the server and WebAssembly client; it does not verify runtime behavior.

RustFS exposes its local S3 API at `http://localhost:9000` and console at `http://localhost:9001`. Its credentials and private bucket are configured through `RUSTFS_*` variables in `.env`.

## Authentication

Registration opens Home immediately and requests email verification. Verification and email changes require an explicit confirmation button. Reset links expire after 30 minutes; email links expire after 24 hours. Reissuing a link replaces the previous link of the same kind.

Password changes keep the current session and revoke others. Resets and confirmed email changes revoke all sessions for that account. Profile separates names, pending email changes, and password changes.

## Internal files

Backend domain modules use `services.files`. There are no file HTTP routes or UI. Modules must authorize access before calling storage operations.

```rust,ignore
use bytes::Bytes;
use crate::server::files::{FileError, NewFile};

async fn store_document(services: &crate::server::Services, content: Bytes)
    -> Result<crate::server::files::FileMetadata, FileError>
{
    services.files.create(NewFile {
        original_name: "document.txt".into(),
        content_type: mime::TEXT_PLAIN,
    }, content).await
}
```

`get(FileId)` returns ready metadata, `read(FileId)` returns metadata and an S3 byte stream, and `delete(FileId)` removes the record and schedules physical deletion. Uploads accept at most 25 MiB. Objects are private and immutable; replacing content creates a new file.

Future tables reference `files(id)` with `ON DELETE RESTRICT`. Unlink a file before deleting it. Deleting a record directly bypasses storage cleanup: always use `FileStore::delete`.

Cleanup intents persist before uploads or metadata deletion. A task retries physical deletion every minute; interrupted uploads become eligible after 15 minutes. Deletion succeeds once the database removal and cleanup intent commit, even when RustFS is temporarily unavailable. Already deleted files are safe to delete again.
