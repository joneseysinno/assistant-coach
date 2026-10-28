//! What the core needs from a sport pack (D-005). The basketball pack implements this in
//! `ac-sport-basketball`; the core never names a basketball thing itself.

use indexmap::IndexMap;

use crate::model::{RoleEdit, Rule};
use crate::val::Val;

pub trait Pack {
    /// Event vocabulary (S1 §4).
    fn is_event(&self, id: &str) -> bool;
    /// Named regions (S3 §5, §7) and court regions.
    fn is_named_region(&self, id: &str) -> bool;
    /// Learning-target taxonomy (S6 §3).
    fn is_target(&self, id: &str) -> bool;
    /// Read catalog (S6 §5).
    fn is_read(&self, id: &str) -> bool;
    /// Expand the game's `base_rules` preset into rules with ids `base.*` (S5 §5).
    fn base_rules(&self, spec: &IndexMap<String, Val>) -> Vec<Rule>;
    /// Compile one role edit (S5 §2.1) into ordinary rules.
    fn role_edit_rules(&self, constraint_id: &str, edit: &RoleEdit) -> Vec<Rule>;
}
