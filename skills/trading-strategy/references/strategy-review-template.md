# Strategy Review Template

Use this template when turning a user's informal strategy idea into an agent-ready plan.

## Inputs

```text
User intent:
Exchange:
Symbol:
Instrument type:
Trading mode:
Timeframe:
Capital base:
Maximum risk:
Allowed leverage:
Data needed:
Execution permissions:
```

## Rules

```text
Regime filter:
Entry condition:
Position sizing:
Stop-loss / invalidation:
Take-profit / exit:
Pause condition:
Daily loss stop:
Maximum drawdown stop:
```

## Safety Review

Answer each item with yes or no:

- Is the trading mode explicit?
- Is the symbol and instrument type unambiguous?
- Is maximum loss defined before entry?
- Is position size derived from risk rather than desired profit?
- Is leverage compatible with stop distance and liquidation buffer?
- Are fees, funding, and slippage considered?
- Is there a rule for no-trade conditions?
- Is there a backtest or demo-forward-test plan?

## Final Decision

```text
Decision: execute / do not execute / needs backtest / needs clarification
Reason:
Risk summary:
Next action:
```
