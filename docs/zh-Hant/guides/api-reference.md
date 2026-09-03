# API 參考概覽

> [← 返回索引](../README.md) · 路由源碼見 `packages/router/src/routes/`

本頁列出 `_router` 二進制（axum）暴露的全部 API 域，按用途分組。這些端點均爲
Java 側控制器的 Rust 移植：路徑、請求/響應結構與 Java 參考實現保持一致，僅
實現語言與傳輸細節不同。每個域對應 `functions/src/functions/api/<domain>.rs`
的業務函數與 `router/src/routes/api/<domain>/` 下的路由文件。

## 地圖內容域

| 域 | 路徑前綴 | 說明 |
| --- | --- | --- |
| area | `/area` | 地區樹（含父子關係、末端標記、權限屏蔽） |
| icon | `/icon` | 地圖圖標資源及其元數據 |
| icon_type | `/icon_type` | 圖標分類 |
| tag | `/tag` | 圖標標籤（把 `icon` 歸到 `tag_type` 下） |
| tag_type | `/tag_type` | 標籤分類（樹形） |
| item | `/item` | 地圖條目（item），含 copy/join 等聚合操作 |
| item_type | `/item_type` | 條目分類，含 move_type 排序 |
| item_common | `/item_common` | 條目公共屬性 |
| marker | `/marker` | 打點（marker）的增刪改查、single、tweak |
| marker_link | `/marker_link` | 打點之間的關聯關係 |
| notice | `/notice` | 公告管理 |
| history | `/history` | 歷史版本列表 |
| cache | `/cache` | 按域的緩存刷新（area/item/marker/icon_tag/notice 等） |
| app | `/app` | 觸發應用更新（清空 BinaryMD5 緩存，客戶端下次輪詢重新拉取） |

## 歸檔與文檔導出

| 域 | 路徑前綴 | 說明 |
| --- | --- | --- |
| item_doc | `/item_doc` | 條目分頁導出（bin 二進制 / md5 校驗） |
| marker_doc | `/marker_doc` | 打點分頁導出（bin / md5） |
| marker_link_doc | `/marker_link_doc` | 打點關聯導出 |
| icon_doc | `/icon_doc` | 圖標歸檔導出（單 blob） |
| tag_doc | `/tag_doc` | 圖標標籤歸檔導出（單 blob） |
| res | `/res` | 資源上傳（MinIO） |

> `*_doc` 系列對應 Java 側 BinaryMD5 壓縮歸檔導出能力。

## 評分

| 域 | 路徑前綴 | 說明 |
| --- | --- | --- |
| score | `/score` | 評分數據與生成 |

## 系統域

掛在 `/system` 下（`router/src/routes/system/`）：

| 域 | 說明 |
| --- | --- |
| user | 用戶管理 |
| role | 角色與權限 |
| device | 設備登記 |
| invitation | 邀請碼 |
| action_log | 操作日誌 |
| oauth | OAuth2 接入 |
| archive | 歸檔管理 |

## 鑑權

除少量公開端點外，所有路由經 `ExtractAuthInfo` 中間件提取 JWT 中的 `AuthInfo`，
業務函數據此做權限判定。鑑權細節見 `packages/utils/src/jwt.rs`。
