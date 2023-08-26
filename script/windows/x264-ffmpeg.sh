cd D:/a/Pianorium/Pianorium

cd deps/source
echo Downloading x264... `date +"%Hh %Mm %Ss"`
git clone https://code.videolan.org/videolan/x264.git
cd x264
curl "http://git.savannah.gnu.org/gitweb/?p=config.git;a=blob_plain;f=config.guess;hb=HEAD" > config.guess
echo Configuring x264... `date +"%Hh %Mm %Ss"`
sed -i 's/host_os = mingw/host_os = msys/' configure
cd ../../build/x264
CC=cl ../../source/x264/configure --prefix=../../installed --enable-static
echo Making x264... `date +"%Hh %Mm %Ss"`
make -j 2
echo Installing x264... `date +"%Hh %Mm %Ss"`
make install

cd ../../source
echo Downloading FFmpeg... `date +"%Hh %Mm %Ss"`
git clone https://git.ffmpeg.org/ffmpeg.git ffmpeg
cd ../build/ffmpeg
echo Configuring FFmpeg... `date +"%Hh %Mm %Ss"`
CC=cl PKG_CONFIG_PATH=../../installed/lib/pkgconfig ../../source/ffmpeg/configure --prefix=../../installed --toolchain=msvc --target-os=win64 --arch=x86_64 --enable-yasm --enable-asm --disable-shared --enable-static --enable-libx264 --enable-gpl --enable-version3 --disable-debug --enable-optimizations --extra-ldflags="-LIBPATH:../../installed/lib" --extra-cxxflags="-I../../installed/include/ -MTd -Od -Zi" --extra-cflags="-I../../installed/include/ -MTd -Od -Zi"
echo Making FFmpeg... `date +"%Hh %Mm %Ss"`
make -j 2
echo Installing FFmpeg... `date +"%Hh %Mm %Ss"`
make install