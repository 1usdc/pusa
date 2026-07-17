# Risk Control

Risk control is the first gate for every trading strategy. A strategy is not executable until loss boundaries, invalidation rules, and market assumptions are explicit.

## Minimum Required Fields

- Trading mode: demo or live.
- Exchange and symbol: for example `OKX BTC-USDT-SWAP` or `Binance ETHUSDT`.
- Direction: long, short, market-neutral, grid, or DCA.
- Timeframe: signal timeframe and expected holding period.
- Entry condition: exact trigger, not a vague market opinion.
- Invalidation rule: the condition that proves the trade thesis wrong.
- Stop-loss rule: price-based, volatility-based, time-based, or event-based.
- Maximum loss: percent of account equity or absolute USDT.
- Position sizing rule: how size is calculated from risk.

## Hard Stop Conditions

Do not execute or recommend live execution when any of these are true:

- No stop-loss or invalidation rule is provided.
- The user provides only a direction, such as "long BTC", without size and risk limit.
- The strategy increases size after losses without a fixed maximum loss.
- The strategy requires unlimited capital, unlimited grid expansion, or unlimited averaging down.
- Leverage is requested but liquidation distance is not considered.
- The strategy depends on unavailable data or unverified market conditions.
- The symbol, contract type, or settlement currency is ambiguous.

## Risk Budget

Default conservative limits when the user has not specified stronger constraints:

- Single trade risk: 0.25% to 1.00% of account equity.
- Daily realized loss stop: 2% to 3% of account equity.
- Strategy drawdown pause: 5% to 8% from recent equity high.
- Maximum correlated exposure: treat BTC, ETH, and high-beta alt perps as correlated in stress.

These defaults are guidance only. Ask the user before applying them to live trading.

## Execution Decision Format

Summarize every strategy decision with:

- Decision: `execute`, `do not execute`, `needs backtest`, or `needs clarification`.
- Reason: one sentence focused on risk and evidence.
- Risk: maximum expected loss, stop level, and major failure mode.
- Next action: exact data check, order action, or question for the user.
