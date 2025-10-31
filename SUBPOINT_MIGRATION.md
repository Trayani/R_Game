# SubPoint Migration Tracker

Incremental migration from SubCellCoord to SubPoint flat coordinate system.

## Strategy
- Migrate one component at a time
- Compile and test after each change
- Commit working checkpoints
- Keep both types coexisting during migration

## Progress

### Phase 1: Actor Struct Fields
- [ ] `current_subcell: Option<SubCellCoord>` → `Option<SubPoint>`
- [ ] `reserved_subcell: Option<SubCellCoord>` → `Option<SubPoint>`
- [ ] `extra_reserved_subcells: Vec<SubCellCoord>` → `Vec<SubPoint>`

### Phase 2: Helper Structs (src/actor.rs)
- [ ] `AffinityResult.anchor: SubCellCoord` → `SubPoint`
- [ ] `DirectingInfo.reserved: SubCellCoord` → `SubPoint`
- [ ] `DirectingInfo.anchor: SubCellCoord` → `SubPoint`
- [ ] `PSCSelectionInfo.old_psc: SubCellCoord` → `SubPoint`
- [ ] `PSCSelectionInfo.reserved: SubCellCoord` → `SubPoint`
- [ ] `PSCSelectionInfo.anchor: Option<SubCellCoord>` → `Option<SubPoint>`
- [ ] `PSCSelectionInfo.chosen: SubCellCoord` → `SubPoint`

### Phase 3: Utility Methods (src/actor.rs)
- [ ] `is_diagonal_move(SubCellCoord, SubCellCoord)` → `(SubPoint, SubPoint)`
- [ ] `find_anchor_cell(SubCellCoord, SubCellCoord)` → `(SubPoint, SubPoint)`
- [ ] `check_anti_cross(SubCellCoord, SubCellCoord)` → `(SubPoint, SubPoint)`
- [ ] `get_horizontal_anchor(SubCellCoord, SubCellCoord)` → `(SubPoint, SubPoint)`
- [ ] `get_vertical_anchor(SubCellCoord, SubCellCoord)` → `(SubPoint, SubPoint)`

### Phase 4: Core Methods (src/actor.rs)
- [ ] `set_subcell_destination(Position)`
- [ ] `calculate_affinity_using_ray_intersection(...)`
- [ ] `calculate_simple_affinity_and_target(...)`
- [ ] `try_reserve_diagonal_with_anchor(...)`
- [ ] `try_reserve_horizontal_vertical(...)`
- [ ] `update_subcell_destination_direct_impl(...)`

### Phase 5: External Files
- [ ] `src/actor_execution.rs` - struct literals and field accesses
- [ ] `src/actor_directives.rs` - any SubCellCoord usage
- [ ] `src/main.rs` - visualization and GUI code
- [ ] `src/lib.rs` - update exports

### Phase 6: Tests (33 files)
- [ ] `tests/test_actor_directing.rs`
- [ ] `tests/test_direct_movement.rs`
- [ ] `tests/test_destination_direct_spec.rs`
- [ ] `tests/test_actor3_backwards.rs`
- [ ] `tests/test_diagonal_cardinal_fallback.rs`
- [ ] ... (28 more test files)

### Phase 7: Cleanup
- [ ] Remove SubCellCoord type (keep as deprecated alias temporarily?)
- [ ] Remove to_subpoint()/from_subpoint() conversion helpers
- [ ] Update SUBCELL_IMPLEMENTATION.md documentation
- [ ] Update CLAUDE.md with new SubPoint system
- [ ] Final full test run

## Notes
- After each checkbox: compile, test, commit
- Keep checkpoints every 2-3 changes
- If blocked, document issue and move to next item
- Maintain parallel compatibility during migration

## Current Session
Started: 2025-10-31
Target: Migrate Actor struct fields first
