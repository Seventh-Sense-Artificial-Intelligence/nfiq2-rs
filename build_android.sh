#!/bin/bash
set -e

# Set path to jniLibs target dir
JNILIBS_DIR="bindings/android/app/src/main/jniLibs"

# Android targets you want to build for
TARGETS=(
  "aarch64-linux-android"    # arm64-v8a
  "armv7-linux-androideabi"  # armeabi-v7a
)

# Build each target
for TARGET in "${TARGETS[@]}"; do
  echo "📦 Building for $TARGET..."
  cargo ndk -t "$TARGET" -o ./target/android build --release
done