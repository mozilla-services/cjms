-- Add up migration script here
	CREATE TABLE aic (
	id uuid NOT NULL,
	PRIMARY KEY (id),
	cj_event_value TEXT NOT NULL,
	flow_id TEXT NOT NULL,
	created timestamptz NOT NULL
	expires timestamptz NOT NULL
);
