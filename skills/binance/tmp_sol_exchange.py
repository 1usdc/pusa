import json
import urllib.request

url = 'https://fapi.binance.com/fapi/v1/exchangeInfo?symbol=SOLUSDT'
with urllib.request.urlopen(url, timeout=10) as r:
    data = json.loads(r.read().decode())
for s in data.get('symbols', []):
    if s.get('symbol') == 'SOLUSDT':
        print(json.dumps(s, indent=2))
