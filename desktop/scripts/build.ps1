param([switch]$SkipCompile,[string]$InnoCompiler,[string]$ExpectedTag)
$ErrorActionPreference='Stop'
$projectRoot=(Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$desktop=Join-Path $projectRoot 'desktop'
Push-Location $projectRoot
try {
    $packageInfo = cargo metadata --locked --no-deps --manifest-path desktop/Cargo.toml --format-version 1
    if($LASTEXITCODE -ne 0){throw 'App-Version konnte nicht gelesen werden'}
    $version = ($packageInfo | ConvertFrom-Json).packages[0].version
    if($version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+$'){throw "Ungültige Release-Version: $version"}
    if($ExpectedTag -and $ExpectedTag -cne "v$version"){throw "Tag $ExpectedTag passt nicht zu App-Version $version"}
    & (Join-Path $PSScriptRoot 'prepare-release.ps1') -SkipInno:([bool]$InnoCompiler)

    if(-not $SkipCompile){
        cargo test --locked --manifest-path desktop/Cargo.toml --lib
        if($LASTEXITCODE -ne 0){throw 'Rust-Tests fehlgeschlagen'}
        cargo build --locked --release --manifest-path desktop/Cargo.toml --bins
        if($LASTEXITCODE -ne 0){throw 'Rust-Releasebuild fehlgeschlagen'}
    }
    $stamp=Get-Date -Format 'yyyyMMdd-HHmmss'
    $out=Join-Path $projectRoot "builds\KD-Desktop-$version-$stamp"
    if(Test-Path -LiteralPath $out){throw 'Buildordner existiert bereits'}
    $stage=Join-Path $out 'app'
    New-Item -ItemType Directory -Path (Join-Path $stage 'runtime\cpu'),(Join-Path $stage 'licenses') -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $desktop 'target\release\kd-desktop.exe') -Destination (Join-Path $stage 'KD.exe')
    Copy-Item -LiteralPath (Join-Path $desktop 'target\release\kd-tool.exe') -Destination (Join-Path $out 'kd-tool.exe')
    $runtime=Join-Path $projectRoot '.tools\llama-b12125-cpu'
    Get-ChildItem -LiteralPath $runtime -File | Where-Object {$_.Extension -eq '.dll' -or $_.Name -eq 'llama-server.exe'} | Copy-Item -Destination (Join-Path $stage 'runtime\cpu')
    # The pinned server executable dispatches into llama-server-impl.dll.
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    $crt = & $vswhere -all -products '*' -property installationPath | ForEach-Object {
        Get-ChildItem -Path (Join-Path $_ 'VC/Redist/MSVC/*/x64/Microsoft.VC*.CRT') -Directory
    } | Sort-Object { [version]$_.Parent.Parent.Name } -Descending | Select-Object -First 1
    if(-not $crt){throw 'Visual Studio C++-Redistributables für x64 fehlen'}
    $crtFiles = Get-ChildItem -LiteralPath $crt.FullName -Filter '*.dll' -File
    $crtFiles | Copy-Item -Destination (Join-Path $stage 'runtime\cpu')
    $crtFiles | Copy-Item -Destination $stage
    Copy-Item -LiteralPath (Join-Path $desktop 'LICENSE.txt'),(Join-Path $desktop 'INSTALLATION.txt'),(Join-Path $desktop 'README.md') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $runtime 'llama.cpp-MIT.txt'),(Join-Path $runtime 'LICENSE-LLVM-OpenMP') -Destination (Join-Path $stage 'licenses')
    $wv=Join-Path $projectRoot '.tools\MicrosoftEdgeWebView2RuntimeInstallerX64.exe'
    $signature = Get-AuthenticodeSignature -LiteralPath $wv
    if($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch '^CN=Microsoft Corporation,'){throw 'Ungültige Microsoft-Signatur für den WebView2-Installer'}
    Copy-Item -LiteralPath $wv -Destination $stage
    $metadata=cargo metadata --locked --filter-platform x86_64-pc-windows-msvc --manifest-path desktop/Cargo.toml --format-version 1
    if($LASTEXITCODE -ne 0){throw 'Abhängigkeitsmetadaten konnten nicht gelesen werden'}
    $metadata = $metadata | ConvertFrom-Json
    $notices=foreach($p in $metadata.packages){
        $licenseDir=Join-Path $stage ('licenses\'+$p.name+'-'+$p.version)
        $sourceDir=Split-Path $p.manifest_path
        $licenseFiles=Get-ChildItem -LiteralPath $sourceDir -File | Where-Object {$_.Name -match '^(LICENSE|COPYING|NOTICE|UNLICENSE)'}
        if($licenseFiles){New-Item -ItemType Directory -Path $licenseDir -Force|Out-Null;$licenseFiles | Copy-Item -Destination $licenseDir}
        [pscustomobject]@{name=$p.name;version=$p.version;license=$p.license;repository=$p.repository}
    }
    $notices | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $stage 'licenses\rust-dependencies.json') -Encoding utf8
    if(Get-ChildItem -LiteralPath $stage -Recurse -File | Where-Object {$_.Extension -in @('.gguf','.safetensors','.bin')}){throw 'Installer-Staging enthält unzulässige Modell-/Binärdaten'}
    if(-not $InnoCompiler){$InnoCompiler=Join-Path $projectRoot '.tools\inno\compiler\ISCC.exe'}
    # A short, fresh output path avoids Windows resource-update contention on
    # deep project paths. Keep failed outputs as evidence, just like old builds.
    $compilerOutput=Join-Path $env:TEMP ('kd-installer-'+$stamp)
    New-Item -ItemType Directory -Path $compilerOutput | Out-Null
    & $InnoCompiler '/Qp' "/DStageDir=$stage" "/DOutputDir=$compilerOutput" "/DAppVersion=$version" (Join-Path $desktop 'installer\kd.iss')
    if($LASTEXITCODE -ne 0){throw 'Installer-Build fehlgeschlagen'}
    Copy-Item -LiteralPath (Join-Path $compilerOutput "KD-Desktop-$version-Setup.exe") -Destination $out
    Get-ChildItem -LiteralPath $out -File -Recurse | ForEach-Object {
        [pscustomobject]@{file=$_.FullName.Substring($out.Length+1);bytes=$_.Length;sha256=(Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash}
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $out 'SHA256.json') -Encoding utf8
    $out | Set-Content -LiteralPath (Join-Path $desktop 'last-build.txt') -Encoding utf8
    Write-Output "Fertiger Build: $out"
}finally{Pop-Location}
