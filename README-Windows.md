# Building PDF Merger on Windows

## Prerequisites

1. Install Rust from [https://www.rust-lang.org/](https://www.rust-lang.org/)
2. Install Git from [https://git-scm.com/](https://git-scm.com/) (optional, for cloning the repository)

## Building on Windows

1. Open a command prompt or PowerShell window
2. Navigate to the project directory
3. Run the build script:
   ```
   build-windows.bat
   ```

   Or build manually:
   ```
   cargo build --release
   ```

## Output

The executable will be created at:
```
target\release\pdf-merger.exe
```

## Running the Application

Double-click the `pdf-merger.exe` file to run the application, or run it from the command line:
```
target\release\pdf-merger.exe
```