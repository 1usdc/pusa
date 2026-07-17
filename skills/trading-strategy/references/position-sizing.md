# Position Sizing

Position sizing converts a risk budget into an order size. Always calculate size from maximum acceptable loss, not from desired profit.

## Core Formula

For linear USDT-settled instruments:

```text
risk_amount = account_equity * risk_percent
stop_distance = abs(entry_price - stop_price)
base_quantity = risk_amount / stop_distance
notional_value = base_quantity * entry_price
initial_margin = notional_value / leverage
```

For contract instruments, convert base quantity to contracts using the exchange-reported contract face value before placing an order.

```text
contracts = base_quantity / contract_face_value
```

Always round down to the exchange minimum step size.

## Required Clarifications

Ask a clarification question when the user provides:

- A bare number for derivatives size, such as `10` or `500`.
- A USDT amount without saying whether it means notional value or margin cost.
- Leverage without maximum loss.
- Percentage without defining whether it is equity risk, margin allocation, or notional exposure.

## Margin vs Notional

Do not treat these as equivalent:

- Notional: total position value.
- Margin: collateral committed to the position.
- Risk amount: maximum planned loss if the stop is hit.

Example:

```text
equity = 10,000 USDT
risk_percent = 0.5%
risk_amount = 50 USDT
entry = 50,000
stop = 49,500
stop_distance = 500
base_quantity = 50 / 500 = 0.1 BTC
notional = 0.1 * 50,000 = 5,000 USDT
at 10x leverage, initial_margin = 500 USDT
```

The trade risks 50 USDT, even though it uses roughly 500 USDT margin and 5,000 USDT notional.

## Sizing Checklist

- Confirm account equity or use the user's specified capital base.
- Confirm entry and stop price.
- Confirm risk percent or absolute risk amount.
- Confirm contract face value for futures, swaps, and options.
- Confirm leverage and liquidation buffer.
- Round size down and show the resulting approximate maximum loss.
