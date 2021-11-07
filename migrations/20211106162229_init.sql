CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE custom_data as (
    random integer
);

CREATE TABLE users
(
    id uuid DEFAULT uuid_generate_v1() NOT NULL CONSTRAINT users_pkey PRIMARY KEY,
    name text NOT NULL,
    birth_date date NOT NULL,
    custom_data custom_data,
    created_at timestamp with time zone default CURRENT_TIMESTAMP,
    updated_at timestamp with time zone
);

CREATE UNIQUE INDEX users_name ON users (name);
