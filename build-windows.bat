@echo off
echo Building PDF Merger for Windows...
cargo build --release
echo.
echo Build complete!
echo.
echo The executable can be found at: target\release\pdf-merger.exe
echo.
pause