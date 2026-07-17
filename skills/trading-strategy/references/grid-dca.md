# Grid and DCA

Grid and DCA strategies distribute entries across price levels or time. They are useful for structured accumulation or range trading, but become dangerous when they average down without a maximum loss.

## Grid Strategy

A grid places buy and sell orders around a reference range.

Required fields:

- Market and trading mode.
- Range high and range low.
- Number of grid levels.
- Capital allocated to the grid.
- Maximum position size.
- Stop condition when price leaves the range.
- Profit-taking rule and fee assumption.

Do not run a grid when:

- The range is not defined.
- The user expects the grid to survive any price move.
- There is no stop or deactivation condition.
- The asset has poor liquidity or unstable spreads.

## DCA Strategy

DCA accumulates or reduces exposure over time or price intervals.

Required fields:

- Total capital budget.
- Per-order amount.
- Schedule or price interval.
- Maximum number of orders.
- Stop or pause condition.
- Exit or rebalance rule.

Do not run DCA when:

- The capital budget is unlimited.
- The strategy doubles size after every loss without a hard cap.
- The user cannot tolerate the expected drawdown.
- The asset is a high-risk token without liquidity or fundamental support.

## Martingale Warning

Martingale-like sizing increases position after losses. Treat it as high risk. Before allowing any martingale variant, require:

- Maximum number of adds.
- Maximum total notional.
- Maximum account loss.
- Liquidation buffer.
- A clear condition that stops the sequence.

If any item is missing, the decision is `do not execute` or `needs clarification`.

## Minimal Grid Template

```text
Market: <exchange symbol>
Range: <low> to <high>
Grid levels: <n>
Capital budget: <amount>
Per-level size: <amount>
Stop/deactivate: <condition>
Take profit: <per-grid target after fees>
Max loss: <absolute or percent>
```

## Minimal DCA Template

```text
Market: <exchange symbol>
Direction: accumulate or reduce
Schedule: <time-based or price-based>
Per-order amount: <amount>
Max orders: <n>
Total budget: <amount>
Pause/stop: <condition>
Exit/rebalance: <condition>
```
