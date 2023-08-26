cd D:/a/Pianorium/Pianorium

cd deps/source/x264
git clone https://code.videolan.org/videolan/x264.git
curl "http://git.savannah.gnu.org/gitweb/?p=config.git;a=blob_plain;f=config.guess;hb=HEAD" > config.guess
sed -i 's/host_os = mingw/host_os = msys/' configure
CC=cl configure --prefix=../../installed --enable-static
make -j 2
make install

cd ..
git clone https://git.ffmpeg.org/ffmpeg.git ffmpeg
cd ffmpeg
CC=cl PKG_CONFIG_PATH=../../installed/lib/pkgconfig configure --prefix=../../installed --toolchain=msvc --target-os=win64 --arch=x86_64 --enable-yasm --enable-asm --disable-shared --enable-static --enable-libx264 --enable-gpl --enable-version3 --disable-debug --enable-optimizations --extra-ldflags="-LIBPATH:../../installed/lib" --extra-cxxflags="-I../../installed/include/ -MTd -Od -Zi" --extra-cflags="-I../../installed/include/ -MTd -Od -Zi"
make -j 2
make install