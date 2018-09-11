#!/bin/sh
set -e

if ! yay -Sy --noconfirm --noremovemake mingw-w64-gdk-pixbuf2; then
	git clone https://aur.archlinux.org/mingw-w64-gdk-pixbuf2
	cd mingw-w64-gdk-pixbuf2
	git checkout f3e92f5c0f60d696c3e291c583cc1cfd8211ca48
	makepkg --noconfirm -i
	cd ..
	rm -rf mingw-w64-gdk-pixbuf2
fi

# self delete
rm -- "$0"
