$ErrorActionPreference = 'Stop'
$packageName = 'ecs-voyager'

# Remove the shim
Uninstall-BinFile -Name 'ecs-voyager'

Write-Host "ECS Voyager has been uninstalled." -ForegroundColor Green
