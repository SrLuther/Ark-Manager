param([string]$Token)

$ErrorActionPreference = 'Stop'
$root = "C:\Users\Ciano\Documents\Ark Manager"
$repo = "SrLuther/Ark-Manager"
$version = "1.1.0"
$tag = "v$version"

if (-not $Token) {
    # Tenta obter do Credential Manager
    $credFile = [System.IO.Path]::GetTempFileName()
    Set-Content $credFile "protocol=https`nhost=github.com`n"
    $credLines = Get-Content $credFile | git credential fill
    Remove-Item $credFile -ErrorAction SilentlyContinue
    foreach ($line in $credLines) {
        if ($line -match "^password=(.+)") { $Token = $Matches[1]; break }
    }
}

if (-not $Token) { Write-Host "ERRO: token nao encontrado"; exit 1 }
Write-Host "Token obtido ($($Token.Length) chars)"

$h = @{
    Authorization = "token $Token"
    Accept = "application/vnd.github.v3+json"
}

# --- Verifica se release já existe ---
$existing = $null
try { $existing = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/tags/$tag" -Headers $h } catch {}

if ($existing) {
    Write-Host "Release já existe: $($existing.html_url)"
    $uploadBase = $existing.upload_url -replace '\{.*\}',''
} else {
    # Lê notas do CHANGELOG
    $changelog = Get-Content "$root\CHANGELOG.md" -Raw
    $notes = [regex]::Match($changelog, "## \[$version\][^\n]*\n(.*?)(?=\n## \[|\z)", [System.Text.RegularExpressions.RegexOptions]::Singleline).Groups[1].Value.Trim()

    $body = @{
        tag_name   = $tag
        name       = "Ark Manager $tag"
        body       = $notes
        draft      = $false
        prerelease = $false
    } | ConvertTo-Json -Depth 5

    $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases" -Method Post -Headers $h -Body $body -ContentType "application/json"
    Write-Host "Release criada: $($rel.html_url)"
    $uploadBase = $rel.upload_url -replace '\{.*\}',''
}

# --- Upload dos arquivos ---
$files = @(
    "$root\src-tauri\target\release\bundle\msi\Ark Manager_${version}_x64_en-US.msi",
    "$root\src-tauri\target\release\bundle\msi\Ark Manager_${version}_x64_en-US.msi.sig",
    "$root\src-tauri\target\release\bundle\nsis\Ark Manager_${version}_x64-setup.exe",
    "$root\src-tauri\target\release\bundle\nsis\Ark Manager_${version}_x64-setup.exe.sig"
)

foreach ($f in $files) {
    $name = [System.IO.Path]::GetFileName($f)
    $urlName = [Uri]::EscapeDataString($name)
    $uploadUrl = "${uploadBase}?name=${urlName}"

    $ct = if ($f -match "\.sig$") { "text/plain" } elseif ($f -match "\.msi$") { "application/x-msi" } else { "application/octet-stream" }

    Write-Host "Uploading: $name ..."
    $bytes = [System.IO.File]::ReadAllBytes($f)
    try {
        $r = Invoke-RestMethod -Uri $uploadUrl -Method Post -Headers $h -Body $bytes -ContentType $ct -TimeoutSec 300
        Write-Host "  OK: $($r.browser_download_url)"
    } catch {
        Write-Host "  ERRO: $($_.Exception.Message)"
    }
}

# --- Gera latest.json ---
$msiSig  = Get-Content "$root\src-tauri\target\release\bundle\msi\Ark Manager_${version}_x64_en-US.msi.sig" -Raw
$nsisSig = Get-Content "$root\src-tauri\target\release\bundle\nsis\Ark Manager_${version}_x64-setup.exe.sig" -Raw
$pubDate = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")

$latest = @{
    version  = $version
    notes    = "Ark Manager $tag"
    pub_date = $pubDate
    platforms = @{
        "windows-x86_64" = @{
            signature = $nsisSig.Trim()
            url       = "https://github.com/$repo/releases/download/$tag/Ark Manager_${version}_x64-setup.exe"
        }
    }
} | ConvertTo-Json -Depth 5

Set-Content "$root\latest.json" $latest -Encoding UTF8
Write-Host ""
Write-Host "latest.json gerado."
Write-Host ""
Write-Host "Proximo passo: commitar e fazer push do latest.json"
Write-Host "  git add latest.json"
Write-Host "  git commit -m 'chore: update latest.json for v$version'"
Write-Host "  git push origin main"
