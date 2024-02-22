#! /bin/sh


# 下载 NDK        https://dl.google.com/android/repository/android-ndk-r20b-linux-x86_64.zip
# 下载 OpenSSL    https://github.com/openssl/openssl/releases/download/openssl-3.2.1/openssl-3.2.1.tar.gz 
# 解压之后将本脚本放入解压后目录并执行

export ANDROID_NDK_ROOT=/home/fc/Downloads/android-ndk-r20b
export PATH=$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH


output_dir=/home/fc/openssl

# 编译arm64-v8a架构的静态库
./Configure android-arm64 no-shared -D__ANDROID_API__=22 --prefix=${output_dir}/arm64-v8a --openssldir=${output_dir}/arm64-v8a/ssl
make clean
make -j4
make install_sw

# 编译armeabi-v7a架构的静态库
./Configure android-arm no-shared -D__ANDROID_API__=22 --prefix=${output_dir}/armeabi-v7a --openssldir=${output_dir}/armeabi-v7a/ssl
make clean
make -j4
make install_sw

# 编译x86架构的静态库
./Configure android-x86 no-shared -D__ANDROID_API__=22 --prefix=${output_dir}/x86 --openssldir=${output_dir}/x86/ssl
make clean
make -j4
make install_sw

# 编译x86_64架构的静态库
./Configure android-x86_64 no-shared -D__ANDROID_API__=22 --prefix=${output_dir}/x86_64 --openssldir=${output_dir}/x86_64/ssl
make clean
make -j4
make install_sw