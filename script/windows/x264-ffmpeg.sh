cd D:/a/Pianorium/Pianorium

git clone https://code.videolan.org/videolan/x264.git deps/source/x264
curl "http://git.savannah.gnu.org/gitweb/?p=config.git;a=blob_plain;f=config.guess;hb=HEAD" > deps/source/x264/config.guess
sed -i 's/host_os = mingw/host_os = msys/' deps/source/x264/configure
CC=cl deps/source/x264/configure --prefix=deps/installed --enable-static
make -j 2
make install

git clone https://git.ffmpeg.org/ffmpeg.git deps/source/ffmpeg
CC=cl PKG_CONFIG_PATH=deps/installed/lib/pkgconfig deps/source/configure --prefix=deps/installed --toolchain=msvc --target-os=win64 --arch=x86_64 --enable-yasm --enable-asm --disable-shared --enable-static --enable-libx264 --enable-libx265 --enable-gpl --enable-version3 --disable-debug --enable-optimizations --extra-ldflags="-LIBPATH:deps/installed/lib" --extra-cxxflags="-Ideps/installed/include/ -MTd -Od -Zi" --extra-cflags="-Ideps/installed/include/ -MTd -Od -Zi"
make -j 2
make install