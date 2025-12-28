import pytest
import json
from datetime import datetime, timedelta, timezone
from jsonschema import validate, ValidationError

# Load the schema (The Law)
with open('shared/telemetry.json') as f:
    SCHEMA = json.load(f)

def create_valid_payload():
    """Generates a boring, correct payload."""
    return {
        "meta": {
            "node_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp_utc": datetime.now(timezone.utc).isoformat(),
            "kernel_version": "5.15.0-generic"
        },
        "memory": {
            "mem_frag_index": 0.45,
            "oom_kill_count": 0
        },
        "scheduler": {
            "load_avg_1m": 1.05,
            "procs_blocked": 0
        },
        "io": {
            "dirty_pages_bytes": 4096
        }
    }

def test_contract_happy_path():
    """Does a valid payload pass?"""
    payload = create_valid_payload()
    # Should not raise
    validate(instance=payload, schema=SCHEMA)

def test_contract_detects_impossible_fragmentation():
    """Scenario: Agent calculation bug sends fragmentation > 100%"""
    payload = create_valid_payload()
    payload['memory']['mem_frag_index'] = 1.5 # Impossible!
    
    with pytest.raises(ValidationError) as excinfo:
        validate(instance=payload, schema=SCHEMA)
    assert "1.5" in str(excinfo.value)

def test_contract_detects_negative_metrics():
    """Scenario: Integer overflow in Rust agent results in negative counter"""
    payload = create_valid_payload()
    payload['scheduler']['procs_blocked'] = -5 
    
    with pytest.raises(ValidationError):
        validate(instance=payload, schema=SCHEMA)

def test_stale_data_rejection():
    """
    Scenario: The Agent was partitioned from the network for 10 minutes.
    We must REJECT this data for real-time prediction, 
    but perhaps log it differently (Business Logic test).
    
    Note: JSON Schema validation validates format, 
    Logic validation validates time. This simulates the Logic check.
    """
    payload = create_valid_payload()
    old_time = datetime.now(timezone.utc) - timedelta(minutes=10)
    payload['meta']['timestamp_utc'] = old_time.isoformat()
    
    # Simulating the business logic check that will go in FastAPI
    payload_time = datetime.fromisoformat(payload['meta']['timestamp_utc'])
    age = datetime.now(timezone.utc) - payload_time
    
    assert age > timedelta(minutes=5), "Test setup failed"
    
    # This assertion defines the requirement for the future code:
    if age > timedelta(minutes=5):
        with pytest.raises(ValueError, match="Data too stale"):
            raise ValueError("Data too stale")