#!/bin/bash

cd /d/a/Pianorium/Pianorium

curl -LO "https://github.com/libsdl-org/SDL/releases/download/release-2.28.2/SDL2-devel-2.28.2-mingw.tar.gz"
tar -xzvf SDL2-devel-2.28.2-mingw.tar.gz
cp -r SDL2-2.28.2/x86_64-w64-mingw32/lib/* ~/.rustup/toolchains/stable-x86_64-pc-windows-msvc/lib/rustlib/x86_64-pc-windows-msvc/lib