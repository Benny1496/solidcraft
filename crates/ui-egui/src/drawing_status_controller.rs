//! Read-only status/inspect refresh coordinator for D07/D26 M1-A.
//! The host performs actual engine queries; this module never issues commands.
use crate::drawing_dimension_status::{DrawingAttention, from_status};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshRequest {
    pub generation: u64,
    pub inspect_ids: Vec<String>,
}

#[derive(Default)]
pub struct DrawingStatusController {
    generation: u64,
    status: Option<Value>,
    inspections: BTreeMap<String, Result<Value, String>>,
    requested: BTreeSet<String>,
}

impl DrawingStatusController {
    /// Invalidate on document switch, edit, undo/redo, drawing update, or view changes.
    /// Late responses from previous generations must be ignored.
    pub fn invalidate(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.status = None;
        self.inspections.clear();
        self.requested.clear();
        self.generation
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Accept a completed drawing.status result only for the current generation.
    pub fn accept_status(&mut self, generation: u64, status: Value) -> bool {
        if generation != self.generation {
            return false;
        }
        self.status = Some(status);
        self.inspections.clear();
        self.requested.clear();
        true
    }

    /// Ask for inspect only on dimensions requiring attention, never every dimension.
    /// Call once after accepting a status snapshot; repeated calls do not refetch.
    pub fn pending_inspections(&mut self) -> RefreshRequest {
        let inspect_ids = self
            .status
            .as_ref()
            .map(from_status)
            .map(|a| a.dimensions.into_iter().filter(|d| d.needs_attention()).map(|d| d.id).filter(|id| self.requested.insert(id.clone())).collect())
            .unwrap_or_default();
        RefreshRequest { generation: self.generation, inspect_ids }
    }

    /// A failed inspect remains visible as an error until the next invalidation.
    pub fn accept_inspection(&mut self, generation: u64, dimension: &str, result: Result<Value, String>) -> bool {
        if generation != self.generation || !self.requested.contains(dimension) {
            return false;
        }
        self.inspections.insert(dimension.to_owned(), result);
        true
    }

    /// No snapshot is shown after invalidation until the fresh status arrives.
    pub fn snapshot(&self) -> Option<DrawingAttention> {
        self.status.as_ref().map(|s| from_status(s).with_inspections(&self.inspections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn inspect_only_attention_and_deduplicate_requests() {
        let mut c = DrawingStatusController::default();
        assert!(c.accept_status(0, json!({"views":[],"dimensions":[{"dimension":"ok","status":"ok"},{"dimension":"lost","status":"lost"},{"dimension":"stale","status":"ok","out_of_date":true}]})));
        let request = c.pending_inspections();
        assert_eq!(request.inspect_ids, vec!["lost", "stale"]);
        assert!(c.pending_inspections().inspect_ids.is_empty());
    }

    #[test]
    fn invalidation_rejects_late_status_and_inspection() {
        let mut c = DrawingStatusController::default();
        assert!(c.accept_status(0, json!({"views":[],"dimensions":[{"dimension":"d","status":"lost"}]})));
        let old = c.pending_inspections();
        c.invalidate();
        assert!(!c.accept_status(old.generation, json!({"views":[],"dimensions":[]})));
        assert!(!c.accept_inspection(old.generation, "d", Ok(json!({"view":"v"}))));
        assert!(c.snapshot().is_none());
        assert!(c.accept_status(c.generation(), json!({"views":[],"dimensions":[]})));
        assert_eq!(c.snapshot().unwrap().dimensions.len(), 0);
    }

    #[test]
    fn inspect_errors_and_unknown_sheets_are_not_hidden() {
        let mut c = DrawingStatusController::default();
        assert!(c.accept_status(
            0,
            json!({"views":[{"view":"v","sheet":"sheet","status":"current"}],"dimensions":[{"dimension":"d","status":"ambiguous"}]})
        ));
        let request = c.pending_inspections();
        assert_eq!(request.inspect_ids, vec!["d"]);
        assert!(c.accept_inspection(request.generation, "d", Err("inspect failed".into())));
        assert_eq!(c.snapshot().unwrap().dimensions[0].inspect_error.as_deref(), Some("inspect failed"));
        assert!(c.accept_inspection(request.generation, "d", Ok(json!({"view":"v"}))));
        assert_eq!(c.snapshot().unwrap().dimensions[0].sheet.as_deref(), Some("sheet"));
    }
}
