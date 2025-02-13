-- +migrate Up

CREATE TABLE users
(
    id                BIGSERIAL PRIMARY KEY,
    username          VARCHAR(255) NOT NULL,
    email             VARCHAR(255) NOT NULL,
    gender            TEXT,
    address           TEXT,
    self_introduction TEXT,
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at        TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- +migrate Down

DROP TABLE users;