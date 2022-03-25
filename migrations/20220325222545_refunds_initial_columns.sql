CREATE TABLE refunds (
id uuid NOT NULL UNIQUE,
PRIMARY KEY (id),
refund_id TEXT NOT NULL UNIQUE,
subscription_id TEXT NOT NULL,
refund_created TIMESTAMPTZ NOT NULL,
refund_amount INT NOT NULL,
refund_status TEXT,
refund_reason TEXT,
status TEXT,
status_t TIMESTAMPTZ,
status_history json
);
