//! The lift model: the load lines and the tackle, and the weight they add up
//! to. A port of `Model.kt` from the Android source.
//!
//! The line id is carried across every edit for the same reason it is on the
//! phone: a text field needs a stable key so a row being typed into is not
//! re-associated with a different line when an earlier one is deleted.

use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

use crate::calc::{self, DEFAULT_DENSITY_KG_M3};
use crate::calc::LoadType;

static NEXT_LINE_ID: AtomicU64 = AtomicU64::new(1);

/// The next id a new load or tackle line gets.
pub fn next_line_id() -> u64 {
    NEXT_LINE_ID.fetch_add(1, Ordering::Relaxed)
}

/// The measurements one load line carries, regardless of which type is active.
pub trait LoadMeasurements {
    fn qty(&self) -> i64;
    fn weight_kg(&self) -> f64;
    fn kg_per_metre(&self) -> f64;
    fn length_m(&self) -> f64;
    fn volume_m3(&self) -> f64;
    fn density_kg_m3(&self) -> f64;
}

/// One line of the lift.
///
/// Every measurement is held whether or not the current type uses it.
/// Switching a line from per-metre to per-volume and back must not lose what
/// was typed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LoadItem {
    pub id: u64,
    pub name: String,
    pub load_type: LoadType,
    pub qty: i64,
    pub weight_kg: f64,
    pub kg_per_metre: f64,
    pub length_m: f64,
    pub volume_m3: f64,
    pub density_kg_m3: f64,
}

impl Default for LoadItem {
    fn default() -> Self {
        Self {
            id: next_line_id(),
            name: String::new(),
            load_type: LoadType::Fixed,
            qty: 1,
            weight_kg: 0.0,
            kg_per_metre: 0.0,
            length_m: 0.0,
            volume_m3: 0.0,
            density_kg_m3: DEFAULT_DENSITY_KG_M3,
        }
    }
}

impl LoadMeasurements for LoadItem {
    fn qty(&self) -> i64 {
        self.qty
    }
    fn weight_kg(&self) -> f64 {
        self.weight_kg
    }
    fn kg_per_metre(&self) -> f64 {
        self.kg_per_metre
    }
    fn length_m(&self) -> f64 {
        self.length_m
    }
    fn volume_m3(&self) -> f64 {
        self.volume_m3
    }
    fn density_kg_m3(&self) -> f64 {
        self.density_kg_m3
    }
}

impl LoadItem {
    /// The line's total, or 0.0 while it is not yet enterable.
    ///
    /// A [`calc::CalcException`] becomes zero rather than propagating: a line
    /// being typed into is incomplete most of the time, and treating that as a
    /// failure would mean the totals spent the whole of data entry reporting an
    /// error.
    pub fn weight(&self) -> f64 {
        calc::load_item_weight(self.load_type, self).unwrap_or(0.0)
    }
}

/// One line of lifting tackle: shackles, strops, the block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TackleItem {
    pub id: u64,
    pub description: String,
    pub qty: i64,
    pub weight_kg: f64,
}

impl Default for TackleItem {
    fn default() -> Self {
        Self {
            id: next_line_id(),
            description: String::new(),
            qty: 1,
            weight_kg: 0.0,
        }
    }
}

impl TackleItem {
    pub fn subtotal(&self) -> f64 {
        self.weight_kg * self.qty as f64
    }
}

/// The Overall Weight figures, in the order the result card renders them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LiftTotals {
    pub load: f64,
    pub tackle: f64,
    pub gross: f64,
    pub required_chart_capacity: f64,
    /// Whether the minimum crane rating is exceeded.
    ///
    /// Kept as the source writes it — and as written it is **always false**:
    /// the required capacity is `gross / 0.75`, which is greater than `gross`
    /// for any positive gross. The source has the same arithmetic, so the flag
    /// stays; changing the rule here would make this the only place the two
    /// differ.
    pub over_limit: bool,
}

impl LiftTotals {
    /// Sum a lift's lines.
    ///
    /// `required_chart_capacity` is the one figure here that is not a sum of
    /// the others: it is the crane capacity the lift needs under the 75% rule.
    pub fn of(load_items: &[LoadItem], tackle_items: &[TackleItem]) -> LiftTotals {
        let load: f64 = load_items.iter().map(LoadItem::weight).sum();
        let tackle: f64 = tackle_items.iter().map(TackleItem::subtotal).sum();
        let gross = calc::gross_lift_weight(load, tackle).unwrap_or(0.0);
        let required = calc::required_chart_capacity(gross, calc::CRANE_USAGE_RATIO).unwrap_or(0.0);
        LiftTotals {
            load,
            tackle,
            gross,
            required_chart_capacity: required,
            over_limit: required > 0.0 && gross > required,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_survive_a_copy() {
        let first = LoadItem::default();
        let second = LoadItem::default();
        assert_ne!(first.id, second.id);
        let copy = first.clone();
        assert_eq!(first.id, copy.id);
    }

    #[test]
    fn a_half_typed_line_weighs_zero() {
        let mut item = LoadItem::default();
        item.load_type = LoadType::PerVolume;
        // density defaulted, volume still zero
        assert_eq!(item.weight(), 0.0);
    }

    #[test]
    fn totals_add_load_and_tackle() {
        let mut load = LoadItem::default();
        load.weight_kg = 1000.0;
        let mut tackle = TackleItem::default();
        tackle.weight_kg = 100.0;
        tackle.qty = 2;
        let totals = LiftTotals::of(&[load], &[tackle]);
        assert_eq!(totals.load, 1000.0);
        assert_eq!(totals.tackle, 200.0);
        assert_eq!(totals.gross, 1200.0);
        assert_eq!(totals.required_chart_capacity, 1600.0);
        assert!(!totals.over_limit);
    }

    #[test]
    fn totals_size_the_crane_at_seventy_five_percent() {
        let load = LoadItem {
            weight_kg: 1500.0,
            ..LoadItem::default()
        };
        let tackle = TackleItem {
            weight_kg: 50.0,
            qty: 2,
            ..TackleItem::default()
        };
        let totals = LiftTotals::of(&[load], &[tackle]);
        assert!((totals.load - 1500.0).abs() < 1e-9);
        assert!((totals.tackle - 100.0).abs() < 1e-9);
        assert!((totals.gross - 1600.0).abs() < 1e-9);
        assert!((totals.required_chart_capacity - 1600.0 / 0.75).abs() < 1e-9);
    }

    #[test]
    fn the_required_capacity_is_always_above_the_gross_so_the_over_flag_stays_off() {
        let load = LoadItem {
            weight_kg: 9000.0,
            ..LoadItem::default()
        };
        let totals = LiftTotals::of(&[load], &[]);
        assert!(!totals.over_limit);
    }
}
