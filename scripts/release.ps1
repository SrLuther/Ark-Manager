<#
.SYNOPSIS
    Script oficial de release do Ark Manager.
    USO: .\scripts\release.ps1 -Version 1.2.0

.DESCRIPTION
    Valida versionamento, changelog e integridade antes de buildar e publicar.
    Nada vai ao GitHub sem passar por aqui.
#>

param(
    [Parameter(Mandatory)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent

function Fail($msg) { Write-Host "ERRO: $msg" -ForegroundColor Red; exit 1 }
function Ok($msg)   { Write-Host "OK   $msg" -ForegroundColor Green }
function Info($msg) { Write-Host "     $msg" -ForegroundColor Cyan }

Write-Host ""
Write-Host "========================================" -ForegroundColor Yellow
Write-Host "  Ark Manager — Release v$Version" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Yellow
Write-Host ""

# ── 1. Git working tree limpo ──────────────────────────────────────────────────
Info "Verificando git working tree..."
$dirty = (git -C $root status --porcelain) | Out-String
if ($dirty.Trim() -ne '') { Fail "Existem arquivos nao commitados. Faca commit antes de lancar.`n$dirty" }
Ok "Working tree limpo."

# ── 2. Branch correta ─────────────────────────────────────────────────────────
Info "Verificando branch..."
$branch = git -C $root rev-parse --abbrev-ref HEAD
if ($branch -ne "main" -and $branch -ne "develop") {
    Fail "Branch atual: '$branch'. Release só pode ser feita a partir de 'main' ou 'develop'."
}
Ok "Branch: $branch"

# ── 3. Consistência de versão nos 3 arquivos ──────────────────────────────────
Info "Verificando versões em package.json / tauri.conf.json / Cargo.toml..."

$pkgVersion   = (Get-Content "$root\package.json" | ConvertFrom-Json).version
$tauriContent = Get-Content "$root\src-tauri\tauri.conf.json" -Raw | ConvertFrom-Json
$tauriVersion = $tauriContent.version
$cargoVersion = (Select-String -Path "$root\src-tauri\Cargo.toml" -Pattern '^version\s*=\s*"(.+)"').Matches[0].Groups[1].Value

$mismatch = @()
if ($pkgVersion   -ne $Version) { $mismatch += "package.json: $pkgVersion" }
if ($tauriVersion -ne $Version) { $mismatch += "tauri.conf.json: $tauriVersion" }
if ($cargoVersion -ne $Version) { $mismatch += "Cargo.toml: $cargoVersion" }

if ($mismatch.Count -gt 0) {
    Write-Host ""
    Write-Host ("Os seguintes arquivos precisam ter a versao {0}:" -f $Version) -ForegroundColor Red
    $mismatch | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    Write-Host ""
    Write-Host "Atualize manualmente ou execute:" -ForegroundColor Yellow
    Write-Host "  (package.json)      `"version`": `"$Version`"" -ForegroundColor Yellow
    Write-Host "  (tauri.conf.json)   `"version`": `"$Version`"" -ForegroundColor Yellow
    Write-Host "  (Cargo.toml)        version = `"$Version`"" -ForegroundColor Yellow
    exit 1
}
Ok "Todas as versoes estao em $Version."

# ── 4. CHANGELOG tem entrada para esta versao ─────────────────────────────────
Info "Verificando CHANGELOG.md..."
$changelog = Get-Content "$root\CHANGELOG.md" -Raw

if ($changelog -notmatch "##\s+\[$Version\]") {
    Fail "CHANGELOG.md nao tem entrada para [$Version].`nDocumente as mudancas antes de lancar."
}

# Extrai o bloco da versao (entre o cabecalho dela e o proximo ## ou fim do arquivo)
$versionBlock = [regex]::Match(
    $changelog,
    "##\s+\[$([regex]::Escape($Version))\][^\n]*\n(.*?)(?=\n##\s+\[|\z)",
    [System.Text.RegularExpressions.RegexOptions]::Singleline
).Groups[1].Value

# Linhas de conteudo real: ignora linhas em branco, separadores e cabecalhos de subsecao (###)
$contentLines = ($versionBlock -split "`n" | Where-Object {
    $_.Trim() -ne '' -and
    $_.Trim() -notmatch '^---+$' -and
    $_.Trim() -notmatch '^###'
})

if ($contentLines.Count -eq 0) {
    Write-Host ""
    Write-Host "ERRO: A secao [$Version] no CHANGELOG.md existe mas esta vazia." -ForegroundColor Red
    Write-Host ""
    Write-Host "Adicione pelo menos uma linha descrevendo o que mudou nesta versao." -ForegroundColor Yellow
    Write-Host "Exemplo:" -ForegroundColor Yellow
    Write-Host "  ### Adicionado" -ForegroundColor Yellow
    Write-Host "  - Descricao da mudanca" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

Info "Descricao encontrada ($($contentLines.Count) itens):"
$contentLines | Select-Object -First 5 | ForEach-Object { Info "  $_" }
if ($contentLines.Count -gt 5) { Info "  ... e mais $($contentLines.Count - 5) itens." }

# Garante que a secao [Nao lancado] nao tem conteudo alem dos cabecalhos
$unreleasedBlock = [regex]::Match($changelog, '## \[Não lançado\](.*?)(?=## \[|\z)', [System.Text.RegularExpressions.RegexOptions]::Singleline).Groups[1].Value
$unreleasedLines = ($unreleasedBlock -split "`n" | Where-Object { $_.Trim() -ne '' -and $_.Trim() -notmatch '^---$' })
if ($unreleasedLines.Count -gt 0) {
    Write-Host ""
    Write-Host "AVISO: A secao [Nao lancado] ainda tem conteudo nao versionado:" -ForegroundColor Yellow
    $unreleasedLines | Select-Object -First 5 | ForEach-Object { Write-Host "  $_" -ForegroundColor Yellow }
    Write-Host ""
    $resp = Read-Host "Continuar mesmo assim? (s/N)"
    if ($resp -notmatch '^[sS]$') { exit 1 }
}
Ok "CHANGELOG.md validado para [$Version]."

# ── 5. Tag nao existe ainda ────────────────────────────────────────────────────
Info "Verificando se tag v$Version ja existe..."
$existingTag = git -C $root tag --list "v$Version"
if ($existingTag) { Fail "Tag v$Version ja existe. Incremente a versao." }
Ok "Tag v$Version disponivel."

# ── 6. Build ──────────────────────────────────────────────────────────────────
Write-Host ""
Info "Iniciando build de producao..."
$key = (Get-Content "$root\src-tauri\signing\ark-manager.key" -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY          = $key
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "Ciano7795@"

Set-Location $root
npm run tauri build
if ($LASTEXITCODE -ne 0) { Fail "Build falhou." }
Ok "Build concluido."

# ── 7. Commit de versao + tag ─────────────────────────────────────────────────
Write-Host ""
Info "Criando commit de versao e tag..."
git -C $root add package.json src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock CHANGELOG.md
git -C $root commit -m "chore: release v$Version"
git -C $root tag -a "v$Version" -m "Release v$Version"
Ok "Commit e tag v$Version criados."

# ── 8. Push ───────────────────────────────────────────────────────────────────
Write-Host ""
$pushResp = Read-Host "Fazer push para GitHub agora? (s/N)"
if ($pushResp -match '^[sS]$') {
    git -C $root push origin $branch
    git -C $root push origin "v$Version"
    Ok "Push concluido. Crie o Release no GitHub a partir da tag v$Version."
} else {
    Info "Push pulado. Execute depois:"
    Info "  git push origin $branch"
    Info "  git push origin v$Version"
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "  Release v$Version preparada!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
