param([switch]$SkipInno)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$toolRoot = Join-Path (Split-Path -Parent (Split-Path -Parent $PSScriptRoot)) '.tools'
New-Item -ItemType Directory -Path $toolRoot -Force | Out-Null

function Get-PinnedFile([string]$Uri, [string]$Path, [string]$Sha256) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        $partial = $Path + '.' + [guid]::NewGuid() + '.part'
        try {
            Invoke-WebRequest -Uri $Uri -OutFile $partial
            if ((Get-FileHash -LiteralPath $partial).Hash -ne $Sha256) {
                throw "Falsche SHA-256-Prüfsumme für $Uri"
            }
            Move-Item -LiteralPath $partial -Destination $Path
        } finally {
            if (Test-Path -LiteralPath $partial) { Remove-Item -LiteralPath $partial }
        }
    }
    if ((Get-FileHash -LiteralPath $Path).Hash -ne $Sha256) {
        throw "Falsche SHA-256-Prüfsumme für $Path"
    }
}

$runtime = Join-Path $toolRoot 'llama-b12125-cpu'
if (-not (Test-Path -LiteralPath (Join-Path $runtime 'llama-server.exe') -PathType Leaf)) {
    $archive = Join-Path $toolRoot 'llama-b12125-cpu.zip'
    Get-PinnedFile 'https://github.com/ggml-org/llama.cpp/releases/download/b12125/llama-b12125-bin-win-cpu-x64.zip' `
        $archive '968091CCB8C230E0CF7E0EED386310703BEBAE4A4FD4375F0C57DB53958307D8'
    Expand-Archive -LiteralPath $archive -DestinationPath $runtime
}
Get-PinnedFile 'https://raw.githubusercontent.com/ggml-org/llama.cpp/7d516e25ead4653169236b3f90d13f7bd8629724/LICENSE' `
    (Join-Path $runtime 'llama.cpp-MIT.txt') '94F29BBED6A22C35B992C5C6EBF0E7C92F13B836B90F36F461C9CF2F0F1D010D'

$webview = Join-Path $toolRoot 'MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
if (-not (Test-Path -LiteralPath $webview -PathType Leaf)) {
    $partial = $webview + '.' + [guid]::NewGuid() + '.part.exe'
    try {
        Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/?linkid=2124701' -OutFile $partial
        $signature = Get-AuthenticodeSignature -LiteralPath $partial
        if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,') {
            throw 'Ungültige Microsoft-Signatur für den WebView2-Installer.'
        }
        Move-Item -LiteralPath $partial -Destination $webview
    } finally {
        if (Test-Path -LiteralPath $partial) { Remove-Item -LiteralPath $partial }
    }
}

$compiler = Join-Path $toolRoot 'inno/compiler'
if (-not $SkipInno -and -not (Test-Path -LiteralPath (Join-Path $compiler 'ISCC.exe') -PathType Leaf)) {
    $installer = Join-Path $toolRoot 'innosetup-7.1.1-x64.exe'
    Get-PinnedFile 'https://github.com/jrsoftware/issrc/releases/download/is-7_1_1/innosetup-7.1.1-x64.exe' `
        $installer '8A5A6711C3766A6E5FB74F280BA0E3551E719CE94538D901FDBD1F8F9920B3F0'
    $arguments = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/SP-', '/NORESTART', '/CURRENTUSER', "/DIR=`"$compiler`"")
    $process = Start-Process -FilePath $installer -ArgumentList $arguments -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0 -or -not (Test-Path -LiteralPath (Join-Path $compiler 'ISCC.exe') -PathType Leaf)) {
        throw "Inno Setup konnte nicht eingerichtet werden (Exit $($process.ExitCode))."
    }
}
