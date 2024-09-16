#!/bin/bash

# Function to detect architecture
detect_arch() {
    case $(uname -m) in
        armv7l)
            echo "armv7"
            ;;
        aarch64)
            echo "aarch64"
            ;;
        x86_64)
            echo "x86_64"
            ;;
        *)
            echo "Unsupported architecture"
            exit 1
            ;;
    esac
}

# Function to download the appropriate binary
download_binary() {
    ARCH=$(detect_arch)
    BASE_URL="https://github.com/bajrangCoder/websockify_rs/releases/download/latest"

    FILE_NAME="websockify_rs-$ARCH"
    DOWNLOAD_URL="$BASE_URL/$FILE_NAME"

    # Download the binary
    echo "Downloading $FILE_NAME for $ARCH architecture..."
    curl -LO $DOWNLOAD_URL

    if [ $? -ne 0 ]; then
        echo "Failed to download the binary!"
        exit 1
    fi

    # Move the binary to a directory in the PATH and make it executable
    mv $FILE_NAME $PREFIX/bin/websockify_rs
    chmod +x $PREFIX/bin/websockify_rs

    echo "Binary downloaded and installed as websockify_rs. You can now use it!"
}

# Run the download function
download_binary
