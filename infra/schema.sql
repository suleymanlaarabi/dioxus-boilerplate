CREATE TABLE users (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    first_name TEXT NOT NULL CHECK (char_length(first_name) BETWEEN 1 AND 100),
    last_name TEXT NOT NULL CHECK (char_length(last_name) BETWEEN 1 AND 100),
    email TEXT NOT NULL UNIQUE CHECK (email = lower(email) AND char_length(email) <= 254),
    password_hash TEXT NOT NULL,
    email_verified_at TIMESTAMPTZ,
    pending_email TEXT CHECK (pending_email = lower(pending_email) AND char_length(pending_email) <= 254),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '7 days'
);

CREATE INDEX sessions_user_id_idx ON sessions(user_id);
CREATE INDEX sessions_expires_at_idx ON sessions(expires_at);

CREATE TABLE auth_tokens (
    token_hash TEXT PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- TokenKind: 1 = verification, 2 = password reset, 3 = email change.
    kind SMALLINT NOT NULL CHECK (kind IN (1, 2, 3)),
    email TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (user_id, kind)
);

CREATE INDEX auth_tokens_expires_at_idx ON auth_tokens(expires_at);

CREATE TABLE auth_rate_limits (
    scope TEXT NOT NULL,
    subject_hash TEXT NOT NULL,
    attempts INTEGER NOT NULL CHECK (attempts > 0),
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (scope, subject_hash)
);

CREATE INDEX auth_rate_limits_expires_at_idx ON auth_rate_limits(expires_at);

CREATE TABLE files (
    id BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    object_key TEXT NOT NULL UNIQUE,
    original_name TEXT NOT NULL CHECK (char_length(original_name) BETWEEN 1 AND 255),
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL CHECK (size_bytes BETWEEN 0 AND 26214400),
    ready BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Durable cleanup intents for incomplete uploads and deleted files.
CREATE TABLE file_cleanup (
    object_key TEXT PRIMARY KEY,
    due_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX file_cleanup_due_at_idx ON file_cleanup(due_at);
