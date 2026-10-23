# WebSocket Frontend Contract

Status: **decided on 2026-10-23: socket.io-compatible mode** · Last verified: 2026-10-22

This note records the contract of the Rust backend's push channel, what the
frontend repositories actually consume today, and the options for converging
the two. It existed so the decision could be made with the facts on the
table — the backend side was settled and hardened at the time of writing;
as of 2026-10-23 the confirmed decision at the end of this note applies.

## What the Rust backend provides

`GET /ws/{userId}` — a plain (RFC 6455) WebSocket endpoint, aligned with the
Java `WebSocketEntrypoint` protocol:

- **Handshake authentication is on by default** (`WS_AUTH_REQUIRED=false` is

the escape hatch restoring the Java pass-filter semantics for trusted
networks). The token comes from the `Authorization: Bearer` header or the
`?token=` query parameter — browsers cannot set custom headers on
`new WebSocket(...)`, so the query parameter is the browser-side channel.
The path `userId` must equal the token subject; the registry key is derived
from the authenticated identity, never from the raw path.

- **Session limits and backpressure**: at most 8 connections per user and 2048

globally; each connection receives events through a bounded channel (128) —
a slow consumer whose queue fills up is disconnected instead of ballooning
memory.

- **Message shape** (Java `W<T>`): `{"event": "...", "message": "",

"data": <T>, "time": "YYYY-MM-DD HH:MM:SS"}`.

- **Heartbeat**: the client sends `{"action":"Ping"}`; the server answers with

a directed `Pong` event.

- **Events**: `NoticeAdded`, `MarkerAdded`, `MarkerLinkageDeleted`,

debounced `*BinaryPurged` cache-invalidation broadcasts (30 s window), and
directed account events such as `UserKickedOut`.

## What the frontends actually consume (verified 2026-09)

- **`map_register_v3`** (admin app) connects through **socket.io-client**

(`transports: ['websocket']`). socket.io speaks its own handshake protocol
(engine.io framing at `/socket.io/`), which is **not compatible** with a
plain RFC 6455 endpoint — it cannot talk to `/ws/{userId}` as implemented,
with or without a token.

- **`map_front_v3`** (production map frontend) has **no WebSocket usage**.

In other words: no current frontend consumes the Rust push endpoint. The
hardening waves (authentication, limits, backpressure) were applied to an
endpoint whose consumers are all in the future — which is exactly when such
hardening is cheapest.

## Options

1. **Frontend migrates to the plain WebSocket contract** (recommended).

`map_register_v3` isolates its socket logic in a worker
(`src/worker/webSocket/socket.worker.ts`); swapping socket.io-client for a
native `WebSocket` plus the documented token handling is a contained
change. The backend needs nothing.

1. **Backend adds a socket.io-compatible endpoint** (e.g. via a Rust socket.io

implementation). Keeps the frontend untouched but adds a second protocol
surface to maintain — and the existing Java-parity endpoint remains
without a consumer anyway.

1. **Park the channel** until a consumer exists. Zero cost; the endpoint is

already behind an env switch and fully tested, so this is a soft default
rather than a removal.

## Recommendation

Option 1 when `map_register_v3` moves to the Rust backend. The decision
belongs to the frontend maintainers; this document is the hand-off. The
`WS_AUTH_REQUIRED` escape hatch exists for a migration window if a legacy
consumer that cannot send tokens turns up.

## 2026-10-22 update

- The socket.io migration has landed on `origin/dev` (`map_register_v3`,

since 2026-09: `src/worker/webSocket/socket.worker.ts` consumes
socket.io options directly, and `src/stores/socket/hooks/useSocket.ts`
opens the connection with path/query). The socket.io-client finding above
is now a confirmed fact; the path-1/2 decision stays open.

- The pre-migration local tree has been patched per the token convention

(branch `fix/ws-token-query`: `getURL` appends `?token=`). That patch
does not apply to the migrated socket.io channel — it only serves the
transition window while a raw-WS frontend build is still in use.

## 2026-10-23 decision

- **Final decision: socket.io-compatible mode (path 2)** — confirmed by the
maintainer: the Rust backend adds a socket.io-compatible endpoint (including
handshake auth) and the frontend keeps socket.io-client; the raw-WS migration
(path 1) is dropped. This supersedes the "Recommendation" above (which
favored path 1) and the "stays open" wording in the 2026-10-22 update.

- The frontend migration prepared for path 1 (`map_register_v3` PR #168: raw
WS + `?token=` + Ping/Pong, with 15 unit tests) has been withdrawn and
closed; the code remains on the fork branch `feat/raw-ws-client` for
reference.

## 2026-10-23 implementation landed (#155)

- The Rust backend serves a Socket.IO-compatible channel via `socketioxide`
(protocol v5 / EIO=4, works with `socket.io-client` 4.x out of the box) at
`WS_SOCKETIO_PATH` (default `/socket.io`). Auth follows the raw-WS rules
(`WS_AUTH_REQUIRED` / dual token channels / subject consistency); events are
pushed as the `message` event with a **string** payload (the `W<T>` envelope
JSON); `rttcheck` echoes timestamps per the Java semantics; the raw WS
endpoint remains as a transition crutch.

- Frontend follow-up (one line): include `token` in the socket.io connection
query — until then deployments can run with `WS_AUTH_REQUIRED=false` (the
Java gateway pass-filter equivalent). Whether to retire the raw `/ws`
endpoint and the escape hatch will be revisited once the new channel is
stable.
