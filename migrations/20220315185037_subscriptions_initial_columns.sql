CREATE TABLE subscriptions (
id uuid NOT NULL UNIQUE,
PRIMARY KEY (id),
aic_id uuid NOT NULL UNIQUE,
cj_event_value TEXT NOT NULL,
flow_id TEXT NOT NULL UNIQUE,
report_timestamp TIMESTAMPTZ NOT NULL,
subscription_created TIMESTAMPTZ NOT NULL,
subscription_id TEXT NOT NULL,
fxa_uid TEXT NOT NULL,
quantity INT NOT NULL,
plan_id TEXT NOT NULL,
plan_currency TEXT NOT NULL,
plan_amount INT NOT NULL,
country TEXT NOT NULL
);
