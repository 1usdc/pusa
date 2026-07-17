# Mean Reversion

Mean-reversion strategies assume price has moved too far from a fair-value area and may return toward it. They require stricter invalidation than trend-following because losing trades can become runaway trends.

## Suitable Conditions

- Market is range-bound or oscillating around a stable reference.
- Funding, basis, or local positioning does not indicate a strong one-way squeeze.
- Spread and fees are small relative to expected reversion.
- Liquidity is adequate near both entry and exit levels.

## Common Signals

- Bollinger Band re-entry: price closes outside a band and then closes back inside.
- Z-score: price deviation from moving average exceeds a threshold and begins to normalize.
- RSI extreme: oversold/overbought condition plus reversal confirmation.
- Support/resistance rejection: failed breakout back into a range.

## Required Rules

- Fair value: define the mean, such as VWAP, moving average, range midpoint, or basis anchor.
- Entry: require confirmation, not just "price is low" or "price is high".
- Stop: outside the range or beyond a volatility threshold.
- Exit: mean touch, partial at midpoint, or fixed risk/reward.
- Time stop: exit if reversion does not occur within the expected window.

## Failure Modes

- A real breakout can invalidate the range and continue strongly.
- Averaging down without a cap can create large drawdowns.
- Low-liquidity altcoins can stay dislocated longer than expected.
- Mean values can shift during news, liquidation cascades, or regime changes.

## Minimal Strategy Template

```text
Market: <exchange symbol>
Regime filter: only trade when market is ranging
Mean anchor: <VWAP / MA / range midpoint>
Entry: <deviation + confirmation>
Stop: <range invalidation or ATR threshold>
Exit: <mean touch / partial target / time stop>
Sizing: risk <x>% of equity; no unlimited averaging down
Do not trade when: <breakout, high-impact news, liquidity too thin>
```
