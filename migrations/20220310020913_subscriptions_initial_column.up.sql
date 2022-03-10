-- Create subscriptions table
CREATE TABLE subscription (
id uuid NOT NULL,
PRIMARY KEY (id),
report_timestamp TIMESTAMPTZ NOT NULL,
subscription_start_date	TIMESTAMPTZ NOT NULL,
fxa_uid TEXT NOT NULL,
quantity INT NOT NULL,
plan_id TEXT NOT NULL,
plan_currency TEXT NOT NULL,
plan_amount INT NOT NULL,
country TEXT NOT NULL,
promotion_codes TEXT NOT NULL
);
