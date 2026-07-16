#!/usr/bin/env python3
"""Generate a compact CycloneDX SBOM from Cargo metadata."""

import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/"output/release/clio-vqpu-sbom.cdx.json"
metadata=json.loads(subprocess.run(["cargo","metadata","--format-version","1","--locked"],cwd=ROOT,check=True,capture_output=True,text=True).stdout)
components=[]
for package in metadata["packages"]:
    components.append({"type":"library" if package["targets"][0]["kind"][0]=="lib" else "application","name":package["name"],"version":package["version"],"purl":f"pkg:cargo/{package['name']}@{package['version']}","licenses":[{"license":{"id":package["license"]}}] if package.get("license") else []})
document={"bomFormat":"CycloneDX","specVersion":"1.5","serialNumber":"urn:uuid:00000000-0000-4000-8000-00000000c110","version":1,"metadata":{"timestamp":datetime.now(timezone.utc).isoformat(),"component":{"type":"application","name":"Clio VQPU","version":"definitive"}},"components":sorted(components,key=lambda item:item["name"])}
OUT.parent.mkdir(parents=True,exist_ok=True);OUT.write_text(json.dumps(document,indent=2)+"\n",encoding="utf-8");print(OUT)
