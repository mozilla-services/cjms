CREATE TABLE cj_events (
    id SERIAL PRIMARY KEY,  -- TODO this may be better as a UUID
    flow_id TEXT,  -- TODO confirm type, uniqueness, nullability
    cj_id TEXT  -- TODO confirm type, uniqueness, nullability
    -- TODO add created_at
    -- TODO add updated_at (it appears that there is a diesel pg function for this purpose)
);
