import hashlib
import tempfile
import unittest
from pathlib import Path

from scripts import check_sha256_file


class CheckSha256FileTests(unittest.TestCase):
    def test_accepts_matching_checksum_output(self):
        artifact_bytes = b"release artifact"
        digest = hashlib.sha256(artifact_bytes).hexdigest()

        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=artifact_bytes,
                checksum_output=f"{digest}  vogon.tar.gz\n",
            ),
            [],
        )

    def test_accepts_binary_marker_from_sha256sum(self):
        artifact_bytes = b"release artifact"
        digest = hashlib.sha256(artifact_bytes).hexdigest()

        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=artifact_bytes,
                checksum_output=f"{digest} *vogon.tar.gz\n",
            ),
            [],
        )

    def test_accepts_artifact_and_default_checksum_paths(self):
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "vogon.zip"
            artifact.write_bytes(b"release artifact")
            digest = hashlib.sha256(artifact.read_bytes()).hexdigest()
            Path(f"{artifact}.sha256").write_text(
                f"{digest}  vogon.zip",
                encoding="utf-8",
            )

            self.assertEqual(check_sha256_file.check_file(artifact), [])

    def test_reports_missing_artifact(self):
        with tempfile.TemporaryDirectory() as directory:
            errors = check_sha256_file.check_file(Path(directory) / "missing.tar.gz")

            self.assertEqual(len(errors), 1)
            self.assertTrue(errors[0].startswith("Artifact cannot be read:"))

    def test_reports_bad_checksum_format(self):
        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=b"release artifact",
                checksum_output="not-a-checksum\n",
            ),
            ["Checksum line must contain a SHA-256 digest and artifact filename"],
        )

    def test_reports_extra_checksum_lines(self):
        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=b"release artifact",
                checksum_output="first\nsecond\n",
            ),
            ["Checksum file must contain exactly one checksum line"],
        )

    def test_reports_invalid_digest(self):
        actual_digest = hashlib.sha256(b"release artifact").hexdigest()

        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=b"release artifact",
                checksum_output="abc  vogon.tar.gz\n",
            ),
            [
                "Checksum digest must be 64 hexadecimal characters",
                f"Checksum digest mismatch: expected abc, got {actual_digest}",
            ],
        )

    def test_reports_filename_mismatch(self):
        artifact_bytes = b"release artifact"
        digest = hashlib.sha256(artifact_bytes).hexdigest()

        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=artifact_bytes,
                checksum_output=f"{digest}  other.tar.gz\n",
            ),
            ["Checksum filename mismatch: expected vogon.tar.gz, got other.tar.gz"],
        )

    def test_reports_digest_mismatch(self):
        wrong_digest = hashlib.sha256(b"other artifact").hexdigest()
        actual_digest = hashlib.sha256(b"release artifact").hexdigest()

        self.assertEqual(
            check_sha256_file.check_output(
                artifact_name="vogon.tar.gz",
                artifact_bytes=b"release artifact",
                checksum_output=f"{wrong_digest}  vogon.tar.gz\n",
            ),
            [
                f"Checksum digest mismatch: expected {wrong_digest}, got {actual_digest}"
            ],
        )


if __name__ == "__main__":
    unittest.main()
