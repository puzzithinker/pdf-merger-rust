# Build PDF Merger for Windows
Write-Host "Building PDF Merger for Windows..." -ForegroundColor Green
cargo build --release
Write-Host "Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "The executable can be found at: target\release\pdf-merger.exe" -ForegroundColor Yellow
Write-Host ""
Write-Host "Press any key to continue..."
$Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")