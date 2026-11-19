param([Parameter(Mandatory)][string]$Destination)
# Explicit developer action only. No runtime download, no imported DLL execution.
$ErrorActionPreference='Stop'
$version='b12125'
$release=Invoke-RestMethod "https://api.github.com/repos/ggml-org/llama.cpp/releases/tags/$version"
$assets=@($release.assets | Where-Object {$_.name -match 'bin-win-cuda-12.*-x64.zip$'})
if($assets.Count -lt 2){throw 'CUDA-12-Server und CUDA-Runtime nicht eindeutig gefunden. Offizielles Release prüfen.'}
$destinationPath=[System.IO.Path]::GetFullPath($Destination)
if(Test-Path -LiteralPath $destinationPath){throw 'Ziel existiert bereits; ein neues Verzeichnis verwenden.'}
New-Item -ItemType Directory -Path $destinationPath | Out-Null
$records=foreach($asset in $assets){
    $archive=Join-Path $destinationPath $asset.name
    Invoke-WebRequest $asset.browser_download_url -OutFile $archive
    $hash=(Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if(-not $asset.digest -or $asset.digest -ne ('sha256:'+$hash)){throw 'Offizielle SHA-256-Prüfsumme fehlt oder stimmt nicht überein.'}
    Expand-Archive -LiteralPath $archive -DestinationPath (Join-Path $destinationPath 'cuda') -Force
    [pscustomobject]@{url=$asset.browser_download_url;sha256=$hash;release=$version}
}
$records | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $destinationPath 'provenance.json') -Encoding utf8
Write-Output 'Nur den geprüften Unterordner cuda nach <App>\runtime\cuda kopieren. CPU-Laufzeit behalten. Auf NVIDIA testen.'
