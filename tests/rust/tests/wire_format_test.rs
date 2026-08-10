//! Wire-format compatibility tests against the **real database data shapes**
//! (see the audit report `db_audit.md`: ddns.minemc.top:13070/genshin_map).
//!
//! These are pure ser/de unit tests — no DB, no tokio. Every sample constant
//! is an inline copy of a shape observed in production rows, so the suite
//! fails loudly the day a VO/enum drifts from the Java-era contract.
//!
//! Coverage:
//! - sys_user.access_policy: prefixed form round-trips; the 2 unprefixed
//!   legacy rows (id=194/195) are documented as a P1 gap (ignored test).
//! - history.content: Java JSON snapshot strings pass through verbatim.
//! - history.editType: numeric strings '0'|'1'|'2'|'3'|'10' from the frontend.
//! - marker.position: single "{x},{y}" string on the wire.
//! - marker.extra: `{"underground":{...}}` opaque Value passthrough.
//! - notice.channel: JSON array of uppercase channel names.
//! - notice.validTimeStart/End: ms number and ISO string both accepted.
//! - marker_linkage.link_action: uppercase enum strings ("TRIGGER_ALL").
//! - item.iconStyleType: numeric 0-3 from the frontend.
//! - sys_user_archive.data: dual shape (legacy object array vs JSON string).
//! - sys_user.password: `{bcrypt}`-prefixed storage (68 chars).

use _database::models::common::notice::ChannelWrapper;
use _utils::bcrypt;
use _utils::models::{
    history::HistoryItemVO,
    item::ItemRequest,
    marker::{MarkerItemLinkVo, MarkerVO},
    marker_link::MarkerLinkVO,
    notice::{NoticeAddRequest, NoticeChannel, NoticeVO},
    system::{ArchiveSlotVo, SysArchiveSlotVo, SysArchiveVo},
};
use _utils::types::{
    AccessPolicyItemEnum, AccessPolicyList, HiddenFlag, HistoryEditType, HistoryOperationType,
    IconStyleType, MarkerLinkageLinkAction,
};

// ─────────────────────────────────────────────────────────────────────────────
// 1. sys_user.access_policy — prefixed form (197/199 rows) round-trips
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn access_policy_prefixed_roundtrip() {
    // The prefixed form is what 197 of 199 real rows store and what the
    // business layer writes; it must never regress.
    let wire = r#"["ip:same_last_ip","dev:same_last_device"]"#;
    let parsed: Vec<AccessPolicyItemEnum> =
        serde_json::from_str(wire).expect("prefixed form deserializes");
    assert_eq!(
        parsed,
        vec![
            AccessPolicyItemEnum::IpSameLastIp,
            AccessPolicyItemEnum::DevSameLastDevice
        ]
    );
    // Serialization keeps the prefix (storage/wire contract).
    assert_eq!(
        serde_json::to_string(&parsed).expect("serialize"),
        wire,
        "prefixes must survive a round-trip"
    );
    // DB json-column path goes through the AccessPolicyList wrapper.
    let wrapped =
        serde_json::from_value::<AccessPolicyList>(serde_json::json!(["ip:same_last_ip"]))
            .expect("AccessPolicyList from stored json");
    assert_eq!(wrapped.0, vec![AccessPolicyItemEnum::IpSameLastIp]);
}

#[test]
fn access_policy_unprefixed_legacy_rows_deserialize() {
    // Real data shape (audit, sys_user id=194 kafka / id=195 firefly, both
    // del_flag=false, role_id=0 Admin): the two unprefixed strings below are
    // the exact values stored in the access_policy json column. Any query
    // hitting these rows currently fails to deserialize the whole row.
    let real_rows = r#"["same_last_ip","same_last_device"]"#;
    let parsed: Vec<AccessPolicyItemEnum> = serde_json::from_str(real_rows)
        .unwrap_or_else(|e| panic!("unprefixed legacy rows must deserialize (P1): {e}"));
    assert_eq!(
        parsed,
        vec![
            AccessPolicyItemEnum::IpSameLastIp,
            AccessPolicyItemEnum::DevSameLastDevice
        ]
    );
    // The DB wrapper must also survive the same input.
    let wrapped = serde_json::from_value::<AccessPolicyList>(serde_json::json!([
        "same_last_ip",
        "same_last_device"
    ]))
    .expect("AccessPolicyList from unprefixed legacy json");
    assert_eq!(wrapped.0.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. history.content — Java JSON snapshot string passes through verbatim
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn history_content_snapshot_passthrough() {
    // Real DB shape (audit, type=4 打点 rows): content is a Java-contract JSON
    // *snapshot string* with top-level content/hiddenFlag/id/itemList/
    // markerCreatorId/markerTitle/picture/pictureCreatorId/position/
    // refreshTime/videoPath. The backend VO must keep it an opaque string.
    const SNAPSHOT: &str = r#"{"content":"风滚草","hiddenFlag":0,"id":66181,"itemList":[{"count":1,"itemId":2992}],"markerCreatorId":28,"markerTitle":"风滚草","picture":"","pictureCreatorId":0,"position":"-7507.75,2244.25","refreshTime":43200000,"videoPath":""}"#;

    let vo = HistoryItemVO {
        version: 0,
        id: 58502,
        create_time: 0.0,
        update_time: None,
        creator_id: Some(46),
        updater_id: None,
        del_flag: false,
        md5: Some("0123456789abcdef0123456789abcdef".into()),
        ipv4: None,
        t_id: 49232,
        history_type: Some(HistoryOperationType::Position),
        edit_type: HistoryEditType::Modified,
        content: SNAPSHOT.to_string(),
    };

    let json = serde_json::to_value(&vo).expect("serialize HistoryItemVO");
    // content is a string on the wire — never a re-parsed nested object.
    assert!(
        json["content"].is_string(),
        "content must stay an opaque string, got: {}",
        json["content"]
    );
    assert_eq!(
        json["content"], SNAPSHOT,
        "content must pass through byte-verbatim"
    );
    // Java wire keys for the rest of the VO.
    assert_eq!(json["tid"], 49232, "t_id serializes as `tid`");
    assert_eq!(json["type"], 4, "historyType serializes as numeric `type`");
    assert_eq!(json["editType"], 2);

    let back: HistoryItemVO = serde_json::from_value(json).expect("deserialize HistoryItemVO");
    assert_eq!(back.content, SNAPSHOT);
    assert_eq!(back.t_id, 49232);
    assert_eq!(back.history_type, Some(HistoryOperationType::Position));
    assert_eq!(back.edit_type, HistoryEditType::Modified);
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. marker.position — single "{x},{y}" string on the wire
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn marker_position_verbatim_string() {
    // Real DB shape (audit): position is one text column, e.g. '-7507.75,2244.25'.
    let vo = MarkerVO {
        version: 0,
        id: 66181,
        create_time: 0.0,
        update_time: None,
        creator_id: None,
        updater_id: None,
        del_flag: false,
        marker_stamp: None,
        marker_title: Some("风滚草".into()),
        position: "-7507.75,2244.25".into(),
        content: Some("风滚草".into()),
        picture: Some(String::new()),
        marker_creator_id: 28,
        picture_creator_id: Some(0),
        video_path: Some(String::new()),
        refresh_time: 43200000,
        hidden_flag: HiddenFlag::Visible,
        extra: Some(serde_json::json!({})),
        item_list: vec![MarkerItemLinkVo {
            item_id: 2992,
            count: 1,
            icon_tag: None,
            icon_id: 0,
        }],
        linkage_id: None,
    };

    let json = serde_json::to_value(&vo).expect("serialize MarkerVO");
    assert!(
        json["position"].is_string(),
        "position must stay a single string, got: {}",
        json["position"]
    );
    assert_eq!(
        json["position"], "-7507.75,2244.25",
        "position must not be split into x/y numbers"
    );
    // itemList survives as the Java camelCase array.
    assert_eq!(json["itemList"][0]["itemId"], 2992);
    assert_eq!(json["itemList"][0]["count"], 1);

    let back: MarkerVO = serde_json::from_value(json).expect("deserialize MarkerVO");
    assert_eq!(back.position, "-7507.75,2244.25");
    assert_eq!(back.hidden_flag, HiddenFlag::Visible);
    assert_eq!(back.item_list[0].item_id, 2992);
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. notice.channel — JSON array of uppercase channel names
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn notice_channel_array_wire() {
    // Real DB shape (audit): channel is a json array of strings, e.g.
    // `['DASHBOARD']` or `['COMMON']`. The VO must serialize the array under
    // the `channel` key and deserialize the stored array back.
    let vo = NoticeVO {
        version: 0,
        id: 6,
        create_time: 0.0,
        update_time: None,
        creator_id: None,
        updater_id: None,
        title: "待生效公告".into(),
        content: Some("<p>你好世界</p>".into()),
        channels: vec![NoticeChannel::Dashboard, NoticeChannel::Common],
        sort_index: 10,
        valid_time_start: None,
        valid_time_end: None,
    };

    let json = serde_json::to_value(&vo).expect("serialize NoticeVO");
    assert!(
        json["channel"].is_array(),
        "channel must be an array on the wire"
    );
    assert_eq!(json["channel"], serde_json::json!(["DASHBOARD", "COMMON"]));

    // DB json-column wrapper (ChannelWrapper) round-trips the stored array.
    let wrapped = serde_json::from_value::<ChannelWrapper>(serde_json::json!(["DASHBOARD"]))
        .expect("ChannelWrapper from stored json");
    assert_eq!(wrapped.0, vec!["DASHBOARD".to_string()]);
    assert_eq!(
        serde_json::to_value(&wrapped).expect("serialize ChannelWrapper"),
        serde_json::json!(["DASHBOARD"])
    );

    // Round-trip the wire shape back into the VO.
    let back: NoticeVO = serde_json::from_value(json).expect("deserialize NoticeVO");
    assert_eq!(
        back.channels,
        vec![NoticeChannel::Dashboard, NoticeChannel::Common]
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. marker_linkage.link_action — uppercase enum strings
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn marker_linkage_link_action_uppercase() {
    // Real DB shape (audit, 3,152 rows): link_action is an uppercase string,
    // e.g. 'TRIGGER_ALL'; all values are in the enum range.
    for (wire, expected) in [
        ("TRIGGER", MarkerLinkageLinkAction::Trigger),
        ("TRIGGER_ALL", MarkerLinkageLinkAction::TriggerAll),
        ("TRIGGER_ANY", MarkerLinkageLinkAction::TriggerAny),
        ("RELATED", MarkerLinkageLinkAction::Related),
        ("DIRECTED", MarkerLinkageLinkAction::Directed),
        ("PATH_UNI_DIR", MarkerLinkageLinkAction::PathUniDir),
        ("PATH_BI_DIR", MarkerLinkageLinkAction::PathBiDir),
        ("EQUIVALENT", MarkerLinkageLinkAction::Equivalent),
    ] {
        let parsed: MarkerLinkageLinkAction =
            serde_json::from_str(&format!("\"{wire}\"")).expect("link_action deserializes");
        assert_eq!(parsed, expected);
        assert_eq!(
            serde_json::to_string(&parsed).expect("serialize"),
            format!("\"{wire}\""),
            "link_action round-trip must preserve the uppercase form"
        );
    }

    // Wire VO (camelCase linkAction) carrying the audited row shape.
    let link = serde_json::json!({
        "version": 0,
        "id": 7,
        "creatorId": null,
        "updaterId": null,
        "updateTime": null,
        "groupId": "851168d7d77d434e93881e504f2a4df1",
        "fromId": 71243,
        "toId": 71244,
        "linkAction": "TRIGGER_ALL",
        "linkReverse": true,
        "path": []
    });
    let vo: MarkerLinkVO = serde_json::from_value(link).expect("MarkerLinkVO deserializes");
    assert_eq!(vo.link_action, Some(MarkerLinkageLinkAction::TriggerAll));
    assert_eq!(vo.from_id, 71243);
    assert_eq!(vo.to_id, 71244);
    assert_eq!(
        vo.group_id.as_deref(),
        Some("851168d7d77d434e93881e504f2a4df1")
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. sys_user.password — `{bcrypt}`-prefixed storage (68 chars)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn sys_user_password_prefix() {
    // Real DB shape (audit): password = `{bcrypt}$2a$...` (68 chars total).
    let stored = bcrypt::generate_storage_password("pw123").expect("generate storage password");
    assert!(
        stored.starts_with("{bcrypt}"),
        "storage uses the {{bcrypt}} prefix"
    );
    assert_eq!(
        stored.len(),
        68,
        "{{bcrypt}}(8) + bcrypt hash(60) = 68 chars, got: {}",
        stored.len()
    );

    // verify_password understands the prefixed form (login path).
    assert!(_utils::bcrypt::verify_password("pw123", &stored).expect("verify ok"));
    assert!(!_utils::bcrypt::verify_password("wrong", &stored).expect("verify ok"));

    // A raw hash without the prefix also verifies (legacy rows).
    let raw = bcrypt::generate_hash("pw123").expect("generate hash");
    assert_eq!(raw.len(), 60, "bcrypt hash alone is 60 chars");
    assert!(bcrypt::verify_password("pw123", &raw).expect("verify raw hash"));

    // A synthetic row shaped exactly like the audited ones.
    let synthetic = format!("{{bcrypt}}{raw}");
    assert_eq!(synthetic.len(), 68);
    assert!(bcrypt::verify_password("pw123", &synthetic).expect("verify synthetic row"));
    assert!(!bcrypt::verify_password("nope", &synthetic).expect("verify synthetic row"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. sys_user_archive.data — dual shape: legacy object array vs JSON string
// ─────────────────────────────────────────────────────────────────────────────

/// Mirrors `functions::system::archive::entity_to_slot_vo`: a `data` column
/// that is a JSON *string* is used as-is; anything else (legacy arrays) is
/// re-serialized back to text.
fn read_archive_data(data: &serde_json::Value) -> String {
    data.as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string(data).unwrap_or_default())
}

/// Mirrors `functions::system::archive::extract_archive` (PUT save body):
/// a `{time, archive, historyIndex}` wrapper yields its `archive` field,
/// otherwise the whole body is serialized as the archive text.
fn extract_archive(body: &serde_json::Value) -> String {
    body.get("archive")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| serde_json::to_string(body).unwrap_or_default())
}

#[test]
fn archive_data_dual_shape() {
    // 1) New-write shape (audit): data = a JSON string whose *content* is the
    //    archive JSON text. Read path must yield the inner text verbatim.
    let inner = r#"{"Data_KYJG":123,"Preference":{},"Time_KYJG":1754280000000}"#;
    let stored_new = serde_json::Value::String(inner.to_string());
    assert_eq!(read_archive_data(&stored_new), inner);

    // 2) Legacy shape (audit, 16 rows): data = object array
    //    `[{"archive":"{...}","time":<str|num>}]` — time mixes strings and
    //    numbers, but the read path never parses it (opaque passthrough).
    let legacy = r#"[{"archive":"{\"a\":1}","time":"1754280000000"},{"archive":"{\"a\":2}","time":1754280000001}]"#;
    let legacy_value: serde_json::Value =
        serde_json::from_str(legacy).expect("legacy array parses");
    let read_back: serde_json::Value = serde_json::from_str(&read_archive_data(&legacy_value))
        .expect("re-serialized data reparses");
    assert_eq!(
        read_back, legacy_value,
        "legacy object-array shape must round-trip through the fallback"
    );

    // 3) ArchiveSlotVo wire keys (Java contract: slotIndex/time/archive).
    let vo = ArchiveSlotVo {
        slot_index: 1,
        time: 1754280000000.0,
        archive: inner.to_string(),
    };
    let json = serde_json::to_value(&vo).expect("serialize ArchiveSlotVo");
    assert_eq!(json["slotIndex"], 1);
    assert_eq!(json["time"], 1754280000000.0_f64);
    assert_eq!(json["archive"], inner);
    assert_eq!(
        serde_json::from_value::<ArchiveSlotVo>(json).expect("deserialize ArchiveSlotVo"),
        vo
    );

    // 4) SysArchiveSlotVo grouped shape: {slotIndex, time, updateTime,
    //    archive: [{time, archive, historyIndex}]} (all_history contract).
    let group = SysArchiveSlotVo {
        version: 0,
        id: 1,
        name: Some("存档 1".into()),
        slot_index: 1,
        create_time: 1754280000000.0,
        update_time: None,
        archive: vec![SysArchiveVo {
            time: 1754280000001.0,
            archive: inner.to_string(),
            history_index: 0,
        }],
    };
    let g = serde_json::to_value(&group).expect("serialize SysArchiveSlotVo");
    assert_eq!(g["slotIndex"], 1);
    assert_eq!(g["archive"][0]["historyIndex"], 0);
    assert_eq!(g["archive"][0]["archive"], inner);

    // 5) PUT body compat: wrapper body extracts `archive`; raw body is
    //    serialized wholesale as the archive text.
    let wrapped = serde_json::json!({"time": 1754280000000.0, "archive": inner, "historyIndex": 0});
    assert_eq!(extract_archive(&wrapped), inner);
    let raw = serde_json::json!({"Data_KYJG": 123});
    let extracted = extract_archive(&raw);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&extracted).expect("raw body round-trips"),
        raw
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. notice.validTimeStart/End — ms number and ISO string both accepted
// ─────────────────────────────────────────────────────────────────────────────

/// Mirrors the documented parse in `functions::api::notice::parse_valid_time`:
/// ms number → timestamp; ISO / naive datetime strings → parsed; garbage → `now`.
fn parse_valid_time(
    value: Option<&serde_json::Value>,
    now: chrono::NaiveDateTime,
) -> Option<chrono::NaiveDateTime> {
    match value {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => Some(if let Some(ms) = v.as_f64() {
            chrono::DateTime::from_timestamp_millis(ms as i64)
                .map(|dt| dt.naive_utc())
                .unwrap_or(now)
        } else if let Some(s) = v.as_str() {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.naive_utc())
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f"))
                .or_else(|_| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S"))
                .unwrap_or(now)
        } else {
            now
        }),
    }
}

#[test]
fn notice_valid_time_parse() {
    // Frontend el-date-picker may serialize validTimeStart/End as either an
    // epoch-ms number or an ISO string; the request model keeps both raw.
    let req_num = r#"{"channel":["COMMON"],"content":"<p>x</p>","title":"t","sortIndex":0,"validTimeStart":1754280000000,"validTimeEnd":null}"#;
    let req_num: NoticeAddRequest =
        serde_json::from_str(req_num).expect("ms-number validTimeStart deserializes");
    assert_eq!(
        req_num.valid_time_start,
        Some(serde_json::json!(1754280000000i64)),
        "ms number stays a JSON number"
    );
    assert_eq!(req_num.valid_time_end, None, "JSON null maps to None");

    let req_iso = r#"{"channel":["COMMON"],"content":"<p>x</p>","title":"t","validTimeStart":"2025-08-04T04:00:00.000Z"}"#;
    let req_iso: NoticeAddRequest =
        serde_json::from_str(req_iso).expect("ISO-string validTimeStart deserializes");
    assert!(
        req_iso.valid_time_start.as_ref().expect("some").is_string(),
        "ISO string stays a JSON string"
    );
    assert_eq!(req_iso.sort_index, 0, "missing sortIndex defaults to 0");

    // Both forms denote the same instant through the documented parse logic.
    let now = chrono::Utc::now().naive_utc();
    let expect = chrono::DateTime::from_timestamp_millis(1754280000000i64)
        .expect("valid epoch ms")
        .naive_utc();
    assert_eq!(
        parse_valid_time(req_num.valid_time_start.as_ref(), now),
        Some(expect)
    );
    assert_eq!(
        parse_valid_time(req_iso.valid_time_start.as_ref(), now),
        Some(expect),
        "ISO string and ms number must parse to the same instant"
    );

    // null / absent → None (column stays NULL); garbage → fallback `now`.
    assert_eq!(parse_valid_time(None, now), None);
    assert_eq!(parse_valid_time(Some(&serde_json::Value::Null), now), None);
    assert_eq!(
        parse_valid_time(Some(&serde_json::json!("not-a-date")), now),
        Some(now)
    );
    assert_eq!(
        parse_valid_time(Some(&serde_json::json!(true)), now),
        Some(now)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. history.editType — numeric strings '0'|'1'|'2'|'3'|'10' from the frontend
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn history_edit_type_string() {
    // Frontend sends editType as numeric *strings* (audit: edit_type values
    // {2,3,10} in the DB); the enum deserializer must accept them.
    for (wire, expected) in [
        ("0", HistoryEditType::Unknown),
        ("1", HistoryEditType::Added),
        ("2", HistoryEditType::Modified),
        ("3", HistoryEditType::Deleted),
        ("10", HistoryEditType::Initialized),
    ] {
        let parsed: HistoryEditType =
            serde_json::from_str(&format!("\"{wire}\"")).expect("numeric string deserializes");
        assert_eq!(parsed, expected);
        // Serializes back to the Java numeric contract (frontend consumes
        // numbers).
        assert_eq!(
            serde_json::to_string(&parsed).expect("serialize"),
            wire,
            "numeric-string wire form round-trips"
        );
    }

    // Through the HistoryItemVO `editType` field (real list-query wire shape).
    let vo: HistoryItemVO = serde_json::from_value(serde_json::json!({
        "version": 0,
        "id": 58502,
        "createTime": 0.0,
        "updateTime": null,
        "creatorId": 46,
        "updaterId": null,
        "delFlag": false,
        "md5": null,
        "ipv4": null,
        "tid": 49232,
        "type": 4,
        "editType": "10",
        "content": "{}"
    }))
    .expect("HistoryItemVO with string editType deserializes");
    assert_eq!(vo.edit_type, HistoryEditType::Initialized);
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. item.iconStyleType — numeric 0-3 from the frontend
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn item_icon_style_numeric() {
    // Frontend sends iconStyleType as numbers (audit: item rows with
    // icon_style_type values {0,1,3}).
    for (num, expected) in [
        (0, IconStyleType::Default),
        (1, IconStyleType::NoBorder),
        (2, IconStyleType::LikeOculus),
        (3, IconStyleType::Oculus),
    ] {
        let parsed: IconStyleType =
            serde_json::from_str(&num.to_string()).expect("numeric iconStyleType deserializes");
        assert_eq!(parsed, expected);
        assert_eq!(
            serde_json::to_string(&parsed).expect("serialize"),
            num.to_string(),
            "numeric form round-trips"
        );
    }

    // Through the ItemRequest model (real audited row: icon_style_type=3).
    let req: ItemRequest = serde_json::from_value(serde_json::json!({
        "name": "散失的风神瞳",
        "areaId": 6,
        "defaultRefreshTime": 0,
        "defaultContent": null,
        "defaultCount": 1,
        "iconId": 4,
        "iconStyleType": 3,
        "hiddenFlag": 0,
        "sortIndex": 99,
        "specialFlag": null
    }))
    .expect("ItemRequest with numeric iconStyleType deserializes");
    assert_eq!(req.icon_style_type, IconStyleType::Oculus);
}

// ─────────────────────────────────────────────────────────────────────────────
// 11. marker.extra — `{"underground":{...}}` opaque Value passthrough
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn marker_extra_underground() {
    // Real DB shape (audit, non-empty marker.extra): the backend treats extra
    // as an opaque serde_json::Value — it must survive MarkerVO serialization
    // structurally identical and deserialize back unchanged.
    let extra = serde_json::json!({
        "underground": {
            "is_underground": true,
            "region_levels": ["ECHO_CHILD_SETTLEMENT"]
        }
    });

    let vo = MarkerVO {
        version: 0,
        id: 66181,
        create_time: 0.0,
        update_time: None,
        creator_id: None,
        updater_id: None,
        del_flag: false,
        marker_stamp: None,
        marker_title: Some("风滚草".into()),
        position: "-7507.75,2244.25".into(),
        content: Some("风滚草".into()),
        picture: Some(String::new()),
        marker_creator_id: 28,
        picture_creator_id: Some(0),
        video_path: Some(String::new()),
        refresh_time: 43200000,
        hidden_flag: HiddenFlag::Visible,
        extra: Some(extra.clone()),
        item_list: vec![MarkerItemLinkVo {
            item_id: 2992,
            count: 1,
            icon_tag: None,
            icon_id: 0,
        }],
        linkage_id: None,
    };

    let json = serde_json::to_value(&vo).expect("serialize MarkerVO");
    assert_eq!(
        json["extra"], extra,
        "extra must pass through as an opaque JSON object"
    );
    assert_eq!(
        json["extra"]["underground"]["is_underground"], true,
        "nested underground payload is preserved"
    );

    let back: MarkerVO = serde_json::from_value(json).expect("deserialize MarkerVO");
    assert_eq!(back.extra, Some(extra));

    // extra NULL (141 audited rows) stays null on the wire.
    let no_extra = MarkerVO { extra: None, ..vo };
    let j = serde_json::to_value(&no_extra).expect("serialize MarkerVO without extra");
    assert!(j["extra"].is_null(), "missing extra serializes as null");
}
