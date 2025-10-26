#!/usr/bin/env pwsh
# Generate a random Ethereum private key for testing
# WARNING: This is for TESTING ONLY. Never use in production with real funds!

$bytes = New-Object byte[] 32
$rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
$rng.GetBytes($bytes)
$privateKey = ($bytes | ForEach-Object { $_.ToString("x2") }) -join ''

Write-Host "Generated Test Private Key (DO NOT USE WITH REAL FUNDS):" -ForegroundColor Yellow
Write-Host $privateKey -ForegroundColor Green
Write-Host ""
Write-Host "Add this to your .env file:" -ForegroundColor Cyan
Write-Host "PRIVATE_KEY=$privateKey"
