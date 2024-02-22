@echo off

set ANDROID_API_LEVEL=22
set NDK_DIR=C:\Users\Administrator\AppData\Local\Android\Sdk\ndk\20.1.5948944

set TOOL_CHAINS=%NDK_DIR%\toolchains\llvm\prebuilt\windows-x86_64\bin\

set CUR_DIR=%~dp0



cbindgen --config cbindgen.toml --crate rust_net --lang c++ --output rust_net.h

rustup target add x86_64-pc-windows-msvc i686-pc-windows-msvc aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android

rmdir /S /Q .\output



mkdir .\output\windows

rem windows x64
cargo build --target=x86_64-pc-windows-msvc --release
mkdir .\output\windows\x64\
copy .\target\x86_64-pc-windows-msvc\release\rust_net.lib .\output\windows\x64\rust_net.lib


rem windows x86
cargo build --target=i686-pc-windows-msvc --release
mkdir .\output\windows\x86\
copy .\target\i686-pc-windows-msvc\release\rust_net.lib .\output\windows\x86\rust_net.lib





rem set OPENSSL_LIB_DIR=%CUR_DIR%libs\openssl\arm64-v8a
rem set OPENSSL_DIR=D:\Tools\vcpkg\installed\x64-windows-static
rem set OPENSSL_STATIC=Yes


mkdir .\output\android\

rem arm64-v8a
set TARGET_AR=%TOOL_CHAINS%\aarch64-linux-android-ar
set TARGET_CC=%TOOL_CHAINS%\aarch64-linux-android%ANDROID_API_LEVEL%-clang
cargo build --target aarch64-linux-android --release

mkdir .\output\android\arm64-v8a\
copy .\target\aarch64-linux-android\release\librust_net.a .\output\android\arm64-v8a\librust_net.a


rem rem armeabi-v7a
set TARGET_AR=%TOOL_CHAINS%\arm-linux-androideabi-ar
set TARGET_CC=%TOOL_CHAINS%\armv7a-linux-androideabi%ANDROID_API_LEVEL%-clang
cargo build --target armv7-linux-androideabi --release

mkdir .\output\android\armeabi-v7a\
copy .\target\armv7-linux-androideabi\release\librust_net.a .\output\android\armeabi-v7a\librust_net.a


rem x86
set TARGET_AR=%TOOL_CHAINS%\i686-linux-android-ar
set TARGET_CC=%TOOL_CHAINS%\i686-linux-android%ANDROID_API_LEVEL%-clang
cargo build --target i686-linux-android --release

mkdir .\output\android\x86\
copy .\target\i686-linux-android\release\librust_net.a .\output\android\x86\librust_net.a


rem rem x86_64
set TARGET_AR=%TOOL_CHAINS%\x86_64-linux-android-ar
set TARGET_CC=%TOOL_CHAINS%\x86_64-linux-android%ANDROID_API_LEVEL%-clang
cargo build --target x86_64-linux-android --release

mkdir output\android\x86_64\
copy .\target\x86_64-linux-android\release\librust_net.a .\output\android\x86_64\librust_net.a