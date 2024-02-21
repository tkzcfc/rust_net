



### 生成.h头文件

```


    安装cbindgen
    cargo install cbindgen

    生成.h头文件
    cbindgen --config cbindgen.toml --crate rust_net --lang c++ --output rust_net.h


```





### 编译静态库

```text
vc link lib:
    openssl:
        Crypt32.lib
        ws2_32.lib
        Bcrypt.lib
        Userenv.lib
        Ntdll.lib
        Secur32.lib
        Ncrypt.lib
        libdurl.lib
        
    rustls:
        Bcrypt.lib
        ws2_32.lib
        Ntdll.lib
        libdurl.lib

android build example:
 
 export TARGET_AR=~/.NDK/arm/bin/arm-linux-androideabi-ar
 export TARGET_CC=~/.NDK/arm/bin/arm-linux-androideabi-clang
 cargo build --target armv7-linux-androideabi --release
 
 
windows:
	cargo build --release
	cargo build --target=x86_64-pc-windows-msvc --release
    cargo build --target=i686-pc-windows-msvc --release


```
