
set -e
MATRIX_OS=${1}
ZIG_VER=0.9.1
ARCH=x86_64
echo "installing zig matrix.os=$MATRIX_OS version=$ZIG_VER"

if [[ "$MATRIX_OS" == "ubuntu-latest" ]] || [[ "$MATRIX_OS" == "ubuntu-20.04" ]]; then
    echo "installing zig on ubuntu"
    # sudo snap install --beta --classic zig && \

    wget https://ziglang.org/download/$ZIG_VER/zig-linux-$ARCH-$ZIG_VER.tar.xz && \
    tar -xf zig-linux-$ARCH-$ZIG_VER.tar.xz && \
    sudo mv zig-linux-$ARCH-$ZIG_VER /usr/local && \
    pushd /usr/local/bin && \
    sudo ln -s ../zig-linux-$ARCH-$ZIG_VER/zig . && \
    popd && \
    rm zig-linux-$ARCH-0.9.1.tar.* && \

    sudo apt-get install lld-12 && \
    echo "FLUVIO_BUILD_LLD=lld-12" | tee -a $GITHUB_ENV
fi

if [[ "$MATRIX_OS" == "macos-latest" ]] || [[ "$MATRIX_OS" == "macos-11" ]] || [[ "$MATRIX_OS" == "macos-12" ]]; then
    echo "installing zig on mac"
    LLVM_VER=14
    brew update
    brew install zig && \
    brew install llvm@${LLVM_VER} && \
    echo "FLUVIO_BUILD_LLD=/usr/local/opt/llvm@${LLVM_VER}/bin/lld" | tee -a $GITHUB_ENV
fi
