import logging

logger = logging.getLogger("alerts")

# Action Codes
ACTION_IGNORE = 0
ACTION_LOG = 1
ACTION_PAGE = 2

class AlertPolicyEngine:
    def __init__(self):
        logger.info("✅ Alert Policy loaded successfully.")

    def evaluate(self, risk_score: int, is_database: bool) -> str:
        """
        Decision logic.
        """
        if risk_score > 80:
            return "PAGERDUTY_TRIGGER"
        elif risk_score > 50 and is_database:
            return "PAGERDUTY_TRIGGER"
        elif risk_score > 20:
            return "SLACK_LOG"
        else:
            return "IGNORE"

# Singleton instance
policy_engine = AlertPolicyEngine()