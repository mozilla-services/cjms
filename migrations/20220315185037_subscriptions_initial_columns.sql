CREATE TABLE subscriptions (
id uuid NOT NULL UNIQUE,
PRIMARY KEY (id),
flow_id TEXT NOT NULL UNIQUE,
subscription_id TEXT NOT NULL UNIQUE,
report_timestamp TIMESTAMPTZ NOT NULL,
subscription_created TIMESTAMPTZ NOT NULL,
fxa_uid TEXT NOT NULL,
quantity INT NOT NULL,
plan_id TEXT NOT NULL,
plan_currency TEXT NOT NULL,
plan_amount INT NOT NULL,
country TEXT,
aic_id uuid,
cj_event_value TEXT,
status TEXT,
status_history json
);
