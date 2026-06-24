#!/usr/bin/env sh
set -eu

repo="${SMOLFS_REPO:-CelestoAI/smolfs}"
version="${SMOLFS_VERSION:-latest}"
install_dir="${SMOLFS_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s)"
arch="$(uname -m)"

case "$os:$arch" in
  Linux:x86_64|Linux:amd64)
    target="x86_64-unknown-linux-gnu"
    ;;
  Darwin:arm64|Darwin:aarch64)
    target="aarch64-apple-darwin"
    ;;
  Darwin:x86_64|Darwin:amd64)
    target="x86_64-apple-darwin"
    ;;
  *)
    echo "smolfs: unsupported platform $os/$arch" >&2
    exit 1
    ;;
esac

asset="smolfs-$target.tar.gz"
if [ "$version" = "latest" ]; then
  url="https://github.com/$repo/releases/latest/download/$asset"
else
  url="https://github.com/$repo/releases/download/$version/$asset"
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT INT TERM

mkdir -p "$install_dir"
curl -fsSL "$url" -o "$tmpdir/$asset"
tar -xzf "$tmpdir/$asset" -C "$tmpdir"
install -m 0755 "$tmpdir/smolfs" "$install_dir/smolfs"

echo "Installed smolfs to $install_dir/smolfs"
case ":$PATH:" in
  *":$install_dir:"*) ;;
  *) echo "Add $install_dir to PATH to run smolfs from any shell." ;;
esac
