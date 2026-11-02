param(
    [ValidateRange(1024, 65535)]
    [int]$Port = 3000,
    [string]$DataDirectory = 'data/projects'
)

$ErrorActionPreference = 'Stop'
$workspace = Split-Path -Parent $PSScriptRoot
Push-Location $workspace
try {
    if (-not $env:CARGO_TARGET_DIR) {
        $env:CARGO_TARGET_DIR = Join-Path $workspace '.runtime/local-target'
    }
    $env:AI_PCB_ADDR = "127.0.0.1:$Port"
    $env:AI_PCB_DATA_DIR = $DataDirectory
    $bundledCli = Join-Path $workspace '.runtime/eda/kicad/bin/kicad-cli.exe'
    $bundledPython = Join-Path $workspace '.runtime/eda/kicad/bin/python.exe'
    if (-not $env:AI_PCB_KICAD_CLI -and (Test-Path -LiteralPath $bundledCli -PathType Leaf)) {
        $env:AI_PCB_KICAD_CLI = $bundledCli
    }
    if (-not $env:AI_PCB_KICAD_PYTHON -and (Test-Path -LiteralPath $bundledPython -PathType Leaf)) {
        $env:AI_PCB_KICAD_PYTHON = $bundledPython
    }
    & cargo run --locked
    if ($LASTEXITCODE -ne 0) {
        throw "AI-PCB exited with code $LASTEXITCODE. Check the port and data-directory lock."
    }
} finally {
    Pop-Location
}
