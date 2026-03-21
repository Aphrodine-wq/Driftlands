use bevy::prelude::*;
use std::collections::HashMap;
use std::time::Instant;
use crate::combat::Enemy;
use crate::building::Building;
use crate::farming::FarmPlot;
use crate::world::WorldObject;

pub struct SpatialPlugin;

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::new(CELL_SIZE))
            .add_systems(Startup, populate_spatial_grid_initial)
            .add_systems(First, update_spatial_grid_incremental);
    }
}

const CELL_SIZE: f32 = 128.0;

/// Simple 2D spatial grid for fast proximity queries.
/// Divides world into cells of CELL_SIZE pixels. Entities register into cells.
#[derive(Resource)]
pub struct SpatialGrid {
    pub cell_size: f32,
    pub enemy_cells: HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
    pub building_cells: HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
    pub farm_cells: HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
    pub world_object_cells: HashMap<(i32, i32), Vec<(Entity, Vec2)>>,

    // Tracks which cell each entity currently belongs to so we can update
    // membership incrementally without rebuilding the whole grid.
    enemy_entity_cell: HashMap<Entity, (i32, i32)>,
    building_entity_cell: HashMap<Entity, (i32, i32)>,
    farm_entity_cell: HashMap<Entity, (i32, i32)>,
    world_object_entity_cell: HashMap<Entity, (i32, i32)>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            enemy_cells: HashMap::new(),
            building_cells: HashMap::new(),
            farm_cells: HashMap::new(),
            world_object_cells: HashMap::new(),

            enemy_entity_cell: HashMap::new(),
            building_entity_cell: HashMap::new(),
            farm_entity_cell: HashMap::new(),
            world_object_entity_cell: HashMap::new(),
        }
    }

    /// Get cell key for a world position
    pub fn cell_key(&self, pos: Vec2) -> (i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        )
    }

    /// Query all enemies within radius of a position
    pub fn query_enemies_in_radius(&self, center: Vec2, radius: f32) -> Vec<(Entity, Vec2)> {
        let min_cell = self.cell_key(center - Vec2::splat(radius));
        let max_cell = self.cell_key(center + Vec2::splat(radius));
        let radius_sq = radius * radius;
        let mut results = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entries) = self.enemy_cells.get(&(cx, cy)) {
                    for &(entity, pos) in entries {
                        if center.distance_squared(pos) <= radius_sq {
                            results.push((entity, pos));
                        }
                    }
                }
            }
        }
        results
    }

    /// Query all buildings within radius of a position
    pub fn query_buildings_in_radius(&self, center: Vec2, radius: f32) -> Vec<(Entity, Vec2)> {
        let min_cell = self.cell_key(center - Vec2::splat(radius));
        let max_cell = self.cell_key(center + Vec2::splat(radius));
        let radius_sq = radius * radius;
        let mut results = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entries) = self.building_cells.get(&(cx, cy)) {
                    for &(entity, pos) in entries {
                        if center.distance_squared(pos) <= radius_sq {
                            results.push((entity, pos));
                        }
                    }
                }
            }
        }
        results
    }

    /// Query all farms within radius of a position
    pub fn query_farms_in_radius(&self, center: Vec2, radius: f32) -> Vec<(Entity, Vec2)> {
        let min_cell = self.cell_key(center - Vec2::splat(radius));
        let max_cell = self.cell_key(center + Vec2::splat(radius));
        let radius_sq = radius * radius;
        let mut results = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entries) = self.farm_cells.get(&(cx, cy)) {
                    for &(entity, pos) in entries {
                        if center.distance_squared(pos) <= radius_sq {
                            results.push((entity, pos));
                        }
                    }
                }
            }
        }
        results
    }

    /// Query all world objects within radius of a position.
    pub fn query_world_objects_in_radius(&self, center: Vec2, radius: f32) -> Vec<(Entity, Vec2)> {
        let min_cell = self.cell_key(center - Vec2::splat(radius));
        let max_cell = self.cell_key(center + Vec2::splat(radius));
        let radius_sq = radius * radius;
        let mut results = Vec::new();
        for cx in min_cell.0..=max_cell.0 {
            for cy in min_cell.1..=max_cell.1 {
                if let Some(entries) = self.world_object_cells.get(&(cx, cy)) {
                    for &(entity, pos) in entries {
                        if center.distance_squared(pos) <= radius_sq {
                            results.push((entity, pos));
                        }
                    }
                }
            }
        }
        results
    }
}

fn remove_entity_from_cell(
    cells: &mut HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
    cell_key: (i32, i32),
    entity: Entity,
) {
    if let Some(entries) = cells.get_mut(&cell_key) {
        if let Some(idx) = entries.iter().position(|(e, _)| *e == entity) {
            entries.swap_remove(idx);
        }
        if entries.is_empty() {
            cells.remove(&cell_key);
        }
    }
}

fn upsert_entity_in_cell(
    cells: &mut HashMap<(i32, i32), Vec<(Entity, Vec2)>>,
    cell_key: (i32, i32),
    entity: Entity,
    pos: Vec2,
) {
    let entries = cells.entry(cell_key).or_default();
    if let Some((_, existing_pos)) = entries.iter_mut().find(|(e, _)| *e == entity) {
        *existing_pos = pos;
    } else {
        entries.push((entity, pos));
    }
}

/// One-time population so queries are correct immediately on the first frame.
fn populate_spatial_grid_initial(
    mut grid: ResMut<SpatialGrid>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    building_query: Query<(Entity, &Transform), With<Building>>,
    farm_query: Query<(Entity, &Transform), With<FarmPlot>>,
    object_query: Query<(Entity, &Transform), With<WorldObject>>,
) {
    grid.enemy_cells.clear();
    grid.building_cells.clear();
    grid.farm_cells.clear();
    grid.world_object_cells.clear();
    grid.enemy_entity_cell.clear();
    grid.building_entity_cell.clear();
    grid.farm_entity_cell.clear();
    grid.world_object_entity_cell.clear();

    for (entity, tf) in enemy_query.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        grid.enemy_cells.entry(key).or_default().push((entity, pos));
        grid.enemy_entity_cell.insert(entity, key);
    }

    for (entity, tf) in building_query.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        grid.building_cells.entry(key).or_default().push((entity, pos));
        grid.building_entity_cell.insert(entity, key);
    }

    for (entity, tf) in farm_query.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        grid.farm_cells.entry(key).or_default().push((entity, pos));
        grid.farm_entity_cell.insert(entity, key);
    }

    for (entity, tf) in object_query.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        grid.world_object_cells.entry(key).or_default().push((entity, pos));
        grid.world_object_entity_cell.insert(entity, key);
    }
}

/// Incrementally updates cell membership based on entity `Transform` changes.
///
/// This avoids the previous O(N) rebuild that cleared/reinserted every entity every frame.
fn update_spatial_grid_incremental(
    mut grid: ResMut<SpatialGrid>,
    enemy_added: Query<(Entity, &Transform), Added<Enemy>>,
    enemy_changed: Query<(Entity, &Transform), (With<Enemy>, Changed<Transform>)>,
    mut enemy_removed: RemovedComponents<Enemy>,
    building_added: Query<(Entity, &Transform), Added<Building>>,
    building_changed: Query<(Entity, &Transform), (With<Building>, Changed<Transform>)>,
    mut building_removed: RemovedComponents<Building>,
    farm_added: Query<(Entity, &Transform), Added<FarmPlot>>,
    farm_changed: Query<(Entity, &Transform), (With<FarmPlot>, Changed<Transform>)>,
    mut farm_removed: RemovedComponents<FarmPlot>,
    object_added: Query<(Entity, &Transform), Added<WorldObject>>,
    object_changed: Query<(Entity, &Transform), (With<WorldObject>, Changed<Transform>)>,
    mut object_removed: RemovedComponents<WorldObject>,
    mut perf: ResMut<crate::debug_perf::DebugPerfTiming>,
) {
    let start = Instant::now();
    // --- Enemies ---
    for (entity, tf) in enemy_added.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        upsert_entity_in_cell(&mut grid.enemy_cells, key, entity, pos);
        grid.enemy_entity_cell.insert(entity, key);
    }

    for (entity, tf) in enemy_changed.iter() {
        let pos = tf.translation.truncate();
        let new_key = grid.cell_key(pos);
        let old_key = grid.enemy_entity_cell.get(&entity).copied();

        match old_key {
            Some(okey) if okey == new_key => {
                upsert_entity_in_cell(&mut grid.enemy_cells, new_key, entity, pos);
            }
            Some(okey) => {
                remove_entity_from_cell(&mut grid.enemy_cells, okey, entity);
                upsert_entity_in_cell(&mut grid.enemy_cells, new_key, entity, pos);
                grid.enemy_entity_cell.insert(entity, new_key);
            }
            None => {
                upsert_entity_in_cell(&mut grid.enemy_cells, new_key, entity, pos);
                grid.enemy_entity_cell.insert(entity, new_key);
            }
        }
    }

    for entity in enemy_removed.read() {
        if let Some(old_key) = grid.enemy_entity_cell.remove(&entity) {
            remove_entity_from_cell(&mut grid.enemy_cells, old_key, entity);
        }
    }

    // --- Buildings ---
    for (entity, tf) in building_added.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        upsert_entity_in_cell(&mut grid.building_cells, key, entity, pos);
        grid.building_entity_cell.insert(entity, key);
    }

    for (entity, tf) in building_changed.iter() {
        let pos = tf.translation.truncate();
        let new_key = grid.cell_key(pos);
        let old_key = grid.building_entity_cell.get(&entity).copied();

        match old_key {
            Some(okey) if okey == new_key => {
                upsert_entity_in_cell(&mut grid.building_cells, new_key, entity, pos);
            }
            Some(okey) => {
                remove_entity_from_cell(&mut grid.building_cells, okey, entity);
                upsert_entity_in_cell(&mut grid.building_cells, new_key, entity, pos);
                grid.building_entity_cell.insert(entity, new_key);
            }
            None => {
                upsert_entity_in_cell(&mut grid.building_cells, new_key, entity, pos);
                grid.building_entity_cell.insert(entity, new_key);
            }
        }
    }

    for entity in building_removed.read() {
        if let Some(old_key) = grid.building_entity_cell.remove(&entity) {
            remove_entity_from_cell(&mut grid.building_cells, old_key, entity);
        }
    }

    // --- Farms ---
    for (entity, tf) in farm_added.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        upsert_entity_in_cell(&mut grid.farm_cells, key, entity, pos);
        grid.farm_entity_cell.insert(entity, key);
    }

    for (entity, tf) in farm_changed.iter() {
        let pos = tf.translation.truncate();
        let new_key = grid.cell_key(pos);
        let old_key = grid.farm_entity_cell.get(&entity).copied();

        match old_key {
            Some(okey) if okey == new_key => {
                upsert_entity_in_cell(&mut grid.farm_cells, new_key, entity, pos);
            }
            Some(okey) => {
                remove_entity_from_cell(&mut grid.farm_cells, okey, entity);
                upsert_entity_in_cell(&mut grid.farm_cells, new_key, entity, pos);
                grid.farm_entity_cell.insert(entity, new_key);
            }
            None => {
                upsert_entity_in_cell(&mut grid.farm_cells, new_key, entity, pos);
                grid.farm_entity_cell.insert(entity, new_key);
            }
        }
    }

    for entity in farm_removed.read() {
        if let Some(old_key) = grid.farm_entity_cell.remove(&entity) {
            remove_entity_from_cell(&mut grid.farm_cells, old_key, entity);
        }
    }

    // --- World objects ---
    for (entity, tf) in object_added.iter() {
        let pos = tf.translation.truncate();
        let key = grid.cell_key(pos);
        upsert_entity_in_cell(&mut grid.world_object_cells, key, entity, pos);
        grid.world_object_entity_cell.insert(entity, key);
    }

    for (entity, tf) in object_changed.iter() {
        let pos = tf.translation.truncate();
        let new_key = grid.cell_key(pos);
        let old_key = grid.world_object_entity_cell.get(&entity).copied();

        match old_key {
            Some(okey) if okey == new_key => {
                upsert_entity_in_cell(&mut grid.world_object_cells, new_key, entity, pos);
            }
            Some(okey) => {
                remove_entity_from_cell(&mut grid.world_object_cells, okey, entity);
                upsert_entity_in_cell(&mut grid.world_object_cells, new_key, entity, pos);
                grid.world_object_entity_cell.insert(entity, new_key);
            }
            None => {
                upsert_entity_in_cell(&mut grid.world_object_cells, new_key, entity, pos);
                grid.world_object_entity_cell.insert(entity, new_key);
            }
        }
    }

    for entity in object_removed.read() {
        if let Some(old_key) = grid.world_object_entity_cell.remove(&entity) {
            remove_entity_from_cell(&mut grid.world_object_cells, old_key, entity);
        }
    }

    perf.spatial_update_ms = start.elapsed().as_secs_f32() * 1000.0;
}
