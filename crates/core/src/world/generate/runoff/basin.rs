use crate::{
    unwrap,
    util::unit::{Meter, Meter3},
    world::{
        generate::{
            runoff::pattern::{
                RunoffDestination, RunoffDestinationMap, RunoffPattern,
            },
            TileBuilder,
        },
        hex::{Cluster, HasHexPosition, TilePoint, TilePointIndexMap},
        Tile,
    },
};
use assert_approx_eq::assert_approx_eq;
use derive_more::{Display, From, Into};
use fnv::FnvBuildHasher;
use log::trace;
use std::{
    collections::{HashMap, HashSet},
    default::Default,
    iter,
};

/// A key that we use to lookup basins. This is just a [TilePoint], but this
/// wrapper semantically implies that the key 1. refers to a basin but 2. has
/// **not** been resolved yet. Basin keys can alias other keys, which is needed
/// when we join two basins (the key of one basin stays the primary while the
/// other aliases the primary, so that anything referencing the old basin
/// remains valid). This can be resolved into a [ResolvedBasinKey] using
/// [Basins::resolve].
///
/// These keys are only used within this module, everything externally just uses
/// [TilePoint].
#[derive(Copy, Clone, Debug, Display, From, Hash, PartialEq, Eq)]
struct BasinKey(TilePoint);

impl BasinKey {
    /// **Dangerously** upgrade this key into a resolved key. This should only
    /// be used if you've actually resolved the key yourself!
    fn danger_upgrade(self) -> ResolvedBasinKey {
        ResolvedBasinKey(self.0)
    }
}

/// A basin key that has been resolved so that it is guaranteed to not be an
/// alias. Created by [Basins::resolve].
#[derive(Copy, Clone, Debug, Display, Into, Hash, PartialEq, Eq)]
struct ResolvedBasinKey(TilePoint);

impl ResolvedBasinKey {
    /// Convert this key into a possibly-alias key.
    fn downgrade(self) -> BasinKey {
        BasinKey(self.0)
    }
}

/// A basin is a cluster of tiles that are all covered by the same collection of
/// runoff. You can think of a basin as basically just a "lake", except that
/// a basin is **not necessarily** guaranteed to become a lake, depending on the
/// rules applied in
/// [WaterFeatureGenerator](crate::world::generate::water_feature::
/// WaterFeatureGenerator).
#[derive(Clone, Debug)]
pub struct Basin {
    /// The primary key for this basin. Each basin is initialized with a single
    /// terminal tile, and that will always remain the primary key.
    key: ResolvedBasinKey,
    /// All the terminals contains within the basin. This starts as just one
    /// element (the initial terminal in the basin), but this will grow for
    /// each call to [Self::join].
    terminals: Vec<TilePoint>,
    /// The list of all tiles in the basin.
    tiles: Cluster<()>,
    /// The total amount of runoff held in this basin. This runoff should
    /// **never be duplicated** with any runoff that exists on any tile in the
    /// basin. In fact, any tile in the basin should always have 0 runoff on
    /// it. During basin calculation, all runoff exists here in the basin
    /// struct, rather than on any individual tile, for ease of access.
    /// Once basin calculation is complete, we can spread the runoff around
    /// to all the constituent tiles in the basin.
    runoff: Meter3,
    /// The sum elevation of all tiles in the basin. Used to calculate average
    /// elevation in [Self::runoff_elevation].
    total_elev: Meter,
    /// A list of all other basins that have overflowed into this one in the
    /// past. We need to keep track of this so that we can identified when two
    /// basins need to be joined. E.g. if basin A overflows into B, then later
    /// B tries to overflow back into A, those two need to be joined.
    prev_donors: HashSet<BasinKey, FnvBuildHasher>,
}

impl Basin {
    /// Initialize a new basin around the given terminal tile. This will remove
    /// any runoff on the tile and move it into the basin's runoff storage.
    pub fn new(terminal: &mut TileBuilder) -> Self {
        let term_pos = terminal.position();
        Self {
            key: ResolvedBasinKey(term_pos),
            terminals: vec![term_pos],
            tiles: Cluster::new(iter::once((term_pos, ())).collect()),
            total_elev: terminal.elevation(),
            runoff: terminal.clear_runoff(),
            prev_donors: HashSet::default(),
        }
    }

    pub fn tiles(&self) -> &Cluster<()> {
        &self.tiles
    }

    /// All terminal tiles in this basin. This will start as just the initial
    /// tile, but every time a new basin is joined into this one, the terminal
    /// list will grow.
    pub fn terminals(&self) -> &[TilePoint] {
        &self.terminals
    }

    /// Get the primary key of this basin. A basin is keyed by the position of
    /// its original tile.
    pub fn key(&self) -> TilePoint {
        self.key.into()
    }

    /// Get the amount of runoff held in this basin
    pub fn runoff(&self) -> Meter3 {
        self.runoff
    }

    /// Add a tile to this basin.
    pub fn add_tile(&mut self, tile: &TileBuilder) {
        self.tiles.insert(tile.position(), ());
        self.total_elev += tile.elevation();
    }

    /// "Runoff elevation" is elevation+runoff for any tile, i.e. the elevation
    /// of the surface of the water. Since water is a liquid[citation needed],
    /// it will spread evenly across the basin which  means all tiles in the
    /// basin will have the same runoff elevation.. runoff_elevation: Meter,
    pub fn runoff_elevation(&self) -> Meter {
        let len = self.tiles.tiles().len() as f64;
        self.total_elev / len + self.runoff / (Tile::AREA * len)
    }

    /// Overflow **into** this basin. `donor` is the basin that is giving us
    /// the runoff.
    pub fn overflow(&mut self, donor: TilePoint, overflow: Meter3) {
        self.runoff += overflow;

        // Make a note of what other basins have overflowed into us. Later on,
        // if we try to overflow back the donor basin, we know to join the
        // basins instead.
        self.prev_donors.insert(donor.into());
    }

    /// Join this basin with another one. This will mutate `self` such that
    /// all of the other basin's tiles are now part of `self`, and all other
    /// fields will be updated as necessary, in order to form one big basin.
    pub fn join(&mut self, other: Self) {
        self.terminals.extend(other.terminals);
        // Mild hack here since we're actually breaking Cluster's contiguity
        // requirement, since the two basins we're joining will be disjoint
        // (for now). This should be resolved the next time this basin gets
        // spread out, since the next candidate tile will be the one that joins
        // the two basins. Until then though, we are temporarily in violation.
        self.tiles.join(other.tiles);

        self.total_elev += other.total_elev;
        self.runoff += other.runoff;

        self.prev_donors.extend(other.prev_donors);
        let terminals = self.terminals.as_slice();
        self.prev_donors.retain(|key| !terminals.contains(&key.0));
    }

    /// Calculate runoff distributions for some tile, but do not allow the
    /// tile to drain to this basin at all. This is useful if this basin has
    /// risen to the level of the tile in question, and you want to force it to
    /// push water elsewhere before adding it to this basin. The returned map
    /// is guaranteed to have a sum equal to the input runoff to this function,
    /// **unless* the map is empty, which would indicate that the tile doesn't
    /// distribute anywhere outside this basin.
    ///
    /// ## Params
    /// - `runoff_pattern` - The pattern of the tile to distribute from. The
    ///   tile should **not** be in this basin
    /// - `runoff_to_distribute` - The amount of runoff to be distributed **out
    ///   of this basin** based on the tile's runoff pattern. Remember, the goal
    ///   here is to push water out of our basin because we've overflowed
    pub fn distribute_elsewhere(
        &mut self,
        runoff_pattern: &RunoffPattern,
        runoff_to_distribute: Meter3,
    ) -> HashMap<RunoffDestination, Meter3, FnvBuildHasher> {
        // Figure out where the runoff will go if we exclude all the terminals
        // that fall within this basin
        let filtered_destinations =
            runoff_pattern.filter_destinations(self.terminals());

        // Calculate how much runoff we'll send to each destination
        let distributed: RunoffDestinationMap<Meter3> = filtered_destinations
            .iter()
            .map(|(destination, fraction)| {
                (*destination, runoff_to_distribute * fraction)
            })
            .collect();

        // Sanity check: Make sure we distributed the amount requested. If we
        // have no destinations though, then obviously we couldn't have
        // distributed anything
        if !distributed.is_empty() {
            assert_approx_eq!(
                distributed.values().copied().sum::<Meter3>().0,
                runoff_to_distribute.0
            );
        }

        // This runoff no longer belongs to us, so remove it from this basin
        let total_distributed_runoff: Meter3 =
            distributed.values().copied().sum();
        self.runoff -= total_distributed_runoff;

        distributed
    }
}

/// A container for all basins on a continent. Since basins within a continent
/// can grow and join each other, this container is useful to provide some
/// functionality around that. The main service this provides is for key
/// aliasing. When one basin is joined into another, the primary key of the
/// absorbed basin will no longer be valid. But there could still be lots of
/// places referencing that key (particularly as terminal positions in
/// [RunoffPattern]). As such, we need aliasing in order to get absorbed basins
/// to point to their new parents. This is all handled transparently within
/// this struct, so all the external methods on this struct support both alias
/// and primary keys.
#[derive(Debug)]
pub struct Basins {
    basins: HashMap<ResolvedBasinKey, Basin, FnvBuildHasher>,
    /// Mapping of alias->aliased key. You may be tempted to change this map
    /// to be alias->primary_key, because you think you're clever and that will
    /// always work since we always insert entries as alias->resolved. BUT that
    /// can break in situations like so:
    ///
    /// - Join a+b
    ///     - Insert alias for b->a
    /// - Join c+a
    ///     - Insert alias for a->c
    ///
    /// Now `b` points to `a`, which is no longer a valid primary key. So this
    /// map has to allow n levels of indirection.
    aliases: HashMap<BasinKey, BasinKey, FnvBuildHasher>,
}

impl Basins {
    /// Initialize all basins on the given continent. This will create one basin
    /// per terminal. Since this function doesn't actually have access to
    /// runoff pattern data, we will assume that any tile with runoff on it is a
    /// terminal. As such, this should only be called **after** runoff has
    /// been pushed out to all the terminal tiles.
    pub fn new(continent: &mut TilePointIndexMap<&mut TileBuilder>) -> Self {
        // Create one basin per terminal tile
        let mut basins = HashMap::default();
        for tile in continent.values_mut() {
            if tile.runoff() > Meter3(0.0) {
                let basin = Basin::new(tile);
                basins.insert(basin.key, basin);
            }
        }
        Self {
            basins,
            aliases: HashMap::default(),
        }
    }

    /// Resolve a basin key, which could possibly be an alias. **The returned
    /// key is not guaranteed to be valid** -- if the given key was invalid
    /// to begin with, then the returned key won't point to anything in the
    /// basins map.
    fn resolve(&self, key: BasinKey) -> ResolvedBasinKey {
        match self.aliases.get(&key) {
            // This key isn't an alias, so upgrade it to a resolved key
            // NOTE: it could still be invalid (i.e. not in the basins map)
            None => key.danger_upgrade(),
            // Stored key is an alias, so do another looking
            Some(key) => self.resolve(*key),
        }
    }

    /// Iterate over all basin keys. This will NOT include alias keys, only
    /// primary keys.
    pub fn keys(&self) -> impl Iterator<Item = TilePoint> + '_ {
        self.basins.keys().map(|key| key.0)
    }

    /// Move the basins out of this struct.
    pub fn into_basins(self) -> impl Iterator<Item = Basin> {
        self.basins.into_values()
    }

    /// Get a reference to a basin. If the given key is an alias, the alias will
    /// be resolved to find the correct basin. Under normal circumstances, we
    /// would never expect a basin lookup to fail, because anything that we
    /// think is a basin key _should_ be a basin key. So as a convenience
    /// measure, this panics in the case of an unknown key, instead of returning
    /// an `Option`.
    pub fn get(&self, key: TilePoint) -> &Basin {
        let key = self.resolve(key.into());
        unwrap!(self.basins.get(&key), "unknown basin key {}", key)
    }

    /// Get a mutable reference to a basin. If the given key is an alias, the
    /// alias will be resolved to find the correct basin. Under normal
    /// circumstances, we would never expect a basin lookup to fail, because
    /// anything that we think is a basin key _should_ be a basin key. So as a
    /// convenience measure, this panics in the case of an unknown key, instead
    /// of returning an `Option`.
    pub fn get_mut(&mut self, key: TilePoint) -> &mut Basin {
        let key = self.resolve(key.into());
        unwrap!(self.basins.get_mut(&key), "unknown basin key {}", key)
    }

    /// Has `donor` overflowed into `recipient` in the past?
    pub fn has_previously_overflowed(
        &self,
        donor: TilePoint,
        recipient: TilePoint,
    ) -> bool {
        let recipient_basin = self.get(recipient);
        recipient_basin.prev_donors.contains(&donor.into())
    }

    /// Join one basin into another, and add some amount of residual overflow
    /// into the result. The basin referred to be `a` will be mutated while the
    /// basin referred to by `b` will be **removed from this collection**. If
    /// `a` and `b` refer to the same basin, then we'll just add the overflow.
    /// Returns the resulting basin (which is just whatever `a` points to).
    pub fn join(
        &mut self,
        a: TilePoint,
        b: TilePoint,
        overflow: Meter3,
    ) -> &Basin {
        let a: BasinKey = a.into();
        let b: BasinKey = b.into();
        trace!("Joining basin {} into basin {}", b, a);
        // Either key could be an alias, resolve both to get the actual keys
        let a_res = self.resolve(a);
        let b_res = self.resolve(b);

        // If a and b are different basins, then remove b and join it into a.
        // This should be the case most of the time, but occasionally we can hit
        // a scenario where we try to join in multiple basins in one iteration
        // of the loop that calls this, where it turns out that both
        // joinees alias to the same basin. In those scenarios, the join is
        // mostly a no-op (we just need to add in the overflow we were given
        // still).
        let b_basin = if a_res != b_res {
            let b_basin = unwrap!(
                self.basins.remove(&b_res),
                "unknown basin: {} (resolved from {})",
                b_res,
                b
            );
            // Store an alias for b->a. For b, we have to use the resolved
            // version, so that if b is already an alias, we add a new alias to
            // the end of its alias chain. For a, we could hypothetically use
            // the unresolved version, but using the resolved key will reduce
            // alias lookups later on.
            let existing =
                self.aliases.insert(b_res.downgrade(), a_res.downgrade());
            // Sanity check that we didn't overwrite an existing alias
            assert!(
                existing.is_none(),
                "Overwrote alias for existing key {} (was pointing to {:?})",
                b_res.downgrade(),
                existing
            );

            Some(b_basin)
        } else {
            None
        };

        let basin = unwrap!(
            self.basins.get_mut(&a_res),
            "unknown basin: {} (resolved from {})",
            a_res,
            a
        );

        // If we actually have two basins, join them
        if let Some(b_basin) = b_basin {
            basin.join(b_basin);
        }
        // Add in whatever overflow we were given
        basin.runoff += overflow;

        basin
    }
}
