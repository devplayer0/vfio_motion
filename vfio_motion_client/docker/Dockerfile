FROM devplayer0/winrust:latest

COPY --chown=build:build gdkpixbuf2.sh /tmp/

RUN gpg --recv-keys 9766E084FB0F43D8 && \
	yay -Syu --noconfirm --noremovemake mingw-w64-freetype2-bootstrap mingw-w64-cairo-bootstrap && \
	yay -S --noconfirm --noremovemake mingw-w64-harfbuzz && \
	gpg --recv-keys 38EE757D69184620 && \
	gpg --recv-keys D81C4887F1679A65 && \
	yay -S --noconfirm --noremovemake mingw-w64-pango && \
	/tmp/gdkpixbuf2.sh && \
	yay -S --noconfirm --noremovemake mingw-w64-rust-bin mingw-w64-librsvg && \
	yay -R -dd --noconfirm mingw-w64-cairo-bootstrap && \
	yay -S --noconfirm --noremovemake mingw-w64-gtk3 && \
	yay -R --noconfirm mingw-w64-wine mingw-w64-rust-bin && \
	yay -S --noconfirm mingw-w64-libvirt && \
	yay --noconfirm --clean && paccache --remove --keep 0
