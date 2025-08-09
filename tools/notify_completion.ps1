# Claude Code 作業完了通知スクリプト
param(
    [string]$Title = "Claude Code",
    [string]$Message = "作業が完了しました"
)

try {
    # トースト通知を試行
    Add-Type -AssemblyName System.Windows.Forms
    $notification = New-Object System.Windows.Forms.NotifyIcon
    $notification.Icon = [System.Drawing.SystemIcons]::Information
    $notification.BalloonTipTitle = $Title
    $notification.BalloonTipText = $Message
    $notification.Visible = $true
    $notification.ShowBalloonTip(5000)
    
    # 5秒待機後にクリーンアップ
    Start-Sleep -Seconds 6
    $notification.Dispose()
    
    Write-Host "通知を送信しました: $Message"
} catch {
    # フォールバック: システムビープ音
    [System.Media.SystemSounds]::Beep.Play()
    Write-Host "ビープ音で通知しました: $Message"
}