# Adapter recipe

Grafting MONTE CRISTO's control plane onto any future agent platform takes two steps.

1. Find where that platform reads standing instructions on session start. Examples of the
   convention as of this pack's generation: Claude Code reads CLAUDE.md; Codex and the
   generic convention read AGENTS.md; Hermes reads .hermes/6layer.md; OpenClaw reads
   .openclaw/6layer.md; Gemini CLI reads GEMINI.md; Copilot reads
   .github/copilot-instructions.md; Cursor reads .cursor/rules/*.mdc; Cline reads
   .clinerules/*.md.

2. Place the PRIME BLOCK there verbatim, byte for byte, between its BEGIN and END markers,
   with one line above it naming the platform.

Nothing else is ever needed, because all real content lives in AGENTS.md and the pack. An
adapter that contains anything volatile is a defect: adapters must stay byte-stable for the
entire run so they never invalidate a prefix cache.

Parity is checked by the command in COMMANDS.md section 9. Every adapter emitted by this
pack is listed there; if you add one, add it to that command in the same commit.

Adapters currently emitted: AGENTS.md (canonical, participates in parity), CLAUDE.md,
.hermes/6layer.md, .openclaw/6layer.md.
