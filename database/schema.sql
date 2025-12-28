-- We assume a standard Postgres setup. 
-- In production, we would enable TimescaleDB extension here: CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS telemetry_raw (
    -- The partition key for time-series
    timestamp_utc TIMESTAMPTZ NOT NULL,
    
    -- Identity
    node_id UUID NOT NULL,
    
    -- The full payload (Schema V1)
    -- We store the raw JSON to handle future schema changes gracefully
    payload JSONB NOT NULL,
    
    -- Extracted signals for fast indexing/alerting (Optimization)
    -- We pull these out of JSONB for performance on common queries
    frag_index FLOAT,
    d_state_count INT,
    
    PRIMARY KEY (timestamp_utc, node_id)
);

-- Index for the "Silent Memory" outage pattern
CREATE INDEX idx_frag_trend ON telemetry_raw (node_id, timestamp_utc DESC, frag_index);

-- Index for the "D-State" outage pattern
CREATE INDEX idx_dstate_spike ON telemetry_raw (timestamp_utc DESC, d_state_count) WHERE d_state_count > 0;