# Backtesting Checklist

Backtesting checks whether a strategy had historical edge under realistic assumptions. It does not prove future profitability.

## Data Requirements

- Symbol, exchange, and instrument type match the intended live market.
- Time range includes trending, ranging, high-volatility, and low-volatility regimes.
- Candle or tick granularity is fine enough for the strategy timeframe.
- Fees, funding, spread, and slippage assumptions are included.
- Delisted or unavailable instruments are handled explicitly when relevant.

## Bias Checks

- No look-ahead bias: signals use only data available at the decision time.
- No repainting indicators unless using their confirmed values only.
- No survivorship bias when testing baskets.
- No cherry-picking of date ranges, symbols, or parameters.
- Parameter search is separated from out-of-sample validation.

## Metrics to Report

- Net return after fees and slippage.
- Maximum drawdown.
- Win rate and average win/loss.
- Profit factor.
- Sharpe or Sortino when appropriate.
- Number of trades.
- Longest losing streak.
- Exposure time and capital utilization.
- Worst day and worst week.

## Deployment Gates

Use this conservative progression:

1. Paper review: rules are complete and risk is bounded.
2. Historical backtest: basic edge survives fees and slippage.
3. Out-of-sample test: parameters do not collapse outside the tuning period.
4. Demo forward test: strategy behaves correctly in live market data.
5. Small live allocation: start below normal risk until operational issues are known.

## Not Ready for Live If

- Backtest has too few trades to be meaningful.
- Performance depends on one or two outlier trades.
- Drawdown exceeds the user's tolerance.
- Fees or slippage turn the edge negative.
- Strategy fails in a recent market regime.
- Execution rules are ambiguous or require discretionary judgment.

## Review Output Format

```text
Readiness: needs backtest / demo only / small live / not ready
Main edge: <why the strategy should work>
Main risk: <why it may fail>
Missing data: <required inputs>
Required next test: <specific test>
```
