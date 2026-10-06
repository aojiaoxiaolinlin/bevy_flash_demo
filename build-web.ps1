param([switch]$DebugBuild)
$ErrorActionPreference = 'Stop'
Push-Location $PSScriptRoot
try {
    $profile = if ($DebugBuild) { 'debug' } else { 'release' }
    $buildArgs = @('build','--locked','--target','wasm32-unknown-unknown')
    if (!$DebugBuild) { $buildArgs += '--release' }
    & cargo @buildArgs
    if ($LASTEXITCODE -ne 0) { throw 'WASM compilation failed' }
    $bindgenName = if ($IsWindows) { 'wasm-bindgen.exe' } else { 'wasm-bindgen' }
    $localBindgen = Join-Path $PSScriptRoot ".tools/wasm-bindgen/bin/$bindgenName"
    $bindgen = if (Test-Path -LiteralPath $localBindgen) { $localBindgen } else { 'wasm-bindgen' }
    & $bindgen "target/wasm32-unknown-unknown/$profile/bevy_flash_demo.wasm" --target web --out-dir dist/pkg --out-name bevy_flash_demo
    if ($LASTEXITCODE -ne 0) { throw 'wasm-bindgen failed; CLI and crate versions must match' }
    foreach ($file in @('index.html','style.css','app.js')) { Copy-Item -LiteralPath $file -Destination (Join-Path 'dist' $file) -Force }
    # Copy demo assets without deleting any existing output tree.
    foreach ($folder in @('animations', 'ui')) {
        $source = Join-Path 'assets' $folder
        $destination = Join-Path 'dist/assets' $folder
        if ($IsWindows) {
            & robocopy $source $destination /E /NFL /NDL /NJH /NJS /NP
            if ($LASTEXITCODE -gt 7) { throw 'Asset packaging failed' }
        } else {
            New-Item -ItemType Directory -Path $destination -Force | Out-Null
            Get-ChildItem -LiteralPath $source | Copy-Item -Destination $destination -Recurse -Force
        }
    }
    Write-Host "Built dist/ ($profile). Serve it over HTTP on localhost or HTTPS."
} finally { Pop-Location }
