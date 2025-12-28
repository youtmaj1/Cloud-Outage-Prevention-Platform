import json
import logging
from pathlib import Path
from fastapi import FastAPI, HTTPException, Request
from jsonschema import validate, ValidationError
import asyncpg
from datetime import datetime
from contextlib import asynccontextmanager

# Setup
app = FastAPI()
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("ingest")

# 1. Load The Law (Schema) ONCE at startup
SCHEMA_PATH = Path("../shared/telemetry.json")
with open(SCHEMA_PATH) as f:
    TELEMETRY_SCHEMA = json.load(f)

# Database Connection Pool
DB_DSN = "postgresql://localhost/pcopp"

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup: Create connection pool
    app.state.pool = await asyncpg.create_pool(DB_DSN, min_size=1, max_size=10)
    logger.info("Database connection pool created")
    yield
    # Shutdown: Close pool
    await app.state.pool.close()
    logger.info("Database connection pool closed")

app = FastAPI(lifespan=lifespan)

@app.post("/ingest")
async def ingest_telemetry(request: Request):
    """
    Accepts telemetry, validates against schema, writes to DB.
    """
    try:
        # 1. Parse JSON
        payload = await request.json()
        
        # 2. Validate against Contract (The Exam)
        # If this fails, we reject immediately. 422 Unprocessable Entity.
        validate(instance=payload, schema=TELEMETRY_SCHEMA)
        
        # 3. Extract critical fields for SQL columns
        ts = payload['meta']['timestamp_utc']
        node = payload['meta']['node_id']
        frag = payload['memory']['mem_frag_index']
        d_state = payload['scheduler']['procs_blocked']
        
        # 4. Write to Storage (The "Walking Skeleton" action)
        async with app.state.pool.acquire() as conn:
            await conn.execute('''
                INSERT INTO telemetry_raw (timestamp_utc, node_id, payload, frag_index, d_state_count)
                VALUES ($1, $2, $3, $4, $5)
            ''', datetime.fromisoformat(ts), node, json.dumps(payload), frag, d_state)
            
        logger.info(f"Accepted payload from {node} - Frag: {frag}")
        return {"status": "accepted"}

    except ValidationError as e:
        logger.error(f"Schema Violation: {e.message}")
        raise HTTPException(status_code=422, detail=f"Schema violation: {e.message}")
    except Exception as e:
        logger.error(f"System Error: {str(e)}")
        raise HTTPException(status_code=500, detail="Internal Server Error")