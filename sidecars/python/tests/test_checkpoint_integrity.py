"""
sidecars/python/tests/test_checkpoint_integrity.py

Automated Security & Integrity Test Suite for Model Checkpoint Loading.
Verifies fail-closed behavior on corrupted, truncated, and unregistered checkpoints.
"""

import os
import sys
import tempfile
import unittest
import torch

# Ensure sidecar root is on path
_root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
if _root not in sys.path:
    sys.path.insert(0, _root)

from security.integrity import (
    load_manifest,
    lookup_checkpoint_metadata,
    load_verified_checkpoint_stream,
    safe_torch_load,
    ModelIntegrityError,
)

class TestCheckpointIntegrity(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.models_dir = os.path.join(_root, "models", "kokoro")
        cls.valid_pth = os.path.join(cls.models_dir, "kokoro-v1_0.pth")
        cls.valid_voice = os.path.join(cls.models_dir, "voices", "af_heart.pt")
        cls.temp_dir = tempfile.TemporaryDirectory()

    @classmethod
    def tearDownClass(cls):
        cls.temp_dir.cleanup()

    def test_01_manifest_validity(self):
        """Manifest loads and contains required upstream LFS entries."""
        manifest = load_manifest()
        self.assertIn("repositories", manifest)
        self.assertIn("hexgrad/Kokoro-82M", manifest["repositories"])
        kokoro_files = manifest["repositories"]["hexgrad/Kokoro-82M"]["files"]
        self.assertIn("kokoro-v1_0.pth", kokoro_files)
        self.assertIn("voices/af_heart.pt", kokoro_files)
        self.assertEqual(len(kokoro_files["kokoro-v1_0.pth"]["sha256"]), 64)

    def test_02_lookup_metadata(self):
        """Metadata lookup matches by relative path or basename."""
        m1 = lookup_checkpoint_metadata("kokoro-v1_0.pth")
        self.assertIsNotNone(m1)
        self.assertEqual(m1["sha256"], "496dba118d1a58f5f3db2efc88dbdc216e0483fc89fe6e47ee1f2c53f18ad1e4")

        m2 = lookup_checkpoint_metadata(self.valid_voice)
        self.assertIsNotNone(m2)
        self.assertEqual(m2["sha256"], "0ab5709b8ffab19bfd849cd11d98f75b60af7733253ad0d67b12382a102cb4ff")

    def test_03_valid_checkpoint_stream(self):
        """Valid voice file produces verified BytesIO stream."""
        if not os.path.isfile(self.valid_voice):
            self.skipTest(f"Voice file missing: {self.valid_voice}")
        stream = load_verified_checkpoint_stream(self.valid_voice)
        self.assertIsNotNone(stream)
        # Verify torch.load succeeds on the stream
        tensor = torch.load(stream, weights_only=True)
        self.assertIsInstance(tensor, torch.Tensor)

    def test_04_tampered_checkpoint_fails_closed(self):
        """Single-bit modified checkpoint raises ModelIntegrityError and aborts."""
        if not os.path.isfile(self.valid_voice):
            self.skipTest("Voice file missing")

        with open(self.valid_voice, "rb") as f:
            data = bytearray(f.read())

        # Flip a bit in the body of the tensor stream
        data[2048] ^= 0xFF

        tampered_path = os.path.join(self.temp_dir.name, "af_heart.pt")
        with open(tampered_path, "wb") as f:
            f.write(data)

        # Must fail closed with ModelIntegrityError
        with self.assertRaises(ModelIntegrityError) as ctx:
            safe_torch_load(tampered_path, weights_only=True)
        self.assertIn("Cryptographic integrity verification failed", str(ctx.exception))

    def test_05_truncated_checkpoint_fails_closed(self):
        """Truncated checkpoint is rejected on size check before hashing."""
        truncated_path = os.path.join(self.temp_dir.name, "af_heart.pt")
        with open(truncated_path, "wb") as f:
            f.write(b"PK\x03\x04incomplete_data")

        with self.assertRaises(ModelIntegrityError) as ctx:
            safe_torch_load(truncated_path, weights_only=True)
        self.assertIn("Checkpoint size mismatch", str(ctx.exception))

    def test_06_unregistered_checkpoint_fails_closed(self):
        """Checkpoint not present in manifest is rejected immediately."""
        unregistered_path = os.path.join(self.temp_dir.name, "untrusted_exploit.pth")
        with open(unregistered_path, "wb") as f:
            f.write(b"malicious_payload_simulation")

        with self.assertRaises(ModelIntegrityError) as ctx:
            safe_torch_load(unregistered_path, weights_only=True)
        self.assertIn("Untrusted or unregistered checkpoint", str(ctx.exception))

    def test_07_safe_torch_load_valid(self):
        """safe_torch_load works identically to torch.load on verified inputs."""
        if not os.path.isfile(self.valid_voice):
            self.skipTest("Voice file missing")
        pack = safe_torch_load(self.valid_voice, weights_only=True)
        self.assertIsInstance(pack, torch.Tensor)
        self.assertEqual(pack.shape, torch.Size([510, 1, 256]))


if __name__ == "__main__":
    unittest.main()
