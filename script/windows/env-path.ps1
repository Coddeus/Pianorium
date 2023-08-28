cd D:\a\Pianorium\Pianorium

mkdir -p deps\source\ffmpeg ; mkdir -p deps\build\x264 ; mkdir -p deps\build\ffmpeg ; mkdir -p deps\installed

$VCINSTALLDIR = $(& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -property installationPath)
Add-Content $env:GITHUB_ENV "LIBCLANG_PATH=${VCINSTALLDIR}\VC\Tools\LLVM\x64\bin`n"
Add-Content $env:GITHUB_ENV "FFMPEG_DIR=${pwd}\deps\installed`n"
Add-Content $env:GITHUB_ENV "PKG_CONFIG_PATH=${pwd}\deps\installed\lib\pkgconfig`n"
Add-Content $env:GITHUB_ENV "CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true`n"
Add-Content $env:GITHUB_ENV "RUST_BACKTRACE=1`n"
Add-Content $env:GITHUB_PATH "D:\a\_temp\msys64`n"
Add-Content $env:GITHUB_PATH "D:\a\_temp\msys64\usr\bin`n"
Add-Content $env:GITHUB_PATH "D:\a\_temp\msys64\mingw64\bin`n"
Add-Content $env:GITHUB_PATH "D:\a\_temp\msys64\mingw64\x86_64-w64-mingw32\bin`n"