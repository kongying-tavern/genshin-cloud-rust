# WebSocket 前端契约

状态：**待前端侧决策** · 最近核实：2026-09-27

本文记录 Rust 后端推送通道的契约、前端仓库当前实际消费的情况、以及收敛
两者的可选路径。目的在于把事实摆全再决策——后端侧已定型并完成加固，本文
不请求任何后端改动。

## Rust 后端提供什么

`GET /ws/{userId}` —— 对齐 Java `WebSocketEntrypoint` 协议的裸
（RFC 6455）WebSocket 端点：

- **握手默认鉴权**（`WS_AUTH_REQUIRED=false` 逃生阀可恢复 Java
  pass-filter 语义，仅供可信网络）。token 走 `Authorization: Bearer` 头或
  `?token=` 查询参数——浏览器 `new WebSocket(...)` 无法自定义请求头，
  查询参数即浏览器端通道。路径 `userId` 必须与 token 主体一致；连接
  注册键取认证身份，绝不信任裸路径。
- **连接上限与背压**：单用户至多 8 连接、全局 2048；每连接经容量 128
  的有界通道接收事件——慢消费者队列满即被断开，而不是无限堆积内存。
- **消息形状**（Java `W<T>`）：`{"event": "...", "message": "",
  "data": <T>, "time": "YYYY-MM-DD HH:MM:SS"}`。
- **心跳**：客户端发 `{"action":"Ping"}`，服务端定向回 `Pong` 事件。
- **事件族**：`NoticeAdded`、`MarkerAdded`、`MarkerLinkageDeleted`、
  防抖合并的 `*BinaryPurged` 缓存失效广播（30 秒窗口）、以及
  `UserKickedOut` 等定向账号事件。

## 前端实际消费什么（2026-09 核实）

- **`map_register_v3`**（管理端）经 **socket.io-client** 连接
  （`transports: ['websocket']`）。socket.io 是自有握手协议（engine.io
  帧，落在 `/socket.io/`），与裸 RFC 6455 端点**不兼容**——无论带不带
  token 都无法直连 `/ws/{userId}`。
- **`map_front_v3`**（生产地图前端）**没有 WebSocket 使用**。

也就是说：当前没有任何前端在消费 Rust 推送端点。加固波次（鉴权、上限、
背压）施加在一个消费者全部在未来时点的端点上——而这恰恰是加固最便宜的
时机。

## 可选路径

1. **前端迁移到裸 WebSocket 契约**（推荐）。`map_register_v3` 的 socket
   逻辑隔离在一个 worker（`src/worker/webSocket/socket.worker.ts`），把
   socket.io-client 换成原生 `WebSocket` + 本文的 token 约定是受控改动。
   后端零改动。
2. **后端补 socket.io 兼容端点**（如 Rust 侧 socket.io 实现）。前端不动，
   但多维护一层协议表面——而且既有的 Java 对齐端点依旧没有消费者。
3. **暂时搁置该通道**直到出现消费者。零成本；端点已在 env 开关之后且
   测试完备，这是软性默认而非移除。

## 建议

在 `map_register_v3` 切到 Rust 后端时选路径 1。决策权在前端维护者；本文
即交接文档。若未来出现无法带 token 的旧消费者，`WS_AUTH_REQUIRED` 逃生阀
专为迁移窗口保留。
