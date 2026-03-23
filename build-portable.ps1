cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

New-Item -ItemType Directory -Force -Path portable | Out-Null
Copy-Item target\release\Calculator.exe portable\Calculator.exe -Force

Write-Host "Built: portable\Calculator.exe ($([math]::Round((Get-Item portable\Calculator.exe).Length / 1KB)) KB)"
