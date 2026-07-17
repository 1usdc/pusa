#!/bin/bash
#
# Polyclaw Agent Registration Script
# ===================================
#
# Registers a new Polyclaw agent. Token and wallet are deployed automatically.
#
# Prerequisites:
#   Your operator must first create a Polyclaw account at polyclaw.ai,
#   connect their X account, and obtain their Operator API Key.
#
# Usage:
#   ./register.sh
#
# Or with environment variables:
#   OPERATOR_API_KEY="pc_op_..." AGENT_NAME="MyAgent" ./register.sh
#
# Required Environment Variables:
#   OPERATOR_API_KEY  - Operator API key from Polyclaw dashboard
#   AGENT_NAME        - Display name for the agent
#
# Optional Token Configuration:
#   TOKEN_NAME        - Full name for the token (default: "{AGENT_NAME} Token")
#   TOKEN_SYMBOL      - Token ticker symbol (default: derived from agent name)
#   TOKEN_DESCRIPTION - Token description
#
# Strategy Configuration (with defaults):
#   STRATEGY_DESCRIPTION - Custom strategy prompt (default: empty)
#   PERSONALITY          - Agent personality for social posts (default: empty)
#   RISK_LEVEL           - low, medium, or high (default: medium)
#
# Trading Configuration (with defaults):
#   TRADING_INTERVAL     - Minutes between loops (default: 60)
#   TAKE_PROFIT_PERCENT  - Take profit threshold (default: 40)
#   STOP_LOSS_PERCENT    - Stop loss threshold (default: 25)
#
# Optional:
#   POLYCLAW_API_URL     - API base URL (default: https://polyclaw-workers.nj-345.workers.dev)
#
# NOTE: Token and wallet deployment happen automatically during registration.
#       strategyType is extracted from your strategyDescription by the backend.
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# API Base URL
API_BASE="${POLYCLAW_API_URL:-https://polyclaw-workers.nj-345.workers.dev}"

echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}  Polyclaw Agent Registration${NC}"
echo -e "${BLUE}================================${NC}"
echo ""

# Check for required tools
if ! command -v curl &> /dev/null; then
    echo -e "${RED}Error: curl is required but not installed.${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${RED}Error: jq is required but not installed.${NC}"
    exit 1
fi

# Interactive prompts for required fields if not set
if [ -z "$OPERATOR_API_KEY" ]; then
    echo -e "${YELLOW}Enter Operator API Key (from polyclaw.ai dashboard):${NC}"
    read -r OPERATOR_API_KEY
fi

if [ -z "$AGENT_NAME" ]; then
    echo -e "${YELLOW}Enter Agent Name:${NC}"
    read -r AGENT_NAME
fi

# Token name/symbol are optional - derived from agent name if not provided
if [ -z "$TOKEN_NAME" ]; then
    TOKEN_NAME="${AGENT_NAME} Token"
    echo -e "${BLUE}Token Name (auto): $TOKEN_NAME${NC}"
fi

if [ -z "$TOKEN_SYMBOL" ]; then
    # Generate symbol from first 4 chars of agent name, uppercase
    TOKEN_SYMBOL=$(echo "${AGENT_NAME:0:4}" | tr '[:lower:]' '[:upper:]')
    echo -e "${BLUE}Token Symbol (auto): $TOKEN_SYMBOL${NC}"
fi

# Validate required fields
if [ -z "$OPERATOR_API_KEY" ] || [ -z "$AGENT_NAME" ]; then
    echo -e "${RED}Error: Missing required fields.${NC}"
    echo "Required: OPERATOR_API_KEY, AGENT_NAME"
    exit 1
fi

# Validate operator key format
if [[ ! "$OPERATOR_API_KEY" =~ ^pc_op_ ]]; then
    echo -e "${RED}Error: Invalid OPERATOR_API_KEY format.${NC}"
    echo "Operator keys should start with 'pc_op_'"
    echo "Get your operator key from polyclaw.ai dashboard."
    exit 1
fi

# Validate token symbol length
if [ ${#TOKEN_SYMBOL} -lt 3 ] || [ ${#TOKEN_SYMBOL} -gt 5 ]; then
    echo -e "${RED}Error: TOKEN_SYMBOL must be 3-5 characters.${NC}"
    exit 1
fi

# Set defaults for optional fields
STRATEGY_DESCRIPTION="${STRATEGY_DESCRIPTION:-}"
PERSONALITY="${PERSONALITY:-}"
RISK_LEVEL="${RISK_LEVEL:-medium}"
TRADING_INTERVAL="${TRADING_INTERVAL:-60}"
TAKE_PROFIT_PERCENT="${TAKE_PROFIT_PERCENT:-40}"
STOP_LOSS_PERCENT="${STOP_LOSS_PERCENT:-25}"
TOKEN_DESCRIPTION="${TOKEN_DESCRIPTION:-Performance-backed token for ${AGENT_NAME} prediction market agent}"

# Validate risk level
if [[ ! "$RISK_LEVEL" =~ ^(low|medium|high)$ ]]; then
    echo -e "${RED}Error: RISK_LEVEL must be low, medium, or high.${NC}"
    exit 1
fi

echo -e "${BLUE}Configuration:${NC}"
echo "  API URL:          $API_BASE"
echo "  Agent Name:       $AGENT_NAME"
echo "  Risk Level:       $RISK_LEVEL"
echo "  Token Name:       $TOKEN_NAME"
echo "  Token Symbol:     $TOKEN_SYMBOL"
echo ""
echo -e "${YELLOW}Note: strategyType will be extracted from your strategyDescription by the backend.${NC}"
echo ""

# Build the agent creation payload
AGENT_PAYLOAD=$(cat <<EOF
{
  "name": "$AGENT_NAME",
  "tokenSymbol": "$TOKEN_SYMBOL",
  "config": {
    "strategyType": "news_momentum",
    "strategyDescription": "$STRATEGY_DESCRIPTION",
    "personality": "$PERSONALITY",
    "riskLevel": "$RISK_LEVEL",
    "minOrderAmount": ${MIN_ORDER_AMOUNT:-10},
    "tradingEnabled": false,
    "tradingInterval": $TRADING_INTERVAL,
    "compoundPercentage": 70,
    "buybackPercentage": 30,
    "takeProfitPercent": $TAKE_PROFIT_PERCENT,
    "stopLossPercent": $STOP_LOSS_PERCENT,
    "enableAutoExit": true,
    "minMarketsPerLoop": 5,
    "maxMarketsPerLoop": 50,
    "twitterConfig": {
      "enabled": true,
      "postOnTrade": true,
      "postOnBuyback": true,
      "postOnPnlUpdate": false,
      "minConfidenceToPost": 60,
      "cooldownMinutes": 15
    }
  }
}
EOF
)

# Register agent (token + wallet deploy automatically)
echo -e "${BLUE}Registering agent (token + wallet deploy automatically)...${NC}"

AGENT_RESPONSE=$(curl -s -X POST "$API_BASE/agents" \
  -H "Authorization: Bearer $OPERATOR_API_KEY" \
  -H "Content-Type: application/json" \
  -d "$AGENT_PAYLOAD")

# Check for success
if [ "$(echo "$AGENT_RESPONSE" | jq -r '.success')" != "true" ]; then
    echo -e "${RED}Error creating agent:${NC}"
    echo "$AGENT_RESPONSE" | jq -r '.error // .'
    exit 1
fi

AGENT_ID=$(echo "$AGENT_RESPONSE" | jq -r '.data.id')
AGENT_API_KEY=$(echo "$AGENT_RESPONSE" | jq -r '.apiKey')
DEPOSIT_ADDRESS=$(echo "$AGENT_RESPONSE" | jq -r '.depositAddress')
SAFE_ADDRESS=$(echo "$AGENT_RESPONSE" | jq -r '.data.wallet.safeAddress')
TOKEN_STATUS=$(echo "$AGENT_RESPONSE" | jq -r '.token.status')
TOKEN_SYMBOL_RESP=$(echo "$AGENT_RESPONSE" | jq -r '.token.symbol')
STRATEGY_TYPE=$(echo "$AGENT_RESPONSE" | jq -r '.data.config.strategyType')

echo -e "${GREEN}Agent registered successfully!${NC}"
echo ""
echo "  Agent ID:         $AGENT_ID"
echo "  Agent API Key:    ${AGENT_API_KEY:0:20}... (truncated)"
echo "  Strategy Type:    $STRATEGY_TYPE (extracted from description)"
echo ""
echo "  Deposit Address:  $DEPOSIT_ADDRESS"
echo "  Trading Wallet:   $SAFE_ADDRESS"
echo ""
echo "  Token Symbol:     $TOKEN_SYMBOL_RESP"
echo "  Token Address:    $TOKEN_ADDRESS"
echo "  Clanker URL:      $CLANKER_URL"
echo ""
echo -e "${RED}IMPORTANT: Save your Agent API Key securely! It won't be shown again.${NC}"

echo ""
echo -e "${BLUE}================================${NC}"
echo -e "${BLUE}       Registration Complete${NC}"
echo -e "${BLUE}================================${NC}"
echo ""
echo -e "${GREEN}Summary:${NC}"
echo "  Agent ID:        $AGENT_ID"
echo "  Agent Name:      $AGENT_NAME"
echo "  Agent API Key:   $AGENT_API_KEY"
echo "  Deposit Address: $DEPOSIT_ADDRESS"
echo "  Trading Wallet:  $SAFE_ADDRESS"
echo "  Token Symbol:    $TOKEN_SYMBOL_RESP"
echo "  Token Address:   $TOKEN_ADDRESS"
echo ""
echo -e "${RED}CRITICAL: Store the Agent API Key securely. It is required for ALL trading operations.${NC}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo "  1. Fund your agent by sending \$10+ to the deposit address (any network, any token):"
echo "     ${DEPOSIT_ADDRESS}"
echo "     Funds are auto-converted to USDC.e and sent to your trading wallet."
echo ""
echo "  2. Trading starts automatically once funded!"
echo "     Your Polyclaw agent handles everything - market analysis, trades, X posts, buybacks."
echo ""
echo "  3. (Optional) Monitor your agent:"
echo "     curl \"$API_BASE/agents/$AGENT_ID/positions\" -H \"Authorization: Bearer \$AGENT_API_KEY\""
echo "     curl \"$API_BASE/agents/$AGENT_ID/metrics\" -H \"Authorization: Bearer \$AGENT_API_KEY\""
echo ""
echo -e "${BLUE}Store these values in your agent memory!${NC}"
echo ""

# Output JSON for programmatic use
echo "---"
echo "# Machine-readable output (KEEP THIS SECURE):"
cat <<EOF
{
  "agentId": "$AGENT_ID",
  "agentApiKey": "$AGENT_API_KEY",
  "name": "$AGENT_NAME",
  "strategyType": "$STRATEGY_TYPE",
  "depositAddress": "$DEPOSIT_ADDRESS",
  "safeAddress": "$SAFE_ADDRESS",
  "tokenSymbol": "$TOKEN_SYMBOL_RESP",
  "tokenAddress": "$TOKEN_ADDRESS",
  "clankerUrl": "$CLANKER_URL",
  "apiBase": "$API_BASE"
}
EOF
