$ErrorActionPreference = 'Stop'
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$packageName = 'ecs-voyager'
$version = '0.2.7'

# Determine architecture
$is64bit = Get-ProcessorBits 64
if ($is64bit) {
    $url64 = "https://github.com/benbpyle/ecs-voyager/releases/download/v$version/ecs-voyager-v$version-x86_64-pc-windows-msvc.zip"
    $checksum64 = '' # Will be filled during package creation
    $checksumType64 = 'sha256'
} else {
    Write-Error "32-bit Windows is not supported"
}

$packageArgs = @{
    packageName    = $packageName
    unzipLocation  = $toolsDir
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = $checksumType64
}

Install-ChocolateyZipPackage @packageArgs

# Create shim for the executable
$exePath = Join-Path $toolsDir "ecs-voyager.exe"
Install-BinFile -Name 'ecs-voyager' -Path $exePath

Write-Host ""
Write-Host "ECS Voyager has been installed!" -ForegroundColor Green
Write-Host ""
Write-Host "Usage: ecs-voyager" -ForegroundColor Cyan
Write-Host "       ecs-voyager --help" -ForegroundColor Cyan
Write-Host ""
Write-Host "For ECS Exec and Port Forwarding features, install session-manager-plugin:" -ForegroundColor Yellow
Write-Host "  choco install session-manager-plugin" -ForegroundColor Cyan
Write-Host ""
