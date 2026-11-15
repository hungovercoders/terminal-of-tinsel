#!/usr/bin/env python3
"""Plays a .tape file into a real terminal session and records what it prints.

    pty_record.py <tape> <cols> <rows> <out.json>

Starts `bash` (the first one on PATH) in a pseudo-terminal of the given size,
presses the keys the tape lists at the pace it sets, and writes every chunk
of output with the time it arrived. record.cjs draws the frames from that.

Only the tape commands the demos use are understood: Type, Enter, Tab, Space,
Right, Left, Up, Down, Escape, Backspace, Sleep and `Set TypingSpeed`. Other
`Set` lines are for the frame renderer and are skipped here.
"""

import base64
import fcntl
import json
import os
import pty
import re
import select
import struct
import sys
import termios
import time

KEYS = {
    "Enter": "\r",
    "Tab": "\t",
    "Space": " ",
    "Escape": "\x1b",
    "Backspace": "\x7f",
    "Right": "\x1b[C",
    "Left": "\x1b[D",
    "Up": "\x1b[A",
    "Down": "\x1b[B",
}


def seconds(text):
    """'400ms', '2.5s' or a bare number of seconds."""
    if text.endswith("ms"):
        return float(text[:-2]) / 1000
    return float(text.rstrip("s"))


class Session:
    """A shell in a pseudo-terminal, and everything it has printed."""

    def __init__(self, cols, rows):
        self.pid, self.fd = pty.fork()
        if self.pid == 0:
            os.environ["TERM"] = "xterm-256color"
            os.environ["COLORTERM"] = "truecolor"
            os.execvp("bash", ["bash"])
        fcntl.ioctl(self.fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
        self.start = time.monotonic()
        self.chunks = []
        self.typing = 0.05

    def elapsed(self):
        return time.monotonic() - self.start

    def read(self, timeout):
        """One chunk of output, or None if nothing came in time."""
        ready, _, _ = select.select([self.fd], [], [], timeout)
        if not ready:
            return None
        try:
            return os.read(self.fd, 65536) or None
        except OSError:
            return None

    def listen(self, duration):
        """Collects output for this long."""
        until = time.monotonic() + duration
        while (left := until - time.monotonic()) > 0:
            data = self.read(left)
            if data is None:
                return
            self.chunks.append(
                {"t": self.elapsed(), "data": base64.b64encode(data).decode()}
            )

    def press(self, keys):
        """Presses each key, with the typing pause after it."""
        for key in keys:
            os.write(self.fd, key.encode())
            self.listen(self.typing)

    def close(self):
        try:
            os.kill(self.pid, 9)
            os.waitpid(self.pid, 0)
        except OSError:
            pass


def play(session, line, where):
    """Carries out one line of a tape."""
    word, _, rest = line.partition(" ")
    rest = rest.strip()
    if word == "Sleep":
        session.listen(seconds(rest))
    elif word == "Type":
        quoted = re.fullmatch(r"""(["'`])(.*)\1""", rest)
        if not quoted:
            sys.exit(f"{where}: can't read: {line}")
        session.press(quoted.group(2))
    elif word in KEYS:
        session.press([KEYS[word]] * (int(rest) if rest else 1))
    elif word == "Set":
        name, _, value = rest.partition(" ")
        if name == "TypingSpeed":
            session.typing = seconds(value)
    else:
        sys.exit(f"{where}: this recorder doesn't know the command: {line}")


def main():
    tape, cols, rows, out = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4]
    session = Session(cols, rows)
    # Let the shell draw its prompt before the first key.
    session.listen(0.6)
    lead_in = session.elapsed()
    with open(tape, encoding="utf-8") as lines:
        for line in lines:
            line = line.strip()
            if line and not line.startswith("#"):
                play(session, line, tape)
    duration = session.elapsed()
    session.close()
    with open(out, "w", encoding="utf-8") as f:
        json.dump({"leadIn": lead_in, "duration": duration, "chunks": session.chunks}, f)


if __name__ == "__main__":
    main()
