"""
sidecars/python/security/integrity.py

Cryptographic Model Checkpoint Integrity & TOCTOU-Immune Verification Gate.
Grounded in upstream Git LFS commit pointers from Hugging Face official repositories.
"""

import os
import io
import json
import hashlib
import torch
from typing import Dict, Any, Optional, Union

class ModelIntegrityError(RuntimeError):
    """Raised when a neural model checkpoint fails cryptographic verification."""
    pass


_CACHED_MANIFEST: Optional[Dict[str, Any]] = None


def get_manifest_path() -> str:
    """Resolves the trusted read-only application manifest path."""
    candidates = [
        # 1. sidecars/python/models/checksums.json
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "models", "checksums.json")),
        # 2. resources/models/manifest.json
        os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", "..", "resources", "models", "manifest.json")),
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    raise ModelIntegrityError("Authoritative model checksum manifest not found in application package.")


def load_manifest() -> Dict[str, Any]:
    """Loads and caches the authoritative Git LFS model manifest."""
    global _CACHED_MANIFEST
    if _CACHED_MANIFEST is not None:
        return _CACHED_MANIFEST

    m_path = get_manifest_path()
    try:
        with open(m_path, "r", encoding="utf-8") as f:
            data = json.load(f)
            _CACHED_MANIFEST = data
            return data
    except Exception as e:
        raise ModelIntegrityError(f"Failed to parse model checksum manifest: {e}")


def lookup_checkpoint_metadata(filename: str) -> Optional[Dict[str, Any]]:
    """
    Looks up expected SHA-256 and size from the authoritative manifest.
    Matches either exact relative key (e.g. 'voices/af_heart.pt') or basename.
    """
    manifest = load_manifest()
    repos = manifest.get("repositories", {})

    clean_name = os.path.basename(filename).strip().lower()
    norm_name = filename.replace("\\", "/").strip().lower()

    for repo_name, repo_data in repos.items():
        files = repo_data.get("files", {})
        for rel_key, file_meta in files.items():
            rel_clean = rel_key.replace("\\", "/").strip().lower()
            if rel_clean == norm_name or rel_clean.endswith("/" + clean_name) or rel_clean == clean_name:
                return file_meta
    return None


def load_verified_checkpoint_stream(filepath: str) -> io.BytesIO:
    """
    Reads a model checkpoint into an isolated private in-memory snapshot,
    computes SHA-256, validates against the authoritative Git LFS OID,
    and returns the validated BytesIO stream.
    
    Guarantees zero TOCTOU race: no external process can mutate the private
    in-memory buffer between verification and torch.load.
    """
    if not os.path.isfile(filepath):
        raise ModelIntegrityError(f"Checkpoint file does not exist: {filepath}")

    meta = lookup_checkpoint_metadata(filepath)
    if not meta or "sha256" not in meta:
        raise ModelIntegrityError(
            f"Untrusted or unregistered checkpoint: '{os.path.basename(filepath)}'. "
            f"Checkpoint is not present in authoritative application manifest."
        )

    expected_sha256 = meta["sha256"].strip().lower()
    expected_size = meta.get("size")

    actual_file_size = os.path.getsize(filepath)
    if expected_size is not None and actual_file_size != expected_size:
        raise ModelIntegrityError(
            f"Checkpoint size mismatch for '{os.path.basename(filepath)}': "
            f"expected {expected_size} bytes, got {actual_file_size} bytes."
        )

    # Read into isolated private memory snapshot
    with open(filepath, "rb") as f:
        data = f.read()

    # Streaming SHA-256 computation over private snapshot
    computed_sha256 = hashlib.sha256(data).hexdigest().lower()
    if computed_sha256 != expected_sha256:
        raise ModelIntegrityError(
            f"Cryptographic integrity verification failed for '{os.path.basename(filepath)}'!\n"
            f"  Expected (Git LFS OID): {expected_sha256}\n"
            f"  Computed SHA-256:       {computed_sha256}\n"
            f"Possible corruption, tampering, or unauthorized model replacement."
        )

    buf = io.BytesIO(data)
    buf.seek(0)
    return buf


def safe_torch_load(
    filepath_or_buffer: Union[str, os.PathLike, io.BytesIO],
    map_location=None,
    weights_only: bool = True
) -> Any:
    """
    Safely deserializes a PyTorch model checkpoint.
    If a file path is provided, it is verified through the private snapshot
    integrity gate BEFORE reaching torch.load.
    """
    if isinstance(filepath_or_buffer, (str, os.PathLike)):
        stream = load_verified_checkpoint_stream(str(filepath_or_buffer))
        return torch.load(stream, map_location=map_location, weights_only=weights_only)
    elif isinstance(filepath_or_buffer, io.BytesIO):
        return torch.load(filepath_or_buffer, map_location=map_location, weights_only=weights_only)
    else:
        raise TypeError(f"Unsupported checkpoint source type: {type(filepath_or_buffer)}")
