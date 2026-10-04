# Project conventions

- Keep code simple, concise, readable, and consistent with existing patterns.
- All code, comments, documentation, UI text, labels, and errors must be in English.
- Inspect nearby code and existing components, helpers, and hooks before adding new ones.
- Prefer the smallest change that fully solves the current requirement. Remove superseded code.

# Structure and reuse

- Keep routes, layouts, pages, shared UI, and server code separate.
- `src/components` contains the official Dioxus Components. Keep their supplied implementations and styles unchanged.
- `src/ui` contains application compositions of those components. Pages coordinate their data and user actions.
- Extract a component, hook, or helper when a real repeated behavior or distinct responsibility appears. Use a precise name and a small interface.
- Keep short, straightforward RSX fragments inline. Do not create generic form frameworks, repository abstractions, or service layers without a concrete need.
- Add a feature module when related operations actually share domain models and rules. Do not reorganize the whole app in anticipation of future features.
- Use an ecosystem library when it replaces a technical responsibility such as cookie parsing or email validation. Avoid competing libraries for the same task and unnecessary dependencies.

# Dioxus 0.7

- Use [Dioxus 0.7 documentation](https://dioxuslabs.com/learn/0.7) and APIs supported by the installed version. Never use `cx`, `Scope`, or `use_state`.
- Components use `#[component]` with owned, `Clone + PartialEq` props. Use reactive props when their values need to change.
- Keep hooks in a stable order. Do not call them in branches or loops, or after conditional early returns. Use a child component when it requires data that may be absent.
- Keep state local unless multiple parts of the app actually share it. Access the named session context through `use_session`.
- Derive values directly from state. Use `use_memo` for useful reactive computations, not as a default wrapper around simple expressions.
- Use `use_action` for async user actions. Read its pending state and result instead of maintaining duplicate busy, error, and success signals.
- Use `use_server_future` for initial server-rendered data that must hydrate consistently. Use `use_resource` for reactive client-side loading when appropriate.
- Use effects for side effects such as navigation and browser-only APIs after hydration. Do not use them to copy derived state between signals.
- Never hold a signal read or write guard across an `await`. Snapshot inputs before starting async work.
- Keep server and initial client rendering identical. Use suspense and error boundaries for loading and failed initial requests; do not convert request failures into anonymous sessions.
- Prefer direct RSX loops and conditionals over iterator-built markup.

# Design system

- Always use the [official Dioxus Components](https://dioxuslabs.com/components), preserving their default styles and supplied theme.
- Never customize fonts, borders, colors, backgrounds, shadows, theme tokens, or component appearance.
- Keep `assets/dioxus-base.css`, copied from the official gallery, unchanged. Load it alongside the component theme.
- Application CSS may only arrange layout: position, dimensions, spacing, flex/grid, and overflow.
- Form buttons keep their intrinsic width. Group related actions in a flex row instead of stretching buttons across the form.
- Use the official dark theme on every page, including Login and Register, with `data-theme="dark"` on the HTML template.
- Home is the entry page. The sidebar is on the left, with Home as its only navigation item and an avatar/name profile link at the bottom.
- Login and Register use a separate authentication layout. Navigation actions outside the sidebar use the official Button Link variant.

# API and type safety

- Use Dioxus 0.7 `#[get]` / `#[post]` server functions. Database and authentication dependencies belong only to the server feature.
- Use explicit structs and enums for meaningful domain concepts and shared API data. Do not add wrappers around every primitive without a concrete invariant or misuse to prevent.
- Avoid untyped JSON, unnecessary casts, and `unwrap` / `expect` for normal runtime states.
- Validate untrusted inputs at server boundaries. Use database constraints for persistent invariants; avoid redundant existence or uniqueness queries.
- Authenticate protected endpoints on the server. UI route guards are only for navigation.
- Keep password hashing off async request threads. Preserve transactions, session checks, and request-origin protection when refactoring.
- Handle unauthorized responses from protected actions through the session helper. Keep technical error details in server logs and show controlled messages in the UI.
- All application SQL uses compile-time checked SQLx `query!`, `query_as!`, `query_scalar!`, or their file variants. Never use unchecked macros or runtime-only query APIs, or bypass type/nullability checks.
- SQLx checks queries directly against running PostgreSQL during server compilation using `DATABASE_URL` from `.env`. Do not use offline metadata or a `.sqlx` cache.

# Development database

- The app is not in production. There is no real or historical data to preserve.
- Never keep legacy code, compatibility shims, old schema support, or fallback implementations.
- Never create migrations or use migration runners, including SQLx migrations.
- `infra/schema.sql` is the single source of truth for the entire current schema. Edit it directly and use `make reset` when the database must be rebuilt, discarding development data.
- Keep the Makefile at the repository root. Infrastructure commands remain independent of the app: `make up` starts only Docker Compose services.

# Authentication and files

- Resend uses the existing `RESEND` environment variable. Never print secrets or raw authentication tokens in logs. Set `RESEND_FROM` to an authorized sender; the development sender is limited to the Resend account's own address.
- `make dev` fixes the Dioxus port with `APP_PORT` (8080 by default) and derives `APP_URL` for email links from the same port. Link construction uses configured `APP_URL`, never request headers.
- Email verification does not block Home or Profile. Email changes remain pending until confirmed; require the current password for password and email changes.
- Authentication tokens are hashed, expire, and are consumed once in a transaction. Serialize account changes by locking the user first; revalidate protected sessions after acquiring the lock.
- Files use the internal `server::files::FileStore` API, a private RustFS bucket, and typed metadata. Never expose storage credentials or raw object keys to the UI.
- Future domain tables reference `files(id)` with actual foreign keys and `ON DELETE RESTRICT`. Domain modules authorize access; do not add polymorphic ownership fields or assume all files belong to a user.
- Files are immutable and limited to 25 MiB. Keep uploads inaccessible until complete, stream reads, and retain durable cleanup intents for interrupted uploads and deletions.
- `make reset` discards both PostgreSQL and RustFS volumes. Restart the application afterwards so its storage bucket is initialized again. No migrations, storage compatibility layers, or generic job framework.

# Quality checks

- Use `make fmt` for formatting and `make lint` for Clippy on the server and WebAssembly client. PostgreSQL must be running for server-side SQLx checks.
- Resolve warnings in application code. Do not modify official components or globally suppress warnings just to make lint output clean.
- Do not add or run tests, separate builds, or manual verification campaigns unless the user requests them.
- When reporting work, state what changed, which checks ran, and any remaining limitations. Do not claim linting proves runtime behavior.
