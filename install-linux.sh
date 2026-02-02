#!/bin/bash
set -e

echo "Installing Poprawiacz Tekstu RS..."

INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp target/release/poprawiacz-tekstu-rs "$INSTALL_DIR/"
echo "✅ Binary installed to $INSTALL_DIR/poprawiacz-tekstu-rs"

ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
mkdir -p "$ICON_DIR"
cp assets/icon_256.png "$ICON_DIR/poprawiacz-tekstu-rs.png"
echo "✅ Icon installed to $ICON_DIR/poprawiacz-tekstu-rs.png"

DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"
cp poprawiacz-tekstu-rs.desktop "$DESKTOP_DIR/"
echo "✅ Desktop entry installed to $DESKTOP_DIR/poprawiacz-tekstu-rs.desktop"

gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true

echo ""
echo "✅ Installation complete!"
echo "You can now find 'Poprawiacz Tekstu' in your application menu."
echo "Or run: poprawiacz-tekstu-rs"
