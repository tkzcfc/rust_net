



## 生成.h头文件

```


    安装cbindgen
    cargo install cbindgen

    生成.h头文件
    cbindgen --config cbindgen.toml --crate rust_net --lang c++ --output rust_net.h


```





## 编译静态库





### Windows 编译静态库

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
        
windows:
	cargo build --release
	cargo build --target=x86_64-pc-windows-msvc --release
    cargo build --target=i686-pc-windows-msvc --release


```



### Android编译

1.添加Rust工具链

```
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

2.开始编译

```




设置交叉编译环境
	linux:
 		export TARGET_AR=~/.NDK/arm/bin/arm-linux-androideabi-ar
 		export TARGET_CC=~/.NDK/arm/bin/arm-linux-androideabi-clang
 	windows:
 		set TARGET_AR C:\Users\Administrator\AppData\Local\Android\Sdk\ndk\24.0.8215888\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android-ar
 		set TARGET_AR C:\Users\Administrator\AppData\Local\Android\Sdk\ndk\24.0.8215888\toolchains\llvm\prebuilt\windows-x86_64\bin\aarch64-linux-android21-clang
 编译：
     cargo build --target armv7-linux-androideabi --release
 


3. 编译静态库
# 为arm64-v8a架构编译
cargo build --target aarch64-linux-android --release
# 为armeabi-v7a架构编译
cargo build --target armv7-linux-androideabi --release
# 为x86架构编译
cargo build --target i686-linux-android --release
# 为x86_64架构编译
cargo build --target x86_64-linux-android --release



```

