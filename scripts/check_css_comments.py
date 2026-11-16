"""全文字符级扫描：定位真正破坏 CSS 的注释错误。

- 注释内再出现 `/*`：浏览器视为注释文本（无害，但 grep 计数会虚高）；
- 注释外出现 `*/`：语法错误；
- 文件结束 depth 仍为 1：注释未闭合，其后所有规则全部失效。
"""
depth = 0
s = open('ui/assets/css/main.css', encoding='utf-8').read()
line = 1
i = 0
n = len(s)
while i < n:
    c = s[i]
    if c == '\n':
        line += 1
        i += 1
        continue
    if depth == 0 and c == '/' and i + 1 < n and s[i + 1] == '*':
        depth = 1
        i += 2
        continue
    if depth == 1 and c == '*' and i + 1 < n and s[i + 1] == '/':
        depth = 0
        i += 2
        continue
    if depth == 1 and c == '/' and i + 1 < n and s[i + 1] == '*':
        print('nested /* inside comment at line', line, ':', s[i:i + 40].split('\n')[0])
    if depth == 0 and c == '*' and i + 1 < n and s[i + 1] == '/':
        print('stray */ outside comment at line', line, ':', s[max(0, i - 40):i + 2].split('\n')[-1])
    i += 1
print('EOF depth =', depth, '(1 = 有注释未闭合，其后样式全部失效)')
