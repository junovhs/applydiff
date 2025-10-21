ApplyDiff
=========

Reliable, token‑efficient code patching you can trust with AI outputs.

## Two patch formats (use the armored one in chat)

**1) Armored Framed Blocks v1 (AFB‑1)** — resilient to chat apps that mangle whitespace and markup.
Use this whenever you're pasting patches in Slack/Discord/ChatGPT/etc.

-----BEGIN APPLYDIFF AFB-1-----
Path: RELATIVE/PATH/TO/FILE
Fuzz: 0.85
Encoding: base64
From:
<base64 of exact old text; may be empty to create/append>
To:
<base64 of new text>
-----END APPLYDIFF AFB-1-----

Notes:
- Multiple blocks can be placed back‑to‑back.
- Base64 can be wrapped; whitespace is ignored.
- Leave "From:" empty (i.e., encode the empty string) to create/append.

**2) Classic sentinel format** — still supported for local usage and when you can trust the transport.

>>> file: RELATIVE/PATH | fuzz=0.85
--- from
<exact old text (may be empty to append)>
--- to
<new text>
<<<

## Behavior highlights

- Path safety: absolute/`..` paths are rejected.
- Line endings preserved: replacement adopts the matched region's `\n` or `\r\n`.
- Append/create: adds a separator newline **only** when appending to a non‑empty file that lacks one.
- Partial apply: good blocks land, bad ones are skipped; backups are created first.

## Built‑in test suite (Gauntlet)

After building, run the self‑test from the app console. Expected cases:

- LF01-Replace-Start — exact replace at file start (fast path)
- MA01a-Simple-Ambiguity — duplicate blocks, must reject
- MA01b-Indentation-Ambiguity — equal candidates, must reject
- FS01-Path-Traversal — reject `../escape.txt`
- FS02-Append-Create — create deep file via empty `from`
- AFB01-Armored-Append-Create — same as FS02 using AFB‑1
- AFB02-Armored-Replace-Exact — replace using AFB‑1

## Quick usage

1. Select your project directory in the app.
2. Paste one or more AFB‑1 blocks or classic format blocks.
3. Review the preview diff.
4. Apply. Backups will be created automatically.