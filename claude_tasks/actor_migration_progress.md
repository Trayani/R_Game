sub# Actor.rs SubPoint Migration - In Progress

## ✅ COMPLETED (Struct Definitions)

### 1. Actor Struct Fields Migrated
**File**: `src/actor.rs` lines 152-154

```rustfix 
// BEFORE:
pub reserved_subcell: Option<SubCellCoord>,
pub extra_reserved_subcells: Vec<SubCellCoord>,

// AFTER:
pub reserved_subcell: Option<SubPoint>,
pub extra_reserved_subcells: Vec<SubPoint>,
```

### 2. Helper Structs Migrated
**File**: `src/actor.rs` lines 65, 78-79, 85-90

**AffinityResult** (line 65):
```rust
pub anchor: SubPoint,  // was SubCellCoord
```

**DirectingInfo** (lines 78-79):
```rust
pub reserved: SubPoint,  // was SubCellCoord
pub anchor: SubPoint,    // was SubCellCoord
```

**PSCSelectionInfo** (lines 85-90):
```rust
pub old_psc: SubPoint,           // was SubCellCoord
pub reserved: SubPoint,          // was SubCellCoord
pub anchor: Option<SubPoint>,    // was Option<SubCellCoord>
pub chosen: SubPoint,            // was SubCellCoord
```

## ⚠️ IN PROGRESS (73 Compilation Errors)

### Error Categories:
1. **Type mismatches (38 errors)**: Methods creating/returning SubCellCoord need to create/return SubPoint
2. **Field access (24 errors)**: Code accessing `.cell_x`, `.cell_y`, `.sub_x`, `.sub_y` needs `.to_cell()` and `.subcell_offset()`
3. **Method signature (6 errors)**: Methods taking 5 SubCellCoord args need to take 4 SubPoint args (no grid_size)
4. **Reservation manager (4 errors)**: `SubCellReservationManager` expects SubCellCoord, needs temporary conversion
5. **to_subpoint() calls (2 errors)**: SubPoint doesn't have `.to_subpoint()` (it IS a SubPoint)
6. **if/else types (2 errors)**: Branches returning different types

### Key Error Locations:

**Anchor creation (lines 870, 944, 1002, 1092)**:
- `anchor` variable is created as `SubCellCoord` but needs to be `SubPoint`
- Need to use `SubPoint::new()` or `get_horizontal_anchor()`/`get_vertical_anchor()` from actor_execution

**Reservation methods (lines 1337-1338, 1647-1648)**:
- Setting `self.reserved_subcell` and `self.extra_reserved_subcells`
- Creating SubCellCoord but need SubPoint

**Field access (lines 1558-1559, many others)**:
- Accessing `.cell_x`, `.sub_x` etc. on SubPoint
- Need to use `.to_cell(grid_size)` and `.subcell_offset(grid_size)`

**Reservation manager calls (lines 1564, 1577, 1578, 1626, etc.)**:
- `reservation_manager.get_owner(&subpoint)` needs SubCellCoord
- Temporary fix: `SubCellCoord::from_subpoint(&subpoint, grid_size)`

**Duplicate helper functions (lines 708-750)**:
- `is_diagonal_move()` - duplicate of actor_execution, should be removed
- `find_anchor_cell()` - duplicate of actor_execution, should be removed
- `get_horizontal_anchor()` (line 1099) - duplicate, should use actor_execution version
- `get_vertical_anchor()` (line 1112) - duplicate, should use actor_execution version

## 🔧 SYSTEMATIC FIX APPROACH

### Phase 1: Remove Duplicate Functions
1. Delete `is_diagonal_move()` at line 708 - use `crate::actor_execution::is_diagonal_move`
2. Delete `find_anchor_cell()` at line 718 - use `crate::actor_execution::find_anchor_cell`
3. Delete `get_horizontal_anchor()` at line 1099 - use `crate::actor_execution::get_horizontal_anchor`
4. Delete `get_vertical_anchor()` at line 1112 - use `crate::actor_execution::get_vertical_anchor`

### Phase 2: Fix Anchor Creation
Replace all `SubCellCoord::new(...)` calls that create anchors with:
```rust
use crate::actor_execution::{get_horizontal_anchor, get_vertical_anchor};

// Instead of:
let anchor = SubCellCoord::new(target.cell_x, current.cell_y, target.sub_x, current.sub_y, grid_size);

// Use:
let anchor = get_horizontal_anchor(&current, &target);
```

### Phase 3: Fix Field Access
Replace all field access patterns:
```rust
// Instead of:
anchor.cell_x, anchor.cell_y, anchor.sub_x, anchor.sub_y

// Use:
let (cell_x, cell_y) = anchor.to_cell(grid_size);
let (sub_x, sub_y) = anchor.subcell_offset(grid_size);
```

### Phase 4: Fix Reservation Manager Calls
Add temporary conversions for SubCellReservationManager:
```rust
// Instead of:
reservation_manager.get_owner(&subpoint)

// Use (temporary until SubCellReservationManager migrated):
let coord = SubCellCoord::from_subpoint(&subpoint, grid_size);
reservation_manager.get_owner(&coord)
```

### Phase 5: Remove Invalid to_subpoint() Calls
```rust
// Instead of:
let coord = subpoint.to_subpoint();

// Just use:
let coord = subpoint;  // or *subpoint if it's a reference
```

## 📝 NEXT STEPS

1. Continue fixing compilation errors phase by phase
2. Test after each phase
3. Once actor.rs compiles, update actor_directives.rs
4. Run full test suite
5. Commit the complete migration

## 🎯 GOAL

Complete migration of actor.rs from SubCellCoord to SubPoint, leveraging the 34 SubPoint utility functions created earlier.

## 📊 PROGRESS

- Structs migrated: ✅ 100%
- Compilation errors fixed: ⏳ 0/73
- Tests passing: ⏳ Pending

**Estimated remaining effort**: Large (73 errors to fix systematically)
