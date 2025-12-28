from fastapi import APIRouter, Depends
from pydantic import BaseModel
import asyncpg
from alert_engine import policy_engine
from typing import List

router = APIRouter()
DB_DSN = "postgres://localhost:5432/pcopp"

class AlertStatus(BaseModel):
    node_id: str
    risk_type: str
    risk_score: int
    recommended_action: str

@router.get("/alerts/active", response_model=List[AlertStatus])
async def get_active_alerts():
    """
    Fetch raw risks from SQL, run them through WASM policy, return Actionable Alerts.
    """
    conn = await asyncpg.connect(DB_DSN)
    results = []
    
    try:
        # 1. Fetch Candidates from the Brain (SQL Views)
        # Combine fragmentation and IO stall views
        rows = await conn.fetch("""
            SELECT node_id, risk_type, 85 as score, true as is_db FROM view_risk_fragmentation
            UNION ALL
            SELECT node_id, risk_type, 95 as score, true as is_db FROM view_risk_stalled_io
        """)
        
        # 2. Apply Policy (WASM)
        for row in rows:
            action = policy_engine.evaluate(
                risk_score=row['score'], 
                is_database=row['is_db']
            )
            
            if action != "IGNORE":
                results.append({
                    "node_id": str(row['node_id']),
                    "risk_type": row['risk_type'],
                    "risk_score": row['score'],
                    "recommended_action": action
                })
                
    finally:
        await conn.close()
        
    return results