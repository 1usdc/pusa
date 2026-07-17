#!/usr/bin/env python3
import urllib.request, json, math, datetime, ssl

SYMBOLS=['BTCUSDT','ETHUSDT']
INTERVALS={'5m':5*60*1000,'15m':15*60*1000,'1h':60*60*1000}
LIMIT=100
N=9

def fetch_klines(symbol, interval, end_ms):
    url=f'https://api.binance.com/api/v3/klines?symbol={symbol}&interval={interval}&limit={LIMIT}&endTime={end_ms}'
    with urllib.request.urlopen(url, timeout=15) as r:
        data=json.loads(r.read().decode())
    return data

def calc_kdj(klines):
    K=50.0; D=50.0
    out=[]
    for i,k in enumerate(klines):
        high=float(k[2]); low=float(k[3]); close=float(k[4])
        start=max(0,i-N+1)
        hh=max(float(x[2]) for x in klines[start:i+1])
        ll=min(float(x[3]) for x in klines[start:i+1])
        rsv=50.0 if hh==ll else (close-ll)/(hh-ll)*100.0
        K=(2*K+rsv)/3.0
        D=(2*D+K)/3.0
        J=3*K-2*D
        out.append({'K':K,'D':D,'J':J,'close':close,'high':high,'low':low,'open':float(k[1]),'openTime':int(k[0]),'closeTime':int(k[6])})
    return out

def up2(arr, key='J'):
    return len(arr)>=3 and arr[-1][key] > arr[-2][key] > arr[-3][key]
def down2(arr, key='J'):
    return len(arr)>=3 and arr[-1][key] < arr[-2][key] < arr[-3][key]
def starts_up(arr):
    return len(arr)>=2 and arr[-1]['J'] > arr[-2]['J']
def starts_down(arr):
    return len(arr)>=2 and arr[-1]['J'] < arr[-2]['J']

def cross(arr):
    if len(arr)<2: return (False,False)
    p,c=arr[-2],arr[-1]
    golden=c['K']>c['D'] and p['K']<=p['D']
    death=c['K']<c['D'] and p['K']>=p['D']
    return golden,death

def divergence(kdj5):
    # local extrema within last 20, using close vs J, exclude endpoints for extrema but include positions relative to neighbors
    win=kdj5[-20:]
    highs=[]; lows=[]
    for i in range(1,len(win)-1):
        if win[i]['close']>win[i-1]['close'] and win[i]['close']>win[i+1]['close']:
            highs.append((i,win[i]['close'],win[i]['J']))
        if win[i]['close']<win[i-1]['close'] and win[i]['close']<win[i+1]['close']:
            lows.append((i,win[i]['close'],win[i]['J']))
    top=False; bottom=False
    top_pts=None; bottom_pts=None
    if len(highs)>=2:
        a,b=highs[-2],highs[-1]
        top = b[1] > a[1] and b[2] < a[2]
        top_pts=(a,b)
    if len(lows)>=2:
        a,b=lows[-2],lows[-1]
        bottom = b[1] < a[1] and b[2] > a[2]
        bottom_pts=(a,b)
    return top,bottom,top_pts,bottom_pts

def near_high_low(kdj5):
    c=kdj5[-1]['close']; hi=max(x['high'] for x in kdj5[-20:]); lo=min(x['low'] for x in kdj5[-20:])
    near_hi=(hi-c)/hi <=0.005 if hi else False
    near_lo=(c-lo)/lo <=0.005 if lo else False
    break10hi=c>max(x['high'] for x in kdj5[-11:-1])
    break10lo=c<min(x['low'] for x in kdj5[-11:-1])
    return near_hi, near_lo, break10hi, break10lo, hi, lo

def evaluate(symbol, allkdj):
    res={'symbol':symbol,'error':None}
    for it,a in allkdj.items():
        if len(a)<50:
            res['error']=f'{it} 数据不足，仅 {len(a)} 根'; return res
        c=a[-1]
        if c['close']<=0 or any(not math.isfinite(c[x]) for x in ['K','D','J']):
            res['error']=f'{it} 价格或指标异常'; return res
    k5,k15,k1=allkdj['5m'],allkdj['15m'],allkdj['1h']
    c5,c15,c1=k5[-1],k15[-1],k1[-1]
    g5,d5=cross(k5); g15,d15=cross(k15); g1,d1=cross(k1)
    j5up=starts_up(k5); j5down=starts_down(k5); j15up2=up2(k15); j15down2=down2(k15)
    j1_up=starts_up(k1); j1_down=starts_down(k1)
    near_hi,near_lo,break10hi,break10lo,hi20,lo20=near_high_low(k5)
    topdiv,botdiv,toppts,botpts=divergence(k5)
    bull_res=(g5 and g15) or (c5['K']>c5['D'] and c15['K']>c15['D'])
    bear_res=(d5 and d15) or (c5['K']<c5['D'] and c15['K']<c15['D'])
    trend='偏多' if c1['K']>c1['D'] else '偏空' if c1['K']<c1['D'] else '中性'
    vol=abs(c5['close']/k5[-2]['close']-1)*100 if len(k5)>=2 else 0
    highvol=vol>3
    candidates=[]
    def add(level, direction, reasons, kind='normal'):
        candidates.append({'level':level,'direction':direction,'reasons':reasons,'kind':kind})
    # S
    if g5 and (c5['K']<30 or c5['J']<20) and (g15 or c15['K']>c15['D']) and (c1['K']>=c1['D'] or j1_up) and not near_hi:
        add('S','多头机会',['5m 超卖附近金叉且 K<30 或 J<20','15m 金叉或 K>D','1h K>=D 或 J 上拐','当前价格未在近20根5m高点0.5%内追高'])
    if d5 and (c5['K']>70 or c5['J']>80) and (d15 or c15['K']<c15['D']) and (c1['K']<=c1['D'] or j1_down) and not near_lo:
        add('S','空头风险',['5m 超买附近死叉且 K>70 或 J>80','15m 死叉或 K<D','1h K<=D 或 J 下拐','当前价格未在近20根5m低点0.5%内追空'])
    # divergence priority
    if topdiv:
        add('A' if d5 else 'C','顶背离预警',['近20根5m价格局部高点抬高且对应J高点降低'] + (['同时出现5m死叉，至少A级风险信号'] if d5 else []),'div')
    if botdiv:
        add('A' if g5 else 'C','底背离预警',['近20根5m价格局部低点降低且对应J低点抬高'] + (['同时出现5m金叉，至少A级机会信号'] if g5 else []),'div')
    # A
    if g5 and (c15['K']>c15['D'] or j15up2) and ((k5[-2]['J']<20 and c5['J']>k5[-2]['J']) or (c5['K']<40 and c5['D']<40)):
        add('A','多头机会',['5m 出现金叉','15m K>D 或 J 连续2根上升','5m J从20以下回升或K、D均低于40'])
    if d5 and (c15['K']<c15['D'] or j15down2) and ((k5[-2]['J']>80 and c5['J']<k5[-2]['J']) or (c5['K']>60 and c5['D']>60)):
        add('A','空头风险',['5m 出现死叉','15m K<D 或 J 连续2根下降','5m J从80以上回落或K、D均高于60'])
    # trigger rules upgrades
    if g5 and c5['K']<50 and j5up:
        lvl='A' if c15['K']>c15['D'] else 'C'
        if c15['K']>c15['D'] and c1['K']>=c1['D']: lvl='S'
        add(lvl,'多头机会',['5m KDJ金叉','5m K<50','5m J较上一根上升'] + (['15m K>D'] if c15['K']>c15['D'] else []) + (['1h K>=D'] if c1['K']>=c1['D'] else []))
    if d5 and c5['K']>50 and j5down:
        lvl='A' if c15['K']<c15['D'] else 'C'
        if c15['K']<c15['D'] and c1['K']<=c1['D']: lvl='S'
        add(lvl,'空头风险',['5m KDJ死叉','5m K>50','5m J较上一根下降'] + (['15m K<D'] if c15['K']<c15['D'] else []) + (['1h K<=D'] if c1['K']<=c1['D'] else []))
    # oversold/overbought
    if (c5['K']<20 or c5['J']<0) and j5up:
        add('B' if g5 else 'C','超卖预警',['5m K<20 或 J<0','J 值开始从低位上拐'])
    if (c5['K']>80 or c5['J']>100) and j5down:
        add('B' if d5 else 'C','超买预警',['5m K>80 或 J>100','J 值开始从高位下拐'])
    # B observation
    if c5['K']>c5['D'] and not g5 and up2(k5) and not break10hi:
        add('B','多头机会',['5m K>D但未发生新金叉','5m J连续2根上升','价格尚未明显突破最近10根K线高点'])
    if c5['K']<c5['D'] and not d5 and down2(k5) and not break10lo:
        add('B','空头风险',['5m K<D但未发生新死叉','5m J连续2根下降','价格尚未明显跌破最近10根K线低点'])
    # C single states
    if not candidates:
        state=[]
        if g5: state.append('5m单周期金叉')
        if d5: state.append('5m单周期死叉')
        if c5['K']<20 and c5['D']<25: state.append('5m超卖区')
        if c5['K']>80 and c5['D']>75: state.append('5m超买区')
        if c5['J']<0: state.append('J<0极度超卖')
        if c5['J']>100: state.append('J>100极度超买')
        if state:
            add('C','状态提醒',state)
    if highvol:
        candidates=[x for x in candidates if x['level'] in ['A','S']]
    order={'S':4,'A':3,'B':2,'C':1}
    candidates.sort(key=lambda x:(order[x['level']], 1 if x['kind']=='div' else 0, 1 if (bull_res or bear_res) else 0), reverse=True)
    main=candidates[0] if candidates else None
    res.update({'price':c5['close'],'kdj':{'5m':c5,'15m':c15,'1h':c1},'cross':{'5m':{'golden':g5,'death':d5},'15m':{'golden':g15,'death':d15}},'resonance':{'bull':bull_res,'bear':bear_res,'trend':trend},'main':main,'candidates':candidates,'highvol':highvol,'vol_pct':vol,'hi20':hi20,'lo20':lo20,'near_hi':near_hi,'near_lo':near_lo})
    return res

def fmt(x): return f'{x:.2f}'

def main():
    now_ms=int(datetime.datetime.now(datetime.timezone.utc).timestamp()*1000)
    endtimes={it:(now_ms//ms)*ms-1 for it,ms in INTERVALS.items()}
    out={'time_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(timespec='seconds'), 'results':{}, 'exceptions':[]}
    for sym in SYMBOLS:
        allkdj={}
        try:
            for it,end in endtimes.items():
                kl=fetch_klines(sym,it,end)
                if not isinstance(kl,list) or len(kl)<50:
                    raise RuntimeError(f'{it} K线不足: {len(kl) if isinstance(kl,list) else "非列表"}')
                allkdj[it]=calc_kdj(kl)
            out['results'][sym]=evaluate(sym,allkdj)
        except Exception as e:
            out['results'][sym]={'symbol':sym,'error':str(e)}
            out['exceptions'].append(f'{sym}: {e}')
    print(json.dumps(out, ensure_ascii=False, indent=2))

if __name__=='__main__': main()
