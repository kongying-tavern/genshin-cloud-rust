# WebSocket 前端契約

狀態：**待前端側決策** · 最近核實：2026-09-27

本文記錄 Rust 後端推送通道的契約、前端倉庫目前實際消費的情況、以及收斂
兩者的可選路徑。目的在於把事實擺全再決策——後端側已定形並完成加固，本文
不請求任何後端改動。

## Rust 後端提供什麼

`GET /ws/{userId}` —— 對齊 Java `WebSocketEntrypoint` 協定的裸
（RFC 6455）WebSocket 端點：

- **握手預設鑑權**（`WS_AUTH_REQUIRED=false` 逃生閥可恢復 Java
  pass-filter 語義，僅供可信網路）。token 走 `Authorization: Bearer` 標頭
  或 `?token=` 查詢參數——瀏覽器 `new WebSocket(...)` 無法自訂請求標頭，
  查詢參數即瀏覽器端通道。路徑 `userId` 必須與 token 主體一致；連線
  註冊鍵取認證身分，絕不信任裸路徑。
- **連線上限與背壓**：單使用者至多 8 連線、全域 2048；每連線經容量 128
  的有界通道接收事件——慢消費者佇列滿即被斷開，而不是無限堆積記憶體。
- **訊息形狀**（Java `W<T>`）：`{"event": "...", "message": "",
  "data": <T>, "time": "YYYY-MM-DD HH:MM:SS"}`。
- **心跳**：客戶端送 `{"action":"Ping"}`，伺服器定向回 `Pong` 事件。
- **事件族**：`NoticeAdded`、`MarkerAdded`、`MarkerLinkageDeleted`、
  防抖合併的 `*BinaryPurged` 快取失效廣播（30 秒視窗）、以及
  `UserKickedOut` 等定向帳號事件。

## 前端實際消費什麼（2026-09 核實）

- **`map_register_v3`**（管理端）經 **socket.io-client** 連線
  （`transports: ['websocket']`）。socket.io 是自有交握協定（engine.io
  幀，落在 `/socket.io/`），與裸 RFC 6455 端點**不相容**——無論帶不帶
  token 都無法直連 `/ws/{userId}`。
- **`map_front_v3`**（生產地圖前端）**沒有 WebSocket 使用**。

也就是說：目前沒有任何前端在消費 Rust 推送端點。加固波次（鑑權、上限、
背壓）施加在一個消費者全部在未來時點的端點上——而這恰恰是加固最便宜的
時機。

## 可選路徑

1. **前端遷移到裸 WebSocket 契約**（建議）。`map_register_v3` 的 socket
   邏輯隔離在一個 worker（`src/worker/webSocket/socket.worker.ts`），把
   socket.io-client 換成原生 `WebSocket` + 本文的 token 約定是受控改動。
   後端零改動。
2. **後端補 socket.io 相容端點**（如 Rust 側 socket.io 實作）。前端不動，
   但多維護一層協定表面——而且既有的 Java 對齊端點依舊沒有消費者。
3. **暫時擱置該通道**直到出現消費者。零成本；端點已在 env 開關之後且
   測試完備，這是軟性預設而非移除。

## 建議

在 `map_register_v3` 切到 Rust 後端時選路徑 1。決策權在前端維護者；本文
即交接文件。若未來出現無法帶 token 的舊消費者，`WS_AUTH_REQUIRED` 逃生閥
專為遷移視窗保留。
