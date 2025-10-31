# SubPoint Migration Tracker

Incremental migration from SubCellCoord to SubPoint flat coordinate system.

## Strategy
- Migrate one component at a time
- Compile and test after each change
- Commit working checkpoints
- Keep both types coexisting during migration

## Progress

### Phase 1: Actor Struct Fields
- [X] `current_subcell: Option<SubCellCoord>` → `Option<SubPoint>` ✓ (Commit: d748020)
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
- [X] `src/actor_directives.rs` - any SubCellCoord usage ✓ (Commit: d748020)
- [X] `src/main.rs` - visualization and GUI code ✓ (Commit: d748020)
- [X] `src/lib.rs` - update exports ✓ (Commit: d748020)

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

### Session Log
- **d748020**: Migrated Actor.current_subcell field
  - Fixed 26 errors in actor.rs
  - Fixed actor_directives.rs, main.rs, lib.rs
  - Library compiles, 25/25 unit tests pass
  - 14 integration test files need field access fixes (pending)
