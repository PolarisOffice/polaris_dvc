//! Rule schema compatible with upstream `sample/jsonFullSpec.json`.
//!
//! The upstream format is permissive — only a subset of fields are used per
//! spec file. We mirror that with `#[serde(default)]` throughout.

pub mod loader;
pub mod schema;

pub use loader::{load_spec, SpecLoadError};
pub use schema::{
    BgFillSpec, BorderRule, BulletSpec, CharShape, ColorValue, LevelType, OutlineShape,
    ParaNumBullet, ParaShape, Permission, Range64, RuleSpec, SpecialCharacter, StringList,
    TableCaptionSpec, TableMarginSpec, TableSizeSpec, TableSpec,
};
