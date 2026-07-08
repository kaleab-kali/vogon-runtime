import subprocess
import unittest
from collections.abc import Sequence

from scripts import check_container_image


class FakeDockerRunner:
    def __init__(
        self,
        *,
        labels: dict[str, str] | None = None,
        user_id: str = "10001",
        failures: dict[str, str] | None = None,
    ) -> None:
        self.labels = labels or check_container_image.EXPECTED_LABELS
        self.user_id = user_id
        self.failures = failures or {}
        self.commands: list[Sequence[str]] = []

    def __call__(self, command: Sequence[str]) -> subprocess.CompletedProcess[str]:
        self.commands.append(command)
        key = " ".join(command)
        for failure_key, stderr in self.failures.items():
            if failure_key in key:
                return subprocess.CompletedProcess(
                    command,
                    1,
                    stdout="",
                    stderr=stderr,
                )

        if command[:3] == ["docker", "image", "inspect"]:
            label = str(command[-1]).split('"')[1]
            return subprocess.CompletedProcess(
                command,
                0,
                stdout=f"{self.labels.get(label, '')}\n",
                stderr="",
            )

        if command[:4] == ["docker", "run", "--rm", "--entrypoint"]:
            return subprocess.CompletedProcess(
                command,
                0,
                stdout=f"{self.user_id}\n",
                stderr="",
            )

        return subprocess.CompletedProcess(command, 127, stdout="", stderr="unexpected")


class CheckContainerImageTests(unittest.TestCase):
    def test_accepts_expected_image_metadata(self):
        runner = FakeDockerRunner()

        self.assertEqual(
            check_container_image.check_image("vogon-runtime:ci", runner=runner),
            [],
        )

        self.assertEqual(len(runner.commands), 6)

    def test_accepts_release_version_and_revision_labels(self):
        labels = {
            **check_container_image.EXPECTED_LABELS,
            "org.opencontainers.image.version": "v0.1.0",
            "org.opencontainers.image.revision": "abc123",
        }

        self.assertEqual(
            check_container_image.check_image(
                "vogon-runtime:v0.1.0",
                expected_labels=labels,
                runner=FakeDockerRunner(labels=labels),
            ),
            [],
        )

    def test_reports_label_mismatch(self):
        labels = dict(check_container_image.EXPECTED_LABELS)
        labels["org.opencontainers.image.licenses"] = "Apache-2.0"

        self.assertEqual(
            check_container_image.check_image(
                "vogon-runtime:ci",
                runner=FakeDockerRunner(labels=labels),
            ),
            [
                "Container label org.opencontainers.image.licenses mismatch: "
                "expected MIT, got Apache-2.0"
            ],
        )

    def test_reports_missing_label_as_empty(self):
        labels = dict(check_container_image.EXPECTED_LABELS)
        del labels["org.opencontainers.image.source"]

        self.assertEqual(
            check_container_image.check_image(
                "vogon-runtime:ci",
                runner=FakeDockerRunner(labels=labels),
            ),
            [
                "Container label org.opencontainers.image.source mismatch: "
                "expected https://github.com/kaleab-kali/vogon-runtime, got <empty>"
            ],
        )

    def test_reports_user_mismatch(self):
        self.assertEqual(
            check_container_image.check_image(
                "vogon-runtime:ci",
                runner=FakeDockerRunner(user_id="0"),
            ),
            ["Container runtime user mismatch: expected 10001, got 0"],
        )

    def test_reports_command_failure_with_stderr(self):
        self.assertEqual(
            check_container_image.check_image(
                "vogon-runtime:ci",
                runner=FakeDockerRunner(failures={"image inspect": "no such image"}),
            ),
            [
                "Container label org.opencontainers.image.title cannot be read: no such image",
                "Container label org.opencontainers.image.source cannot be read: no such image",
                "Container label org.opencontainers.image.licenses cannot be read: no such image",
                "Container label org.opencontainers.image.version cannot be read: no such image",
                "Container label org.opencontainers.image.revision cannot be read: no such image",
            ],
        )


if __name__ == "__main__":
    unittest.main()
