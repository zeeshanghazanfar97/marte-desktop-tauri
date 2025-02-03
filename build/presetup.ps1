# Check if 'uv' is installed
if (Get-Command "uv" -ErrorAction SilentlyContinue) {
    Write-Host "uv installed already."
} else {
    powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"
}
$uvPath = "$HOME/.local/bin/uv.exe"
if (Test-Path $uvPath) {
    & $uvPath python install 3.11
    Write-Host "Python 3.11 installation initiated using uv."
    # Check if virtual env already exists
    $programFiles = "C:\Program Files (x86)"
    $path_venv = Join-Path -Path $programFiles -ChildPath "Marte/venv"
    if (-Not (Test-Path $path_venv)) {
        & $uvPath venv $path_venv --python 3.11
        Write-Host "Virtual environment created at $path_venv using Python 3.11."
    } else {
        Write-Host "Virtual environment already exists at $path_venv."
    }
    Set-Location $path_venv
    & $uvPath pip install fastapi uvicorn requests beautifulsoup4 pyttsx3 cryptography pyperclip googlesearch-python
    Write-Host "Pip libraries installed with uv."
    exit 0
} else {
    Write-Host "uv executable not found at $uvPath."
    exit 1
}

# $path_root = Join-Path -Path $programFiles -ChildPath "Marte"
# # Rename any file matching "marte*.exe" to "marte.exe" in the path_root directory
# Get-ChildItem -Path $path_root -Filter "marte*.exe" | ForEach-Object {
#     $newName = Join-Path -Path $path_root -ChildPath "marte.exe"
#     Rename-Item -Path $_.FullName -NewName $newName
#     Write-Host "Renamed $($_.Name) to marte.exe"
# }