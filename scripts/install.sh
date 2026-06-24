#!/usr/bin/env sh
set -eu

repo="${SMOLFS_REPO:-CelestoAI/smolfs}"
version="${SMOLFS_VERSION:-latest}"
install_dir="${SMOLFS_INSTALL_DIR:-$HOME/.local/bin}"
install_cli="${SMOLFS_INSTALL_CLI:-1}"
install_backend="${SMOLFS_INSTALL_BACKEND:-$install_cli}"
install_python="${SMOLFS_INSTALL_PYTHON:-0}"
python_mode="${SMOLFS_PYTHON_MODE:-auto}"
python_package="${SMOLFS_PYTHON_PACKAGE:-smolfs}"

case "$version:$python_package" in
  v*:smolfs)
    python_package="smolfs==${version#v}"
    ;;
esac

is_enabled() {
  case "$1" in
    1|true|TRUE|yes|YES|on|ON)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

install_cli_binary() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Linux:x86_64|Linux:amd64)
      target="x86_64-unknown-linux-gnu"
      ;;
    Linux:arm64|Linux:aarch64)
      target="aarch64-unknown-linux-gnu"
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
    url="https://github.com/$repo/releases/latest/download/$asset?download=1"
  else
    url="https://github.com/$repo/releases/download/$version/$asset?download=1"
  fi

  mkdir -p "$install_dir"
  if ! curl -fsL "$url" -o "$tmpdir/$asset"; then
    echo "smolfs: could not download $asset from GitHub Releases." >&2
    echo "No CLI release asset may exist yet for version '$version' and target '$target'." >&2
    if [ "$version" = "dev" ]; then
      echo "The dev channel is rebuilt from pushes to main." >&2
    else
      echo "Stable release assets are created by the Publish CLI workflow when a v* tag is pushed." >&2
    fi
    echo "From a source checkout, use: cargo build -p smolfs-cli && ./target/debug/smolfs --help" >&2
    exit 1
  fi
  tar -xzf "$tmpdir/$asset" -C "$tmpdir"
  install -m 0755 "$tmpdir/smolfs" "$install_dir/smolfs"

  echo "Installed smolfs CLI to $install_dir/smolfs"
  case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "Add $install_dir to PATH to run smolfs from any shell." ;;
  esac
}

install_python_package() {
  if ! command -v uv >/dev/null 2>&1; then
    echo "smolfs: uv is required to install the Python SDK." >&2
    echo "Install uv, then run: uv add $python_package" >&2
    exit 1
  fi

  case "$python_mode" in
    auto)
      if [ -f pyproject.toml ]; then
        uv add "$python_package"
      elif [ -n "${VIRTUAL_ENV:-}" ]; then
        uv pip install "$python_package"
      else
        echo "smolfs: Python SDK install needs a project or virtualenv." >&2
        echo "Run from a directory with pyproject.toml, activate a virtualenv, or set SMOLFS_PYTHON_MODE=user." >&2
        exit 1
      fi
      ;;
    project)
      uv add "$python_package"
      ;;
    venv)
      if [ -z "${VIRTUAL_ENV:-}" ]; then
        echo "smolfs: SMOLFS_PYTHON_MODE=venv requires an active virtualenv." >&2
        exit 1
      fi
      uv pip install "$python_package"
      ;;
    user)
      uv pip install --user "$python_package"
      ;;
    *)
      echo "smolfs: SMOLFS_PYTHON_MODE must be auto, project, venv, or user." >&2
      exit 1
      ;;
  esac

  echo "Installed smolfs Python SDK ($python_package)"
}

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT INT TERM

if is_enabled "$install_cli"; then
  install_cli_binary
fi

if is_enabled "$install_backend"; then
  if [ ! -x "$install_dir/smolfs" ]; then
    echo "smolfs: cannot install storage backend because the SmolFS CLI was not installed." >&2
    echo "Set SMOLFS_INSTALL_CLI=1 or run: smolfs doctor --install" >&2
    exit 1
  fi
  "$install_dir/smolfs" doctor --install
fi

if is_enabled "$install_python"; then
  install_python_package
fi
