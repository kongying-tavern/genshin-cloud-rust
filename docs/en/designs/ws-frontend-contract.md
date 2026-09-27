# WebSocket Frontend Contract

Status: **decision pending (frontend side)** · Last verified: 2026-09-27

This note records the contract of the Rust backend's push channel, what the
frontend repositories actually consume today, and the options for converging
the two. It exists so the decision can be made with the facts on the table —
the backend side is settled and hardened; nothing in this note requests
backend work.

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
2. **Backend adds a socket.io-compatible endpoint** (e.g. via a Rust socket.io
   implementation). Keeps the frontend untouched but adds a second protocol
   surface to maintain — and the existing Java-parity endpoint remains
   without a consumer anyway.
3. **Park the channel** until a consumer exists. Zero cost; the endpoint is
   already behind an env switch and fully tested, so this is a soft default
   rather than a removal.

## Recommendation

Option 1 when `map_register_v3` moves to the Rust backend. The decision
belongs to the frontend maintainers; this document is the hand-off. The
`WS_AUTH_REQUIRED` escape hatch exists for a migration window if a legacy
consumer that cannot send tokens turns up.
