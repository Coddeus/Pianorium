cd D:\a\Pianorium\Pianorium

Invoke-WebRequest "https://github.com/libsdl-org/SDL/releases/download/release-2.28.2/SDL2-devel-2.28.2-mingw.zip" -OutFile SDL2-2.28.2.zip
7z x SDL2-2.28.2.zip
Copy-Item -Path "SDL2-2.28.2\x86_64-w64-mingw32\lib\*" -Destination "C:\Users\runneradmin\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\x86_64-pc-windows-msvc\lib" -Recurse

