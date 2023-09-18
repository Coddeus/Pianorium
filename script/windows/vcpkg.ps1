cd D:/a/Pianorium/Pianorium

git clone https://github.com/microsoft/vcpkg
cd ./vcpkg
./bootstrap-vcpkg.bat
echo "set(VCPKG_TARGET_ARCHITECTURE x64)
set(VCPKG_CRT_LINKAGE dynamic)
set(VCPKG_LIBRARY_LINKAGE static)
set(VCPKG_BUILD_TYPE release)" > ./triplets/community/x64-windows-static-md.cmake
./vcpkg install --triplet x64-windows-static-md