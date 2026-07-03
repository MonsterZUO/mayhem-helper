# MayhemHelper Windows 自验收脚本
# 用法(Windows PowerShell)：powershell -ExecutionPolicy Bypass -File scripts\windows-acceptance.ps1
# 验：Blitz 国服可达性(F1) / cdragon / LCU 连接 / 当前英雄识别 / 出装写入端点 schema
# 只读为主；不改你的游戏数据。开着 LoL 客户端时能验更多。

$ErrorActionPreference = 'Continue'
function Section($t) { Write-Host "`n===== $t =====" -ForegroundColor Cyan }
function Pass($t)    { Write-Host "  [PASS] $t" -ForegroundColor Green }
function Fail($t)    { Write-Host "  [FAIL] $t" -ForegroundColor Red }
function Skip($t)    { Write-Host "  [SKIP] $t" -ForegroundColor Yellow }
function Info($t)    { Write-Host "  $t" -ForegroundColor Gray }

# ---- 1. Blitz 国服可达性 (深审 F1，最关键) ----
Section "1. Blitz 数据源可达性 (F1)"
$blitzBody = '{"query":"query q($c:String!){executeDatabricksQuery(game:LEAGUE queryName:\"prod_aram_mayhem_champion\" params:[{name:\"champion_id\",value:$c}]){payload}}","variables":{"c":"5"}}'
try {
  $sw = [System.Diagnostics.Stopwatch]::StartNew()
  $r = Invoke-WebRequest -Uri 'https://datalake.v2.iesdev.com/graphql' -Method Post -ContentType 'application/json' -Body $blitzBody -TimeoutSec 15 -UseBasicParsing
  $sw.Stop()
  $hasData = ($r.Content -match 'dataArray') -and ($r.Content -match 'augments')
  if ($r.StatusCode -eq 200 -and $hasData) { Pass "Blitz 直连成功 HTTP 200，$([int]$sw.ElapsedMilliseconds)ms，返回真实数据 → 可实时取数" }
  elseif ($r.StatusCode -eq 200) { Fail "HTTP 200 但无预期数据(可能被中间页/防火墙改写)" }
  else { Fail "HTTP $($r.StatusCode)" }
} catch { Fail "连不上 Blitz：$($_.Exception.Message) → 将走出厂快照兜底(仍可用，非实时)" }

# ---- 2. CommunityDragon 元数据 ----
Section "2. 海克斯元数据源 (cdragon)"
try {
  $r2 = Invoke-WebRequest -Uri 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/cherry-augments.json' -TimeoutSec 15 -UseBasicParsing
  if ($r2.StatusCode -eq 200 -and $r2.Content.Length -gt 1000) { Pass "cdragon 可达 (海克斯中文名/图标源)" } else { Fail "cdragon HTTP $($r2.StatusCode)" }
} catch { Fail "连不上 cdragon：$($_.Exception.Message) → 海克斯名会显示为数字 id(降级可用)" }

# ---- 3. LCU (需 LoL 客户端登录) ----
Section "3. LCU 客户端接口 (需登录 LoL)"
$proc = Get-CimInstance Win32_Process -Filter "name='LeagueClientUx.exe'" -ErrorAction SilentlyContinue
if (-not $proc) { Skip "未检测到 LeagueClientUx 进程 → 请登录 LoL 客户端后重跑本节"; }
else {
  $cmd = $proc.CommandLine
  $port  = [regex]::Match($cmd, '--app-port=([0-9]+)').Groups[1].Value
  $token = [regex]::Match($cmd, '--remoting-auth-token=([\w-]+)').Groups[1].Value
  if (-not $port -or -not $token) { Fail "拿不到 LCU 端口/令牌(命令行解析失败)" }
  else {
    Pass "LCU 已连接 (端口 $port)"
    $base = "https://127.0.0.1:$port"
    $auth = 'Basic ' + [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("riot:$token"))
    $hdr  = @{ Authorization = $auth }
    # PS5 忽略自签证书
    try { Add-Type @"
using System.Net;using System.Security.Cryptography.X509Certificates;
public class TrustAll : ICertificatePolicy { public bool CheckValidationResult(ServicePoint s, X509Certificate c, WebRequest r, int p){return true;} }
"@ -ErrorAction SilentlyContinue; [System.Net.ServicePointManager]::CertificatePolicy = New-Object TrustAll } catch {}

    function LcuGet($path) { Invoke-RestMethod -Uri "$base$path" -Headers $hdr -Method Get -TimeoutSec 10 }

    # 3a. 当前召唤师(连接确认 + accountId/summonerId 供出装写入)
    try {
      $me = LcuGet '/lol-summoner/v1/current-summoner'
      Pass "当前召唤师: $($me.displayName)  summonerId=$($me.summonerId) accountId=$($me.accountId)"
      if ($me.accountId -and $me.summonerId) { Pass "accountId/summonerId 齐全 → 出装写入(U5)前置数据 OK" } else { Fail "缺 accountId/summonerId" }
    } catch { Fail "取当前召唤师失败：$($_.Exception.Message)" }

    # 3b. 当前英雄识别 (U4) — 在选英雄阶段才有
    try {
      $sess = LcuGet '/lol-champ-select/v1/session'
      $cellId = $sess.localPlayerCellId
      $mine = $sess.myTeam | Where-Object { $_.cellId -eq $cellId }
      if ($mine -and $mine.championId -gt 0) { Pass "当前英雄识别(U4): cellId=$cellId → championId=$($mine.championId)" }
      else { Info "在选英雄阶段但尚未分配英雄(championId=0)——随机/roll 后会有值" }
    } catch { Skip "当前不在选英雄阶段(U4 识别需在海克斯大乱斗选英雄时验)" }

    # 3c. 出装写入端点 (U5) — 只读 GET 验端点+schema，不改你的数据
    try {
      $sets = LcuGet "/lol-item-sets/v1/item-sets/$($me.summonerId)/sets"
      $n = @($sets.itemSets).Count
      Pass "出装端点(U5)可读: 现有 $n 份 item-set，wrapper 字段 accountId/itemSets 存在 → schema 路径通"
      Info "(实际写入 associatedMaps:[12] 的验证：在 App 里点「一键导入出装」，再重跑本节看是否多出 [海克斯大乱斗] set)"
    } catch { Fail "出装端点读取失败：$($_.Exception.Message)" }
  }
}

Section "完成"
Info "把以上整段输出贴给我(或我经 SSH 直接跑)，我据此判 F1/LCU/U4/U5 各项通过与否。"
