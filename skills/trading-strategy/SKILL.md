---
name: trading-strategy
description: "Use this skill when the user asks about trading strategy design, automated strategy execution, crypto quant rules, risk control, position sizing, stop loss/take profit, trend following, mean reversion, grid/DCA, backtesting, strategy review, or building a strategy RAG knowledge base. Covers strategy playbooks, execution checklists, and risk guardrails. This skill is for strategy reasoning and documentation; use exchange-specific skills for live market data, balances, orders, and execution."
keywords: [strategy, trading, quant, risk, position-sizing, stop-loss, take-profit, trend-following, mean-reversion, grid, dca, backtest, crypto, 策略, 交易策略, 量化, 风控, 仓位, 止损, 止盈, 趋势跟随, 均值回归, 网格, 定投, 回测]
license: MIT
metadata:
  author: anotherclaw
  version: "0.1.0"
---

# Trading Strategy

This knowledge base provides reusable trading-strategy guidance for crypto agents. Use it before designing, reviewing, or executing an automated strategy.

## Scope

- Strategy design and review: translate user intent into explicit rules, invalidation criteria, and risk limits.
- Risk control: decide whether a strategy is safe enough to run before any live execution.
- Position sizing: calculate exposure from account equity, stop distance, leverage, and maximum loss.
- Strategy families: trend following, mean reversion, grid/DCA, and event-driven checks.
- Backtesting checklist: verify a strategy before moving from idea to demo or live mode.

## Required Workflow

1. Read [`references/risk-control.md`](./references/risk-control.md) before any strategy that may place, close, or resize positions.
2. Read [`references/position-sizing.md`](./references/position-sizing.md) when the user mentions size, leverage, margin, account percentage, or maximum loss.
3. Read the relevant strategy-family document before recommending rules:
   - [`references/trend-following.md`](./references/trend-following.md)
   - [`references/mean-reversion.md`](./references/mean-reversion.md)
   - [`references/grid-dca.md`](./references/grid-dca.md)
4. Read [`references/backtesting-checklist.md`](./references/backtesting-checklist.md) before claiming a strategy is ready for demo or live automation.
5. Use exchange-specific skills only for data retrieval and execution after strategy logic and risk limits are clear.

## 中文知识库

- [`references/zh-risk-first-trading.md`](./references/zh-risk-first-trading.md)：以保本为第一原则的交易决策框架。
- [`references/zh-position-and-leverage.md`](./references/zh-position-and-leverage.md)：仓位、杠杆、保证金、名义价值和最大亏损的换算。
- [`references/zh-strategy-selection.md`](./references/zh-strategy-selection.md)：按市场状态选择趋势、震荡、网格、DCA 或空仓。
- [`references/zh-automation-playbook.md`](./references/zh-automation-playbook.md)：自动化策略上线前的执行规范和暂停条件。
- [`references/zh-review-and-journal.md`](./references/zh-review-and-journal.md)：策略复盘、交易日志和优化清单。

## Execution Guardrails

- Never execute a live strategy without an explicit stop-loss or invalidation rule.
- Never infer margin-vs-notional sizing from a bare USDT amount; ask the user to clarify.
- Treat high leverage, martingale, and unrestricted grid expansion as high-risk by default.
- If data, symbol, timeframe, position size, or trading mode is missing, stop and ask for clarification.
- Always report whether the conclusion is: `execute`, `do not execute`, `needs backtest`, or `needs clarification`.
