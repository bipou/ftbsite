use crate::models::{Category, PageInfo, Topic};
use serde::{Deserialize, Serialize};

/// 赔率记录（footballs.lines 内嵌数组）—— 仅境外版
#[cfg(feature = "oth")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Line {
    pub win: f32,
    pub draw: f32,
    pub loss: f32,
    pub created_at: String,
}

/// 计算记录（footballs.calcs 内嵌数组）—— 仅境外版
#[cfg(feature = "oth")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Calc {
    pub s: String,
    pub wdl: String,
    pub tg: String,
    pub gd: String,
    pub created_at: String,
}

// ── 阵容 ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct LineupPlayer {
    pub number: u8,
    pub name: String,
    pub position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TeamLineup {
    pub formation: String,
    pub coach: Option<String>,
    pub starters: Vec<LineupPlayer>,
    pub substitutes: Vec<LineupPlayer>,
}

// ── 事件时间线 ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballEvent {
    pub minute: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<u8>,
    #[serde(rename = "type")]
    pub event_type: String,
    pub player: String,
    pub team: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assist: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_out: Option<String>,
    pub note: String,
}

// ── 技术统计 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SideStats {
    pub home: f32,
    pub away: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SideStatsInt {
    pub home: u16,
    pub away: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballStats {
    pub possession: SideStats,
    pub shots: SideStatsInt,
    pub shots_on_target: SideStatsInt,
    pub corners: SideStatsInt,
    pub fouls: SideStatsInt,
    pub offsides: SideStatsInt,
    pub yellow_cards: SideStatsInt,
    pub red_cards: SideStatsInt,
    pub passes: SideStatsInt,
    pub pass_accuracy: SideStats,
}

// ── 分析文章 ────────────────────────────────────────────────────────────

/// 分析文章（footballs_analyses 表）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct FootballAnalysis {
    pub id: String,
    pub football_id: String,
    /// AI 分析为 None，用户分析指向 users 表
    pub user_id: Option<String>,
    /// 摘要，落库 footballs 则为 None
    #[serde(default)]
    pub summary: Option<String>,
    /// Markdown 原文（落库的唯一文本字段）
    pub content: String,
    /// 渲染后的 HTML（服务端转换，不落库）
    pub content_html: String,
    /// AI 模型名，用户分析为空串
    pub ai_model: String,
    /// 0=草稿 1=发布 -1=删除
    pub status: i8,
}

/// A football match with all resolved relations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Football {
    pub id: String,
    pub category_id: String,
    #[serde(default)]
    pub season: Option<String>,
    #[serde(default)]
    pub home_team: Option<String>,
    #[serde(default)]
    pub away_team: Option<String>,
    /// Formatted "MM-DD HH:MM" UTC
    #[serde(default)]
    pub kick_off_at_mdhm: Option<String>,
    /// Formatted "MM-DD HH:MM" UTC+8
    #[serde(default)]
    pub kick_off_at_mdhm8: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub hits: u64,
    /// Status: 4=both,3=picks,2=hot,1=published,0=submit,-1=draft,-2=deleted
    pub status: i8,
    /// 正式赛果——比分，如 "3:1"（footballs 表直存，未完成则为 None）
    pub result_s: Option<String>,
    /// 正式赛果——胜平负（3=胜 / 1=平 / 0=负）
    pub result_wdl: Option<u8>,
    /// 正式赛果——总进球（≥0）
    pub result_tg: Option<u8>,
    /// 正式赛果——净胜球（可负）
    pub result_gd: Option<i8>,
    #[cfg(feature = "oth")]
    pub il_odds: Vec<Line>,
    #[cfg(feature = "oth")]
    pub all_odds: Vec<Line>,
    #[cfg(feature = "oth")]
    pub il_calcs: Vec<Calc>,
    #[cfg(feature = "oth")]
    pub all_calcs: Vec<Calc>,
    pub home_lineup: Option<TeamLineup>,
    pub away_lineup: Option<TeamLineup>,
    #[serde(default)]
    pub events: Vec<FootballEvent>,
    pub stats: Option<FootballStats>,
    #[serde(default)]
    pub summary: Option<String>,
    pub analyses: Vec<FootballAnalysis>,
    /// 赛事趣名或文章标题
    pub article_title: Option<String>,
    /// 0=文章 >0=赛事
    pub ana_type: u8,
    pub category: Option<Category>,
    pub topics: Vec<Topic>,
}

impl Football {
    pub fn title(&self) -> String {
        match (&self.home_team, &self.away_team) {
            (Some(h), Some(a)) => [h.as_str(), " vs ", a.as_str()].join(""),
            _ => self.article_title.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FootballsResult {
    pub page_info: PageInfo,
    pub items: Vec<Football>,
}
