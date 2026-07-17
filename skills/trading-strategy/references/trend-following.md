# Trend Following

Trend-following strategies attempt to participate in directional continuation. They should avoid predicting bottoms or tops and should exit when the trend structure fails.

## Suitable Conditions

- Price is making higher highs and higher lows for long setups, or lower highs and lower lows for short setups.
- Volatility is sufficient to cover fees and slippage.
- Liquidity is deep enough for the intended size.
- The higher timeframe does not directly conflict with the signal timeframe.

## Common Signals

- Moving-average regime: price above rising MA for longs, below falling MA for shorts.
- Breakout: close above resistance or below support, with volume or volatility confirmation.
- Donchian channel: enter on N-period high or low breakout.
- Momentum confirmation: RSI, MACD, or rate-of-change supports direction without extreme overextension.

## Required Rules

- Entry: exact trigger, including timeframe and candle close requirements.
- Stop: below structure low for longs, above structure high for shorts, or ATR-based invalidation.
- Exit: trailing stop, opposite signal, time stop, or target based on risk multiple.
- Risk: position size must be calculated from stop distance.

## Failure Modes

- Choppy range markets cause repeated false breakouts.
- Late entries after vertical moves create poor risk/reward.
- Tight stops near obvious liquidity zones may be hunted before continuation.
- Overlapping correlated positions can multiply directional risk.

## Minimal Strategy Template

```text
Market: <exchange symbol>
Timeframe: <signal timeframe>
Direction: long or short
Regime filter: <higher timeframe rule>
Entry: <specific breakout or momentum condition>
Stop: <invalidation level>
Sizing: risk <x>% of equity based on stop distance
Exit: <trailing stop / target / opposite signal>
Do not trade when: <range, news, low liquidity, spread too wide>
```
