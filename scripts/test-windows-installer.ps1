param([Parameter(Mandatory)][string]$Installer)
$ErrorActionPreference = 'Stop'
$testRoot = Join-Path $env:RUNNER_TEMP ("velo-install-" + [guid]::NewGuid())
$installDir = Join-Path $testRoot 'Thai ทดสอบ'
New-Item -ItemType Directory -Force $testRoot | Out-Null
$process = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDir") -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Installer failed: $($process.ExitCode)" }
$exe = Join-Path $installDir 'velo.exe'
foreach ($file in @($exe, "$installDir/libmpv-2.dll", "$installDir/libmpv-provenance.json")) {
    if (!(Test-Path $file)) { throw "Installed file missing: $file" }
}
# A development PATH would mask a missing packaged dependency.
$savedPath = $env:PATH
$app = $null
try {
    $env:PATH = "$env:SystemRoot/System32;$env:SystemRoot"
    $app = Start-Process -FilePath $exe -WorkingDirectory $testRoot -PassThru
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    do {
        Start-Sleep -Milliseconds 500
        $app.Refresh()
        if ($app.HasExited) { throw "Installed app exited: $($app.ExitCode)" }
    } while ($app.MainWindowHandle -eq 0 -and [DateTime]::UtcNow -lt $deadline)
    if ($app.MainWindowHandle -eq 0) { throw 'Installed app did not create a window' }
    Start-Sleep -Seconds 5
    $app.Refresh()
    if ($app.HasExited) { throw 'Installed app crashed after window creation' }
    Write-Host 'Installed Preview launched with a clean PATH and created a window.'
} finally {
    $env:PATH = $savedPath
    if ($app -and !$app.HasExited) {
        $app.CloseMainWindow() | Out-Null
        if (!$app.WaitForExit(5000)) { $app.Kill() }
    }
}
# Leave the isolated installation for runner diagnostics; the ephemeral runner
# is discarded. Never invoke an uninstaller against a user's installation.
