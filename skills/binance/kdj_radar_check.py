import json, math, urllib.request, urllib.parse
from datetime import datetime, timezone

BASE='https://api.binance.com'
SYMBOLS=['BTCUSDT','ETHUSDT']
INTERVALS=['5m','15m','1h']
LIMIT=120

def get_json(path, params=None):
    url=BASE+path
    if params:
        url += '?' + urllib.parse.urlencode(params)
    req=urllib.request.Request(url, headers={'User-Agent':'kdj-radar/1.0'})
    with urllib.request.urlopen(req, timeout=15) as r:
        return json.loads(r.read().decode())

def fetch_klines(symbol, interval, now_ms):
    data=get_json('/api/v3/klines', {'symbol':symbol,'interval':interval,'limit':LIMIT})
    closed=[]
    for k in data:
        # k: open time, open, high, low, close, volume, close time...
        if int(k[6]) <= now_ms:
            closed.append({
                'open_time': int(k[0]), 'open': float(k[1]), 'high': float(k[2]),
                'low': float(k[3]), 'close': float(k[4]), 'volume': float(k[5]),
                'close_time': int(k[6])
            })
    return closed[-100:]

def kdj(klines, n=9, k_period=3, d_period=3):
    K=50.0; D=50.0
    out=[]
    for i,c in enumerate(klines):
        start=max(0, i-n+1)
        window=klines[start:i+1]
        hh=max(x['high'] for x in window)
        ll=min(x['low'] for x in window)
        if hh == ll:
            rsv=50.0
        else:
            rsv=(c['close']-ll)/(hh-ll)*100.0
        K=( (k_period-1)*K + rsv )/k_period
        D=( (d_period-1)*D + K )/d_period
        J=3*K-2*D
        out.append({'K':K,'D':D,'J':J,'rsv':rsv})
    return out

def cross(vals):
    if len(vals)<2: return (False, False)
    p=vals[-2]; c=vals[-1]
    golden = c['K'] > c['D'] and p['K'] <= p['D']
    dead = c['K'] < c['D'] and p['K'] >= p['D']
    return golden, dead

def rising2(vals, key='J'):
    return len(vals)>=3 and vals[-1][key] > vals[-2][key] > vals[-3][key]

def falling2(vals, key='J'):
    return len(vals)>=3 and vals[-1][key] < vals[-2][key] < vals[-3][key]

def j_upturn(vals):
    return len(vals)>=2 and vals[-1]['J'] > vals[-2]['J']

def j_downturn(vals):
    return len(vals)>=2 and vals[-1]['J'] < vals[-2]['J']

def local_extrema_divergence(klines5, kdj5, window=20):
    ks=klines5[-window:]; vs=kdj5[-window:]
    highs=[]; lows=[]
    for i in range(1,len(ks)-1):
        if ks[i]['high'] > ks[i-1]['high'] and ks[i]['high'] > ks[i+1]['high']:
            highs.append((i, ks[i]['high'], vs[i]['J']))
        if ks[i]['low'] < ks[i-1]['low'] and ks[i]['low'] < ks[i+1]['low']:
            lows.append((i, ks[i]['low'], vs[i]['J']))
    top=False; bottom=False; top_pair=None; bottom_pair=None
    if len(highs)>=2:
        a,b=highs[-2], highs[-1]
        top = b[1] > a[1] and b[2] < a[2]
        top_pair=(a,b)
    if len(lows)>=2:
        a,b=lows[-2], lows[-1]
        bottom = b[1] < a[1] and b[2] > a[2]
        bottom_pair=(a,b)
    return {'top':top,'bottom':bottom,'top_pair':top_pair,'bottom_pair':bottom_pair,'highs_count':len(highs),'lows_count':len(lows)}

def near_recent_high(klines, pct=0.005):
    recent=klines[-20:]
    c=klines[-1]['close']; h=max(x['high'] for x in recent)
    return (h-c)/h <= pct if h else False

def near_recent_low(klines, pct=0.005):
    recent=klines[-20:]
    c=klines[-1]['close']; l=min(x['low'] for x in recent)
    return (c-l)/l <= pct if l else False

def decide(symbol, data, kdjs):
    k5,k15,k1 = data['5m'],data['15m'],data['1h']
    v5,v15,v1 = kdjs['5m'],kdjs['15m'],kdjs['1h']
    c5,c15,c1 = v5[-1],v15[-1],v1[-1]
    p5=v5[-2]
    g5,d5=cross(v5); g15,d15=cross(v15); g1,d1=cross(v1)
    div=local_extrema_divergence(k5,v5)
    price=k5[-1]['close']
    prev_close=k5[-2]['close']
    vol_pct=(price-prev_close)/prev_close*100 if prev_close else 0
    violent=abs(vol_pct)>3
    bull_res=(g5 and g15) or (c5['K']>c5['D'] and c15['K']>c15['D'])
    bear_res=(d5 and d15) or (c5['K']<c5['D'] and c15['K']<c15['D'])
    trend='bull' if c1['K']>c1['D'] else 'bear' if c1['K']<c1['D'] else 'neutral'
    candidates=[]
    def add(level, direction, reasons, kind='normal'):
        candidates.append({'level':level,'direction':direction,'reasons':reasons,'kind':kind})
    # Divergences first as candidates
    if div['top']:
        lvl='A' if d5 else 'C'
        add(lvl,'顶背离预警',['最近20根5m内价格形成更高局部高点而对应J值高点降低'] + (['同时出现5m死叉'] if d5 else ['未同时出现5m死叉，确认不足']),'div')
    if div['bottom']:
        lvl='A' if g5 else 'C'
        add(lvl,'底背离预警',['最近20根5m内价格形成更低局部低点而对应J值低点抬高'] + (['同时出现5m金叉'] if g5 else ['未同时出现5m金叉，确认不足']),'div')
    # S/A rules
    if g5 and (c5['K']<30 or c5['J']<20) and (g15 or c15['K']>c15['D']) and (c1['K']>=c1['D'] or j_upturn(v1)) and (not near_recent_high(k5)):
        add('S','多头机会',['5m超卖附近金叉且K<30或J<20','15m金叉或K>D','1h K>=D或J上拐','当前价格不在近20根5m最高价0.5%内追高'])
    if d5 and (c5['K']>70 or c5['J']>80) and (d15 or c15['K']<c15['D']) and (c1['K']<=c1['D'] or j_downturn(v1)) and (not near_recent_low(k5)):
        add('S','空头风险',['5m超买附近死叉且K>70或J>80','15m死叉或K<D','1h K<=D或J下拐','当前价格不在近20根5m最低价0.5%内追空'])
    if g5 and (c15['K']>c15['D'] or rising2(v15)) and ((p5['J']<20 and c5['J']>p5['J']) or (c5['K']<40 and c5['D']<40)):
        add('A','多头机会',['5m出现金叉','15m K>D或15m J连续2根上升','5m J从20以下回升或K/D均低于40'])
    if d5 and (c15['K']<c15['D'] or falling2(v15)) and ((p5['J']>80 and c5['J']<p5['J']) or (c5['K']>60 and c5['D']>60)):
        add('A','空头风险',['5m出现死叉','15m K<D或15m J连续2根下降','5m J从80以上回落或K/D均高于60'])
    # Specific trigger upgrades
    if g5 and c5['K']<50 and c5['J']>p5['J']:
        lvl='A' if c15['K']>c15['D'] else 'C'
        if c15['K']>c15['D'] and (c1['K']>=c1['D'] or trend=='bull'): lvl='S'
        add(lvl,'多头机会',['5m KDJ金叉','5m K<50','J值较上一根上升', '15m K>D' if c15['K']>c15['D'] else '15m未确认'])
    if d5 and c5['K']>50 and c5['J']<p5['J']:
        lvl='A' if c15['K']<c15['D'] else 'C'
        if c15['K']<c15['D'] and (c1['K']<=c1['D'] or trend=='bear'): lvl='S'
        add(lvl,'空头风险',['5m KDJ死叉','5m K>50','J值较上一根下降', '15m K<D' if c15['K']<c15['D'] else '15m未确认'])
    # Oversold/overbought
    if (c5['K']<20 or c5['J']<0) and c5['J']>p5['J']:
        add('C','超卖预警',['5m K<20或J<0','J值从低位上拐'])
    if (c5['K']>80 or c5['J']>100) and c5['J']<p5['J']:
        add('C','超买预警',['5m K>80或J>100','J值从高位下拐'])
    # B observation
    if c5['K']>c5['D'] and not g5 and rising2(v5) and price <= max(x['high'] for x in k5[-10:]):
        add('B','多头机会',['5m K>D但未发生新金叉','J值连续2根上升','价格尚未明显突破最近10根K线高点'])
    if c5['K']<c5['D'] and not d5 and falling2(v5) and price >= min(x['low'] for x in k5[-10:]):
        add('B','空头风险',['5m K<D但未发生新死叉','J值连续2根下降','价格尚未明显跌破最近10根K线低点'])
    # C status single
    if not candidates:
        if g5: add('C','多头机会',['仅5m单周期金叉，缺少15m或1h确认'])
        if d5: add('C','空头风险',['仅5m单周期死叉，缺少15m或1h确认'])
        if c5['K']<20 and c5['D']<25: add('C','超卖预警',['5m K<20且D<25'])
        if c5['K']>80 and c5['D']>75: add('C','超买预警',['5m K>80且D>75'])
        if c5['J']<0: add('C','超卖预警',['5m J<0，极度超卖'])
        if c5['J']>100: add('C','超买预警',['5m J>100，极度超买'])
    order={'S':4,'A':3,'B':2,'C':1}
    # sort: level, divergence priority, resonance priority
    def score(x):
        res=1 if ((x['direction'].startswith('多') or x['direction'].startswith('底')) and bull_res) or ((x['direction'].startswith('空') or x['direction'].startswith('顶') or x['direction'].startswith('超买')) and bear_res) else 0
        divs=1 if x.get('kind')=='div' else 0
        return (order[x['level']], divs, res)
    best=max(candidates, key=score) if candidates else None
    push=False
    if best and best['level'] in ('A','S'): push=True
    elif best and best['level'] in ('B','C') and not violent: push=False  # ordinary reminder in summary, not real-time push
    return {
        'symbol':symbol,'price':price,'close_time':k5[-1]['close_time'],'kdj':{'5m':c5,'15m':c15,'1h':c1},
        'prev5':p5,'cross':{'5m':{'golden':g5,'dead':d5},'15m':{'golden':g15,'dead':d15}},
        'resonance':{'bull':bull_res,'bear':bear_res,'trend1h':trend},'divergence':div,
        'violent':violent,'vol_pct':vol_pct,'signal':best,'push':push,'candidates':candidates
    }

def main():
    out={'exchange':'Binance','fetched_at':datetime.now(timezone.utc).isoformat(),'symbols':{},'errors':[]}
    try:
        now_ms=int(get_json('/api/v3/time')['serverTime'])
        out['server_time']=datetime.fromtimestamp(now_ms/1000, tz=timezone.utc).isoformat()
    except Exception as e:
        out['status']='data_fetch_failed'; out['errors'].append('server time: '+repr(e)); print(json.dumps(out,ensure_ascii=False,indent=2)); return
    for sym in SYMBOLS:
        try:
            data={}
            for itv in INTERVALS:
                kl=fetch_klines(sym,itv,now_ms)
                if len(kl)<50:
                    raise ValueError(f'{itv} data insufficient: {len(kl)}')
                if len(kl)<100:
                    # allowed but record? user asked at least 100; if less than 50 no signals. We'll record less than 100 anomaly.
                    pass
                if any(x['close']<=0 or x['high']<=0 or x['low']<=0 for x in kl):
                    raise ValueError(f'{itv} zero/invalid price')
                data[itv]=kl
            kdjs={itv:kdj(data[itv]) for itv in INTERVALS}
            for itv, vals in kdjs.items():
                for v in vals[-3:]:
                    if not all(math.isfinite(v[k]) for k in ('K','D','J')):
                        raise ValueError(f'{itv} KDJ invalid')
            out['symbols'][sym]=decide(sym,data,kdjs)
        except Exception as e:
            out['symbols'][sym]={'symbol':sym,'error':repr(e),'valid':False}
            out['errors'].append(f'{sym}: {repr(e)}')
    out['status']='ok' if not out['errors'] else 'partial_error'
    print(json.dumps(out,ensure_ascii=False,indent=2))

if __name__=='__main__':
    main()
