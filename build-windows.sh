#!/usr/bin/env bash
export GTK_INSTALL_PATH=/opt/gtkwin
export RELEASE=release-windows
export PKG_CONFIG_ALLOW_CROSS=1 
export PKG_CONFIG_PATH=/opt/gtkwin/lib/pkgconfig 

cargo build --target=x86_64-pc-windows-gnu --release
mkdir ${RELEASE}
cp target/x86_64-pc-windows-gnu/release/*.exe ${RELEASE}
cp ${GTK_INSTALL_PATH}/bin/*.dll ${RELEASE}
mkdir -p ${RELEASE}/share/glib-2.0/schemas
mkdir ${RELEASE}/share/icons
cp ${GTK_INSTALL_PATH}/share/glib-2.0/schemas/* ${RELEASE}/share/glib-2.0/schemas
cp -r ${GTK_INSTALL_PATH}/share/icons/* ${RELEASE}/share/icons
rm packt.zip
zip -r packt.zip ${RELEASE}
