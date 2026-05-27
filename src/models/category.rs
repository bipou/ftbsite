use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 分类：name 为多语言字段，key 即语言代码（zh/en/jp…），新增语言零代码改动
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: String,
    pub name: HashMap<String, String>,
    pub level: u8,
    pub pinned: bool,
}
