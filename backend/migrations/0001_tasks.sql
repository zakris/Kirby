CREATE TABLE tasks (
    id         UUID PRIMARY KEY,
    title      TEXT NOT NULL,
    date       DATE NOT NULL,
    done       BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX tasks_date_idx ON tasks (date);
