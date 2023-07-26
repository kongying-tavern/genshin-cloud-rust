use moka::future::Cache;

use crate::models::{
    area::Model as Area, history::Model as History, icon::Model as Icon,
    icon_type::Model as IconType, icon_type_link::Model as IconTypeLink, item::Model as Item,
    item_area_public::Model as ItemAreaPublic, item_type::Model as ItemType,
    item_type_link::Model as ItemTypeLink, marker::Model as Marker,
    marker_item_link::Model as MarkerItemLink, marker_punctuate::Model as MarkerPunctuate,
    sys_role::Model as SysRole, sys_user::Model as SysUser,
    sys_user_archive::Model as SysUserArchive, sys_user_role_link::Model as SysUserRoleLink,
    tag::Model as Tag, tag_type::Model as TagType, tag_type_link::Model as TagTypeLink,
};

pub struct CacheObject {
    pub area: Cache<i64, Area>,
    pub history: Cache<i64, History>,
    pub icon: Cache<i64, Icon>,
    pub icon_type: Cache<i64, IconType>,
    pub icon_type_link: Cache<i64, IconTypeLink>,
    pub item: Cache<i64, Item>,
    pub item_area_public: Cache<i64, ItemAreaPublic>,
    pub item_type: Cache<i64, ItemType>,
    pub item_type_link: Cache<i64, ItemTypeLink>,
    pub marker: Cache<i64, Marker>,
    pub marker_item_link: Cache<i64, MarkerItemLink>,
    pub marker_punctuate: Cache<i64, MarkerPunctuate>,
    pub sys_role: Cache<i64, SysRole>,
    pub sys_user: Cache<i64, SysUser>,
    pub sys_user_archive: Cache<i64, SysUserArchive>,
    pub sys_user_role_link: Cache<i64, SysUserRoleLink>,
    pub tag: Cache<i64, Tag>,
    pub tag_type: Cache<i64, TagType>,
    pub tag_type_link: Cache<i64, TagTypeLink>,
}

impl Default for CacheObject {
    fn default() -> Self {
        Self {
            area: Cache::new(10_000),
            history: Cache::new(10_000),
            icon: Cache::new(10_000),
            icon_type: Cache::new(10_000),
            icon_type_link: Cache::new(10_000),
            item: Cache::new(10_000),
            item_area_public: Cache::new(10_000),
            item_type: Cache::new(10_000),
            item_type_link: Cache::new(10_000),
            marker: Cache::new(10_000),
            marker_item_link: Cache::new(10_000),
            marker_punctuate: Cache::new(10_000),
            sys_role: Cache::new(10_000),
            sys_user: Cache::new(10_000),
            sys_user_archive: Cache::new(10_000),
            sys_user_role_link: Cache::new(10_000),
            tag: Cache::new(10_000),
            tag_type: Cache::new(10_000),
            tag_type_link: Cache::new(10_000),
        }
    }
}
