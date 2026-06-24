from __future__ import annotations

import argparse
import json
import sys
from importlib.metadata import PackageNotFoundError, version
from typing import Any, Sequence

from . import SmolFS, SmolFSError, doctor
from ._native import install_managed_juicefs


def main(argv: Sequence[str] | None = None) -> int:
    parser = _build_parser()
    args = parser.parse_args(argv)

    try:
        if args.command == "doctor":
            return _doctor(args)

        fs = SmolFS.from_env()
        if args.command == "init":
            return _init(fs, args)
        if args.command == "mount":
            return _mount(fs, args)
        if args.command == "status":
            return _status(fs, args)
        if args.command == "flush":
            return _flush(fs, args)
        if args.command in {"unmount", "umount"}:
            return _unmount(fs, args)
    except SmolFSError as error:
        print(f"smolfs: {error}", file=sys.stderr)
        return 1

    parser.error(f"unknown command {args.command!r}")
    return 2


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="smolfs",
        description="Durable developer volumes for agents",
    )
    parser.add_argument("--version", action="version", version=f"smolfs {_package_version()}")
    subparsers = parser.add_subparsers(dest="command", required=True)

    doctor_parser = subparsers.add_parser(
        "doctor", help="Check JuiceFS, FUSE, and local SmolFS setup"
    )
    doctor_parser.add_argument(
        "--install",
        action="store_true",
        help="Copy a discovered JuiceFS binary into SmolFS' managed bin directory",
    )
    doctor_parser.add_argument(
        "--json", action="store_true", help="Print a machine-readable setup report"
    )

    init_parser = subparsers.add_parser("init", help="Create a named SmolFS volume")
    init_parser.add_argument("name", help="Volume name, using letters, numbers, '.', '_' or '-'")
    init_parser.add_argument(
        "--dev", action="store_true", help="Create a local JuiceFS volume for development"
    )
    init_parser.add_argument(
        "--metadata", help="JuiceFS metadata URL, such as redis://localhost:6379/1"
    )
    init_parser.add_argument(
        "--store", help="Object store URL, such as s3://bucket/prefix or file:///tmp/objects"
    )
    init_parser.add_argument(
        "--storage", help="JuiceFS storage type escape hatch, such as s3, gs, or file"
    )
    init_parser.add_argument("--bucket", help="JuiceFS bucket/endpoint used with --storage")

    mount_parser = subparsers.add_parser("mount", help="Mount a SmolFS volume at a local path")
    mount_parser.add_argument("name", help="Existing SmolFS volume name")
    mount_parser.add_argument("path", help="Local directory where the volume should be mounted")
    mount_parser.add_argument(
        "--foreground",
        action="store_true",
        help="Run JuiceFS in the foreground instead of background mode",
    )
    mount_parser.add_argument(
        "--check-storage",
        action="store_true",
        help="Ask JuiceFS to test object storage access before mounting",
    )

    status_parser = subparsers.add_parser("status", help="Show configured SmolFS volumes")
    status_parser.add_argument("name", nargs="?", help="Optional volume name to inspect")
    status_parser.add_argument(
        "--json", action="store_true", help="Print machine-readable status"
    )

    flush_parser = subparsers.add_parser("flush", help="Best-effort flush check for a mounted volume")
    flush_parser.add_argument("name", help="Mounted SmolFS volume name")

    unmount_parser = subparsers.add_parser(
        "unmount", help="Unmount a SmolFS volume and wait for JuiceFS flush"
    )
    unmount_parser.add_argument("name", help="Mounted SmolFS volume name")
    unmount_parser.add_argument(
        "--force", action="store_true", help="Force unmount a busy mountpoint"
    )

    umount_parser = subparsers.add_parser("umount", help="Alias for `smolfs unmount`")
    umount_parser.add_argument("name", help="Mounted SmolFS volume name")
    umount_parser.add_argument(
        "--force", action="store_true", help="Force unmount a busy mountpoint"
    )

    return parser


def _doctor(args: argparse.Namespace) -> int:
    if args.install:
        path = install_managed_juicefs()
        print(f"Installed managed JuiceFS binary at {path}")

    report = doctor()
    if args.json:
        print(json.dumps(report, indent=2))
        return 0

    print(f"SmolFS home: {report['home']}")
    print(f"Config: {report['config']}")

    juicefs = report["juicefs"]
    if juicefs["found"]:
        managed = " (managed)" if juicefs["managed"] else ""
        print(f"JuiceFS: {juicefs['path'] or '(unknown)'}{managed}")
        if juicefs["version"]:
            print(f"Version: {juicefs['version']}")
    else:
        print("JuiceFS: missing")
        print("Fix: run `smolfs doctor --install` or set SMOLFS_JUICEFS_BIN")

    fuse = report["fuse"]
    if fuse["found"]:
        print(f"FUSE: {fuse['detail']}")
    else:
        print(f"FUSE: missing ({fuse['detail']})")
        if fuse["fix"]:
            print(f"Fix: {fuse['fix']}")

    return 0


def _init(fs: SmolFS, args: argparse.Namespace) -> int:
    volume = fs.init(
        args.name,
        dev=args.dev,
        metadata=args.metadata,
        store=args.store,
        storage=args.storage,
        bucket=args.bucket,
    )
    mode = "dev" if volume.dev else "cloud"
    print(f"Initialized volume {volume.name} ({mode})")
    return 0


def _mount(fs: SmolFS, args: argparse.Namespace) -> int:
    mount = fs.mount(
        args.name,
        args.path,
        foreground=args.foreground,
        check_storage=args.check_storage,
    )
    print(f"Mounted volume {mount.name} at {mount.mountpoint}")
    return 0


def _status(fs: SmolFS, args: argparse.Namespace) -> int:
    status = fs.status(args.name)
    if args.json:
        print(json.dumps(_status_to_dict(status), indent=2))
        return 0

    if not status.volumes:
        print("No volumes")
        return 0

    for volume in status.volumes:
        mountpoint = volume.mountpoint or "-"
        mode = "dev" if volume.dev else "cloud"
        print(f"{volume.name}\t{mode}\t{volume.storage}\t{mountpoint}")

    return 0


def _flush(fs: SmolFS, args: argparse.Namespace) -> int:
    fs.flush(args.name)
    print(f"Flushed volume {args.name}")
    return 0


def _unmount(fs: SmolFS, args: argparse.Namespace) -> int:
    fs.unmount(args.name, force=args.force)
    print(f"Unmounted volume {args.name}")
    return 0


def _status_to_dict(status: Any) -> dict[str, Any]:
    return {"volumes": [_volume_to_dict(volume) for volume in status.volumes]}


def _volume_to_dict(volume: Any) -> dict[str, Any]:
    return {
        "name": volume.name,
        "metadata_url": volume.metadata_url,
        "storage": volume.storage,
        "bucket": volume.bucket,
        "dev": volume.dev,
        "mountpoint": volume.mountpoint,
    }


def _package_version() -> str:
    try:
        return version("smolfs")
    except PackageNotFoundError:
        return "0.1.0"


if __name__ == "__main__":
    raise SystemExit(main())
