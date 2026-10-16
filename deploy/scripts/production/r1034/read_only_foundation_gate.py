#!/usr/bin/env python3
"""Read-only exact-source Ubuntu26 commercial foundation readiness.

Never promotes a nonroot check to recovery/root/PITR/production GO.
Does not read credentials, customer configuration, or root-owned attestations.
"""
from __future__ import annotations
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import pwd
import re
import stat
import subprocess
import sys

SHA40=re.compile(r"^[0-9a-f]{40}$")
SHA256=re.compile(r"^[0-9a-f]{64}$")
MANAGED_PORTS=(80,443,5432,3003,3005,3006)
USERS=("ipatpapi","ipatpoidc","ipattapi","ipattoidc","ipatdverify","ipatdnsverify","ipatpgmigrate")


def command(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args,text=True,capture_output=True,timeout=12,check=False)


def file_digest(path:Path) -> str:
    h=hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda:f.read(1024*1024),b""):
            h.update(block)
    return h.hexdigest()


def source_check(root:Path, expected:str) -> None:
    if not root.is_absolute() or root.is_symlink() or not root.is_dir():
        raise ValueError("reviewed absolute source checkout required")
    head=command("git","-C",str(root),"rev-parse","HEAD")
    clean=command("git","-C",str(root),"status","--porcelain","--untracked-files=all")
    if head.returncode or head.stdout.strip()!=expected or clean.returncode or clean.stdout.strip():
        raise ValueError("source must match exact clean reviewed commit")


def candidate_check(directory:Path,expected_commit:str,expected_digest:str) -> None:
    if not directory.is_absolute() or directory.is_symlink() or not directory.is_dir():
        raise ValueError("absolute non-symlink candidate required")
    binary=directory/"control-api"
    source=directory/"SOURCE_COMMIT"
    manifest=directory/"SHA256SUMS"
    for file in (binary,source,manifest):
        if file.is_symlink() or not file.is_file() or file.stat().st_nlink!=1:
            raise ValueError("unsafe candidate inode")
    if stat.S_IMODE(binary.stat().st_mode)!=0o500:
        raise ValueError("release binary permission drift")
    if source.read_text().strip()!=expected_commit:
        raise ValueError("candidate source commit drift")
    if file_digest(binary)!=expected_digest:
        raise ValueError("release binary checksum drift")
    approved={}
    for line in manifest.read_text().splitlines():
        found=re.fullmatch(r"([0-9a-f]{64})  (control-api|SOURCE_COMMIT)",line)
        if not found or found.group(2) in approved:
            raise ValueError("invalid or duplicate release manifest")
        approved[found.group(2)]=found.group(1)
    if set(approved)!={"control-api","SOURCE_COMMIT"}:
        raise ValueError("incomplete release manifest")
    if any(file_digest(directory/name)!=digest for name,digest in approved.items()):
        raise ValueError("release manifest checksum mismatch")


def inspected_port_state(port:int) -> str:
    # Ask ss for the numeric socket port rather than parsing textual address
    # suffixes. Inspection errors must never masquerade as absence.
    checked=command("ss","-H","-ltn",f"( sport = :{port} )")
    if checked.returncode:
        raise ValueError(f"unable to inspect managed port {port}")
    return "LISTENING" if checked.stdout.strip() else "NOT_LISTENING"


def inspect_runtime() -> dict:
    if os.geteuid()==0:
        raise ValueError("nonroot-only: use separate independent privileged recovery review")
    os_release={}
    for line in Path("/etc/os-release").read_text().splitlines():
        if "=" in line:
            k,v=line.split("=",1)
            os_release[k]=v.strip('"')
    if (os_release.get("ID"),os_release.get("VERSION_ID"))!=("ubuntu","26.04"):
        raise ValueError("Ubuntu Server 26.04 is required")
    result={"os":"ubuntu26.04","architecture":platform.machine()}
    if platform.machine() not in ("x86_64","amd64"):
        raise ValueError("built candidate architecture requires independent review")
    for name in USERS:
        try:pwd.getpwnam(name); state="PRESENT"
        except KeyError:state="ABSENT"
        result["identity_"+name]=state
    for port in MANAGED_PORTS:
        result["tcp_port_"+str(port)]=inspected_port_state(port)
    pg=command("pg_lsclusters","--no-header") if Path("/usr/bin/pg_lsclusters").exists() else None
    if pg is None:result["postgres_clusters"]="TOOLS_NOT_INSTALLED"
    elif pg.returncode:result["postgres_clusters"]="UNKNOWN_INSPECTION_FAILED"
    else:result["postgres_clusters"]="PRESENT" if pg.stdout.strip() else "NONE"
    installed=command("apt-cache","policy","postgresql-18")
    m=re.search(r"(?m)^\s*Candidate:\s*(\S+)",installed.stdout)
    result["postgresql18_candidate_available"]=bool(installed.returncode==0 and m and m.group(1).startswith("18."))
    result["root_recovery_attestation"]="NOT_ASSESSED_REQUIRES_PRIVILEGED_REVIEW"
    result["independent_full_host_restore"]="NOT_ASSESSED_REQUIRES_PRIVILEGED_REVIEW"
    result["production_pitr_ha"]="NOT_VERIFIED"
    result["human_mfa_tls_tenant"]="NOT_VERIFIED"
    result["public_go"]=False
    return result


def main() -> int:
    p=argparse.ArgumentParser()
    p.add_argument("--source",required=True,type=Path)
    p.add_argument("--candidate",required=True,type=Path)
    p.add_argument("--commit",required=True)
    p.add_argument("--binary-sha256",required=True)
    args=p.parse_args()
    try:
        if not SHA40.fullmatch(args.commit) or not SHA256.fullmatch(args.binary_sha256):
            raise ValueError("exact commit and binary digest required")
        source_check(args.source,args.commit)
        candidate_check(args.candidate,args.commit,args.binary_sha256)
        result=inspect_runtime()
        result.update({"schema":"ipat.r1034.nonroot-preflight.v1",
                       "source_commit":args.commit,
                       "source_and_binary_hash_verified":True,
                       "candidate_is_root_owned":False})
        print(json.dumps(result,sort_keys=True))
        return 0
    except (OSError,ValueError,subprocess.TimeoutExpired) as error:
        print(json.dumps({"ok":False,"reason":str(error),"public_go":False},sort_keys=True))
        return 4


if __name__=="__main__":
    sys.exit(main())
