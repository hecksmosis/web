CREATE TABLE IF NOT EXISTS users (
    id integer PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username text NOT NULL UNIQUE,
    profile text,
    permission_level integer NOT NULL DEFAULT 0,
    password text NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    session_token BYTEA PRIMARY KEY,
    user_id integer REFERENCES users (id) ON DELETE CASCADE
);