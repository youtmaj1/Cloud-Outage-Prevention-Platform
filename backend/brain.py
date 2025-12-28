import asyncio
import logging
import asyncpg
from datetime import datetime

# Setup
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger("brain")
DB_DSN = "postgres://localhost:5432/pcopp"

async def run_inference_cycle():
    """
    Runs every 30 seconds. Checks the SQL views for risky nodes.
    """
    conn = await asyncpg.connect(DB_DSN)
    try:
        logger.info("🧠 Brain scanning for anomalies...")
        
        # 1. Check Memory Fragmentation Risk
        rows = await conn.fetch("SELECT * FROM view_risk_fragmentation")
        for row in rows:
            trigger_alert(
                node=row['node_id'], 
                risk="MEMORY_FRAG", 
                detail=f"Frag Index: {row['avg_frag']:.2f} (Rising)"
            )

        # 2. Check I/O Stall Risk
        rows = await conn.fetch("SELECT * FROM view_risk_stalled_io")
        for row in rows:
            trigger_alert(
                node=row['node_id'], 
                risk="IO_DEATH_SPIRAL", 
                detail=f"Blocked Procs: {row['max_blocks']} (Stuck > 30s)"
            )
            
    except Exception as e:
        logger.error(f"Brain Freeze: {e}")
    finally:
        await conn.close()

def trigger_alert(node, risk, detail):
    """
    Placeholder for the Alerting System (Go 6).
    """
    print(f"\n🚨 [PREDICTION] Node {node} is at risk!")
    print(f"   TYPE:   {risk}")
    print(f"   REASON: {detail}\n")

async def main():
    await run_inference_cycle()

if __name__ == "__main__":
    asyncio.run(main())