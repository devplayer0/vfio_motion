#!/bin/bash

PKG_CONFIG_PATH=/usr/x86_64-w64-mingw32/lib/pkgconfig exec cargo build --target=x86_64-pc-windows-gnu "$@"
