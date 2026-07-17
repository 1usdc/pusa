import json, subprocess, math, time, datetime

SYMBOLS=['BTCUSDT','ETHUSDT']
INTERVALS=['5m','15m','1h']
LIMIT=120

def run_cli(symbol, interval):
    cmd=['binance-cli','spot','klines','--symbol',symbol,'--interval',interval,'--limit',str(LIMIT)]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=60)
    if p.returncode!=0:
        raise RuntimeError((p.stderr or p.stdout).strip())
    return json.loads(p.stdout)

def closed_klines(rows):
    now=int(time.time()*1000)
    return [r for r in rows if int(r[6]) <= now]

def calc_kdj(rows, n=9, kp=3, dp=3):
    k=50.0; d=50.0
    out=[]
    for i,r in enumerate(rows):
        close=float(r[4]); high=float(r[2]); low=float(r[3])
        if close<=0 or high<=0 or low<=0:
            out.append(None); continue
        start=max(0,i-n+1)
        window=rows[start:i+1]
        hh=max(float(x[2]) for x in window)
        ll=min(float(x[3]) for x in window)
        rsv=50.0 if hh==ll else (close-ll)/(hh-ll)*100.0
        k=(kp-1)/kp*k + (1/kp)*rsv
        d=(dp-1)/dp*d + (1/dp)*k
        j=3*k-2*d
        vals={'K':k,'D':d,'J':j,'close':close,'high':high,'low':low,'open':float(r[1]),'openTime':int(r[0]),'closeTime':int(r[6])}
        if any((not math.isfinite(vals[x])) for x in ['K','D','J','close']):
            out.append(None)
        else:
            out.append(vals)
    return out

def cross(cur, prev):
    golden = cur['K']>cur['D'] and prev['K']<=prev['D']
    death = cur['K']<cur['D'] and prev['K']>=prev['D']
    return golden, death

def two_up(vals, key='J'):
    return len(vals)>=3 and vals[-1][key]>vals[-2][key]>vals[-3][key]

def two_down(vals, key='J'):
    return len(vals)>=3 and vals[-1][key]<vals[-2][key]<vals[-3][key]

def upturn(vals): return len(vals)>=2 and vals[-1]['J']>vals[-2]['J']
def downturn(vals): return len(vals)>=2 and vals[-1]['J']<vals[-2]['J']

def local_extrema(vals, typ='high'):
    res=[]
    # local extrema within recent 20, using interior points and fallback sorted if fewer than 2
    for i in range(1,len(vals)-1):
        if typ=='high' and vals[i]['high']>=vals[i-1]['high'] and vals[i]['high']>=vals[i+1]['high']:
            res.append((i,vals[i]['high'],vals[i]['J']))
        if typ=='low' and vals[i]['low']<=vals[i-1]['low'] and vals[i]['low']<=vals[i+1]['low']:
            res.append((i,vals[i]['low'],vals[i]['J']))
    return res[-2:]

def divergence(vals20):
    hs=local_extrema(vals20,'high')
    ls=local_extrema(vals20,'low')
    top=False; bottom=False
    detail=[]
    if len(hs)>=2:
        a,b=hs[-2],hs[-1]
        if b[1]>a[1] and b[2]<a[2]:
            top=True; detail.append(f"顶背离: 高点 {a[1]:.2f}->{b[1]:.2f}, J {a[2]:.2f}->{b[2]:.2f}")
    if len(ls)>=2:
        a,b=ls[-2],ls[-1]
        if b[1]<a[1] and b[2]>a[2]:
            bottom=True; detail.append(f"底背离: 低点 {a[1]:.2f}->{b[1]:.2f}, J {a[2]:.2f}->{b[2]:.2f}")
    return top,bottom,detail

def fmt(v): return f"K={v['K']:.2f}, D={v['D']:.2f}, J={v['J']:.2f}"

def evaluate(symbol, data):
    exc=[]
    kdjs={}
    rows_by={}
    for itv, rows in data.items():
        rows=closed_klines(rows)
        rows_by[itv]=rows
        if len(rows)<50:
            return {'symbol':symbol,'error':f'{itv} 已收盘K线不足50根: {len(rows)}'}
        vals=calc_kdj(rows)
        vals=[v for v in vals if v is not None]
        if len(vals)<50:
            return {'symbol':symbol,'error':f'{itv} KDJ有效值不足'}
        kdjs[itv]=vals
    v5=kdjs['5m']; v15=kdjs['15m']; v1h=kdjs['1h']
    c5,p5=v5[-1],v5[-2]; c15,p15=v15[-1],v15[-2]; c1,p1=v1h[-1],v1h[-2]
    gold5,death5=cross(c5,p5); gold15,death15=cross(c15,p15); gold1,death1=cross(c1,p1)
    bull_res=(gold5 and gold15) or (c5['K']>c5['D'] and c15['K']>c15['D'])
    bear_res=(death5 and death15) or (c5['K']<c5['D'] and c15['K']<c15['D'])
    trend='偏多' if c1['K']>c1['D'] else ('偏空' if c1['K']<c1['D'] else '中性')
    highs20=max(x['high'] for x in v5[-20:]); lows20=min(x['low'] for x in v5[-20:])
    high10=max(x['high'] for x in v5[-11:-1]); low10=min(x['low'] for x in v5[-11:-1])
    not_chase_high = c5['close'] < highs20*0.995
    not_chase_low = c5['close'] > lows20*1.005
    vol=abs(c5['close']/p5['close']-1)*100
    violent=vol>3
    topdiv,botdiv,divdet=divergence(v5[-20:])
    candidates=[]
    def add(level, direction, reasons): candidates.append({'level':level,'direction':direction,'reasons':reasons})
    # S/A definitions
    if gold5 and (c5['K']<30 or c5['J']<20) and (gold15 or c15['K']>c15['D']) and (c1['K']>=c1['D'] or c1['J']>p1['J']) and not_chase_high:
        add('S','多头机会',['5m超卖附近金叉','15m金叉或K>D','1h偏多或J上拐','未在20根5m高点0.5%内追高'])
    if death5 and (c5['K']>70 or c5['J']>80) and (death15 or c15['K']<c15['D']) and (c1['K']<=c1['D'] or c1['J']<p1['J']) and not_chase_low:
        add('S','空头风险',['5m超买附近死叉','15m死叉或K<D','1h偏空或J下拐','未在20根5m低点0.5%内追空'])
    if gold5 and (c15['K']>c15['D'] or two_up(v15)) and (p5['J']<20<c5['J'] or (c5['K']<40 and c5['D']<40)):
        add('A','多头机会',['5m金叉','15m K>D或J连续2根上升','5m J从20以下回升或K/D均低于40'])
    if death5 and (c15['K']<c15['D'] or two_down(v15)) and (p5['J']>80>c5['J'] or (c5['K']>60 and c5['D']>60)):
        add('A','空头风险',['5m死叉','15m K<D或J连续2根下降','5m J从80以上回落或K/D均高于60'])
    # specific trigger upgrades
    if gold5 and c5['K']<50 and c5['J']>p5['J']:
        lvl='A' if c15['K']>c15['D'] else 'B'
        if c15['K']>c15['D'] and (c1['K']>=c1['D'] or c1['J']>p1['J']): lvl='S'
        add(lvl,'多头机会',['5m金叉','5m K<50','J较上一根上升','15m/1h支持度决定等级'])
    if death5 and c5['K']>50 and c5['J']<p5['J']:
        lvl='A' if c15['K']<c15['D'] else 'B'
        if c15['K']<c15['D'] and (c1['K']<=c1['D'] or c1['J']<p1['J']): lvl='S'
        add(lvl,'空头风险',['5m死叉','5m K>50','J较上一根下降','15m/1h支持度决定等级'])
    if botdiv:
        add('A' if gold5 else 'C','底背离预警',divdet + (['同时5m金叉，至少A'] if gold5 else []))
    if topdiv:
        add('A' if death5 else 'C','顶背离预警',divdet + (['同时5m死叉，至少A'] if death5 else []))
    if (c5['K']<20 or c5['J']<0) and c5['J']>p5['J']:
        add('C' if not gold5 else 'A','超卖预警',['5m K<20或J<0','J低位上拐'])
    if (c5['K']>80 or c5['J']>100) and c5['J']<p5['J']:
        add('C' if not death5 else 'A','超买预警',['5m K>80或J>100','J高位下拐'])
    if c5['K']>c5['D'] and not gold5 and two_up(v5) and c5['close']<=high10:
        add('B','多头机会',['5m K>D但未新金叉','J连续2根上升','价格尚未明显突破10根高点'])
    if c5['K']<c5['D'] and not death5 and two_down(v5) and c5['close']>=low10:
        add('B','空头风险',['5m K<D但未新死叉','J连续2根下降','价格尚未明显跌破10根低点'])
    if not candidates:
        if gold5: add('C','多头机会',['仅5m金叉，缺少确认'])
        elif death5: add('C','空头风险',['仅5m死叉，缺少确认'])
        elif c5['K']<20 or c5['D']<25 or c5['J']<0: add('C','超卖预警',['5m处于超卖或J极端'])
        elif c5['K']>80 or c5['D']>75 or c5['J']>100: add('C','超买预警',['5m处于超买或J极端'])
    rank={'S':4,'A':3,'B':2,'C':1}
    main=max(candidates,key=lambda x:(rank[x['level']], 1 if '背离' in x['direction'] else 0)) if candidates else {'level':'无','direction':'无有效信号','reasons':['未触发KDJ预警条件']}
    push=main['level'] in ['S','A'] or (main['level'] in ['B','C'] and not violent)
    if violent and main['level'] in ['B','C']: push=False
    extreme=[]
    if c5['J']>120 or c15['J']>120 or c1['J']>120 or c5['J']<-20 or c15['J']<-20 or c1['J']<-20:
        extreme.append('J 值极端，短线波动可能放大')
    return {'symbol':symbol,'price':c5['close'],'kdj':{'5m':c5,'15m':c15,'1h':c1},'prev':{'5m':p5,'15m':p15,'1h':p1},'gold5':gold5,'death5':death5,'gold15':gold15,'death15':death15,'bull_res':bull_res,'bear_res':bear_res,'trend':trend,'volatility_pct':vol,'violent':violent,'main':main,'push':push,'extreme':extreme,'divergence':divdet,'rows':{k:len(v) for k,v in kdjs.items()}}

def invalidation(direction):
    if '多头' in direction or '底背离' in direction or '超卖' in direction:
        return '5m KDJ重新死叉；或J值连续2根下降；或价格跌破信号触发K线低点；或6根5m K线内未继续走强（约30分钟）'
    if '空头' in direction or '顶背离' in direction or '超买' in direction:
        return '5m KDJ重新金叉；或J值连续2根上升；或价格突破信号触发K线高点；或6根5m K线内未继续走弱（约30分钟）'
    return '信号条件消失或30分钟冷却后重新评估'

def resonance(r):
    if r.get('bull_res'): return f"偏多共振；1h趋势{r['trend']}"
    if r.get('bear_res'): return f"偏空共振；1h趋势{r['trend']}"
    return f"5m/15m未形成同向共振；1h趋势{r['trend']}"

def main():
    data={}; errors=[]; results=[]
    for sym in SYMBOLS:
        d={}
        try:
            for itv in INTERVALS:
                d[itv]=run_cli(sym,itv)
            results.append(evaluate(sym,d))
        except Exception as e:
            results.append({'symbol':sym,'error':'数据获取失败: '+str(e)[:300]})
    # cross-market adjustment
    btc=next((r for r in results if r.get('symbol')=='BTCUSDT' and not r.get('error')),None)
    eth=next((r for r in results if r.get('symbol')=='ETHUSDT' and not r.get('error')),None)
    if btc and eth and btc['main']['level']=='S' and '空头' in btc['main']['direction'] and '多头' in eth['main']['direction']:
        if not (eth['kdj']['5m']['K']>eth['kdj']['5m']['D'] and eth['kdj']['15m']['K']>eth['kdj']['15m']['D'] and eth['kdj']['1h']['K']>eth['kdj']['1h']['D']):
            levels=['无','C','B','A','S']; idx=levels.index(eth['main']['level']) if eth['main']['level'] in levels else 0
            if idx>1:
                eth['main']['level']=levels[idx-1]
                eth['main']['reasons'].append('BTCUSDT出现S级风险且ETH多头未满足三周期共振，等级下调一级')
                eth['push']=eth['main']['level'] in ['S','A']
    now=datetime.datetime.now(datetime.timezone.utc).astimezone().strftime('%Y-%m-%d %H:%M:%S %Z')
    out={'time':now,'results':results}
    print(json.dumps(out,ensure_ascii=False,indent=2))

if __name__=='__main__': main()
