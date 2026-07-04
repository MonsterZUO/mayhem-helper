# 桌面会话执行桥（GUI 验收用）
#
# 为什么需要：SSH 会话与桌面会话隔离，SSH 里截不到真实桌面。
# 本脚本由你在 Windows **桌面终端**里手动运行（因此活在桌面会话），
# Mac 侧 agent 经 SSH 往 inbox 投 .ps1 任务，本桥执行后把输出/截图写回 outbox。
#
# 用法：桌面开 PowerShell → powershell -ExecutionPolicy Bypass -File scripts\desktop-bridge.ps1
# 停止：Ctrl+C 或关窗口。只在需要 GUI 验收时开，用完即关（安全边界：
# inbox 仅你自己的 SSH 密钥可写；桥不开则无人能动你桌面）。

$bridge = "$env:USERPROFILE\agent-bridge"
New-Item -ItemType Directory -Force -Path "$bridge\inbox", "$bridge\outbox" | Out-Null
Write-Host "== 桌面执行桥已启动 =="
Write-Host "信箱: $bridge\inbox  (agent 投 .ps1)"
Write-Host "出箱: $bridge\outbox (输出/截图)"
Write-Host "Ctrl+C 停止"

while ($true) {
    Get-ChildItem "$bridge\inbox\*.ps1" -ErrorAction SilentlyContinue | Sort-Object Name | ForEach-Object {
        $name = $_.BaseName
        Write-Host "[$(Get-Date -Format HH:mm:ss)] 执行: $($_.Name)"
        try {
            $out = & $_.FullName *>&1 | Out-String
        } catch {
            $out = "ERROR: $($_ | Out-String)"
        }
        Set-Content -Path "$bridge\outbox\$name.out.txt" -Value $out -Encoding UTF8
        Remove-Item $_.FullName -Force
        Write-Host "  -> outbox\$name.out.txt"
    }
    Start-Sleep -Milliseconds 800
}
