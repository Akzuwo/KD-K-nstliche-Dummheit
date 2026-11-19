param([ValidateSet(560,1280)][int]$Width=1280,[switch]$SkipCompile,[switch]$Release)
$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
if(-not $SkipCompile){
    $cargoArgs=@('build','--locked','--manifest-path',(Join-Path $root 'desktop\Cargo.toml'),'--features','ui-check','--bin','kd-ui-check')
    if($Release){$cargoArgs+='--release'}
    & cargo @cargoArgs
    if($LASTEXITCODE -ne 0){throw 'UI-Testbuild fehlgeschlagen'}
}
$stamp=Get-Date -Format 'yyyyMMdd-HHmmss-fff'
$profile=if($Release){'release'}else{'debug'}
$data=Join-Path $root "validation-desktop\navigation-$profile-$Width-$stamp"
$previousData=$env:KD_DATA_DIR
$previousWidth=$env:KD_CHECK_WIDTH
try{
    $env:KD_DATA_DIR=$data
    $env:KD_CHECK_WIDTH=$Width.ToString()
    $process=Start-Process -FilePath (Join-Path $root "desktop\target\$profile\kd-ui-check.exe") -PassThru -WindowStyle Hidden
    if(-not $process.WaitForExit(45000)){Stop-Process -Id $process.Id;throw "UI-Test hat kein Ergebnis geliefert: $data"}
    $result=Join-Path $data 'ui-result.json'
    if(-not(Test-Path -LiteralPath $result)){throw "UI-Bericht fehlt: $data"}
    Get-Content -LiteralPath $result
    if($process.ExitCode -ne 0){throw "UI-Test fehlgeschlagen: $result"}
}finally{
    $env:KD_DATA_DIR=$previousData
    $env:KD_CHECK_WIDTH=$previousWidth
}
