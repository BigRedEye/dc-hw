CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TYPE access_level AS ENUM ('user', 'admin');

CREATE TABLE users (
    id serial PRIMARY KEY,
    phone text NULL UNIQUE,
    email text NULL UNIQUE,
    password text NOT NULL,
    permissions access_level NOT NULL
);

CREATE TABLE sessions (
    id serial PRIMARY KEY,
    refresh_token text NOT NULL,
    access_token text NOT NULL,
    expires_at timestamp NOT NULL,
    user_id integer NOT NULL REFERENCES users(id)
);

CREATE TABLE confirmations (
    id serial PRIMARY KEY,
    token text NOT NULL,
    phone text NULL UNIQUE,
    email text NULL UNIQUE,
    user_id integer NOT NULL REFERENCES users(id)
);
