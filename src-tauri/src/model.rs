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
    /// The lift this line belongs to. Lines with the same name are lifted
    /// together and add up; the plan's overall weight is the heaviest lift.
    pub lift: String,
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
            lift: "Lift 1".to_string(),
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
    /// The lift this rigging rides with. Blank is shared: it counts on every
    /// lift, so a line entered before the lifts were named is never dropped.
    pub lift: String,
    pub qty: i64,
    pub weight_kg: f64,
}

impl Default for TackleItem {
    fn default() -> Self {
        Self {
            id: next_line_id(),
            description: String::new(),
            lift: "Lift 1".to_string(),
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

/// One lift's own total: the lines that are lifted together.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LiftGroup {
    pub name: String,
    pub weight: f64,
    /// The rigging assigned to this lift.
    #[serde(default)]
    pub tackle: f64,
}

impl LiftGroup {
    /// What this lift puts on the hook.
    pub fn total(&self) -> f64 {
        self.weight + self.tackle
    }
}

/// The Overall Weight figures, in the order the result card renders them.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LiftTotals {
    /// The heaviest lift's total, not the sum of every line on the sheet.
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
    /// Every lift and its own total, in the order the lines were entered.
    #[serde(default)]
    pub lifts: Vec<LiftGroup>,
}

/// A line's lift name, with the blank name read as the first lift.
fn lift_name(raw: &str) -> &str {
    let name = raw.trim();
    if name.is_empty() {
        "Lift 1"
    } else {
        name
    }
}

impl LiftTotals {
    /// Sum a lift's lines.
    ///
    /// `required_chart_capacity` is the one figure here that is not a sum of
    /// the others: it is the crane capacity the lift needs under the 75% rule.
    pub fn of(load_items: &[LoadItem], tackle_items: &[TackleItem]) -> LiftTotals {
        // Lines under one lift are lifted together and add up. Different lifts
        // are alternatives, so the plan is sized by the heaviest one rather
        // than by everything on the sheet.
        let mut lifts: Vec<LiftGroup> = Vec::new();
        for item in load_items {
            let name = lift_name(&item.lift);
            match lifts.iter_mut().find(|group| group.name == name) {
                Some(group) => group.weight += item.weight(),
                None => lifts.push(LiftGroup {
                    name: name.to_string(),
                    weight: item.weight(),
                    tackle: 0.0,
                }),
            }
        }

        // Named tackle rides with its own lift. Unnamed tackle, or tackle that
        // names no lift in this plan, is shared: it is counted on every lift,
        // so a typo adds weight rather than quietly leaving it out.
        let mut shared = 0.0;
        for item in tackle_items {
            let named = !item.lift.trim().is_empty();
            let name = lift_name(&item.lift);
            match lifts.iter_mut().find(|group| group.name == name) {
                Some(group) if named => group.tackle += item.subtotal(),
                _ => shared += item.subtotal(),
            }
        }
        for lift in &mut lifts {
            lift.tackle += shared;
        }

        let governing = lifts
            .iter()
            .max_by(|a, b| a.total().total_cmp(&b.total()));
        let (load, tackle) = match governing {
            Some(lift) => (lift.weight, lift.tackle),
            None => (0.0, shared),
        };
        let gross = calc::gross_lift_weight(load, tackle).unwrap_or(0.0);
        let required = calc::required_chart_capacity(gross, calc::CRANE_USAGE_RATIO).unwrap_or(0.0);
        LiftTotals {
            load,
            tackle,
            gross,
            required_chart_capacity: required,
            over_limit: required > 0.0 && gross > required,
            lifts,
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
        let item = LoadItem {
            load_type: LoadType::PerVolume,
            // density defaulted, volume still zero
            ..LoadItem::default()
        };
        assert_eq!(item.weight(), 0.0);
    }

    #[test]
    fn totals_add_load_and_tackle() {
        let load = LoadItem {
            weight_kg: 1000.0,
            ..LoadItem::default()
        };
        let tackle = TackleItem {
            weight_kg: 100.0,
            qty: 2,
            ..TackleItem::default()
        };
        let totals = LiftTotals::of(&[load], &[tackle]);
        assert_eq!(totals.load, 1000.0);
        assert_eq!(totals.tackle, 200.0);
        assert_eq!(totals.gross, 1200.0);
        assert_eq!(totals.required_chart_capacity, 1600.0);
        assert!(!totals.over_limit);
    }

    #[test]
    fn the_overall_weight_is_the_heaviest_lift() {
        let steel = LoadItem {
            lift: "Steel".to_string(),
            weight_kg: 800.0,
            ..LoadItem::default()
        };
        let concrete = LoadItem {
            lift: "Concrete".to_string(),
            weight_kg: 1500.0,
            ..LoadItem::default()
        };
        let bucket = LoadItem {
            lift: "Concrete".to_string(),
            weight_kg: 80.0,
            ..LoadItem::default()
        };
        let totals = LiftTotals::of(&[steel, concrete, bucket], &[]);
        assert!((totals.load - 1580.0).abs() < 1e-9);
        assert_eq!(totals.lifts.len(), 2);
        assert_eq!(totals.lifts[0].name, "Steel");
        assert!((totals.lifts[1].weight - 1580.0).abs() < 1e-9);
    }

    #[test]
    fn tackle_rides_with_its_own_lift() {
        let steel = LoadItem {
            lift: "Steel".to_string(),
            weight_kg: 1000.0,
            ..LoadItem::default()
        };
        let concrete = LoadItem {
            lift: "Concrete".to_string(),
            weight_kg: 950.0,
            ..LoadItem::default()
        };
        let rig = TackleItem {
            lift: "Concrete".to_string(),
            weight_kg: 50.0,
            qty: 2,
            ..TackleItem::default()
        };
        let totals = LiftTotals::of(&[steel, concrete], &[rig]);
        assert!((totals.load - 950.0).abs() < 1e-9);
        assert!((totals.tackle - 100.0).abs() < 1e-9);
        assert!((totals.gross - 1050.0).abs() < 1e-9);
    }

    #[test]
    fn shared_tackle_counts_on_the_lift_it_rides_with() {
        let steel = LoadItem {
            lift: "Steel".to_string(),
            weight_kg: 1000.0,
            ..LoadItem::default()
        };
        let concrete = LoadItem {
            lift: "Concrete".to_string(),
            weight_kg: 500.0,
            ..LoadItem::default()
        };
        let rig = TackleItem {
            lift: String::new(),
            weight_kg: 100.0,
            ..TackleItem::default()
        };
        let totals = LiftTotals::of(&[steel, concrete], &[rig]);
        assert!((totals.load - 1000.0).abs() < 1e-9);
        assert!((totals.tackle - 100.0).abs() < 1e-9);
        assert!((totals.gross - 1100.0).abs() < 1e-9);
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
