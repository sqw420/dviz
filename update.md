# MViz Development Update Log

This file tracks implementation progress, test results, and check-in records.

---

## Phase 1: Core Foundation

**Status:** COMPLETED
**Date:** 2026-01-06

### Task 1.1: Transform Types
- **Status:** Completed
- **Files Created:**
  - `mviz-core/src/types/transform.rs`
  - `mviz-core/src/types/mod.rs`
  - `mviz-core/src/lib.rs`
  - `mviz-core/Cargo.toml`
- **Types Implemented:**
  - `FrameId` - Coordinate frame identifier
  - `Timestamp` - Nanosecond precision timestamps
  - `Transform` - 3D rigid transform (translation + quaternion rotation)
  - `StampedTransform` - Transform with timestamp and frame IDs
  - `Pose` - Position and orientation in a frame
- **Tests:** 10 tests passing
- **Notes:** Fixed `Mul` trait implementation for Transform composition

### Task 1.2: Point Cloud Types
- **Status:** Completed
- **Files Created:**
  - `mviz-core/src/types/point_cloud.rs`
- **Types Implemented:**
  - `Color` - RGBA color with constants and HSV conversion
  - `Colormap` - 8 colormaps (Jet, Rainbow, Turbo, Viridis, Grayscale, Hot, Cool, Plasma)
  - `ColorMode` - Flat, PerPoint, ByAxis, ByIntensity, ByHeight, ByDistance
  - `Axis` - X, Y, Z enum
  - `PointCloudStyle` - Point size, colormap, alpha settings
  - `PointCloud` - Points with positions, colors, intensities
- **Tests:** 8 tests passing
- **Notes:** Added serde_json as dev-dependency for serialization tests

### Task 1.3: Marker Types
- **Status:** Completed
- **Files Created:**
  - `mviz-core/src/types/marker.rs`
- **Types Implemented:**
  - `MarkerType` - Arrow, Cube, Sphere, Cylinder, LineStrip, LineList, CubeList, SphereList, Points, Text, MeshResource, TriangleList
  - `MarkerAction` - Add, Modify, Delete, DeleteAll
  - `MarkerScale` - X, Y, Z scaling
  - `Marker` - Full marker with all properties
  - `MarkerBuilder` - Fluent builder pattern for marker creation
  - `MarkerArray` - Collection of markers with iteration support
- **Tests:** 9 tests passing

### Task 1.4: Display Trait
- **Status:** Completed
- **Files Created:**
  - `mviz-core/src/display.rs`
- **Types Implemented:**
  - `DisplayError` - Error types for display operations
  - `DisplayStatus` - Disabled, Initializing, Ok, Warning, Error
  - `DisplayInfo` - Metadata about display types
  - `PropertyValue` - Dynamic property values (Bool, Int, Float, String, Color, Vec3)
  - `PropertyMeta` - Property metadata with defaults
  - `Properties` - Property container with typed accessors
  - `TransformProvider` trait - Coordinate frame transform lookup
  - `IdentityTransformProvider` - Simple identity transform provider
  - `DisplayContext` - Context for accessing visualization services
  - `DisplayId` - Unique display instance identifier
  - `Display` trait - Base trait for all visualization displays
  - `DisplayFactory` trait - Factory for creating display instances
  - `DisplayRegistry` - Registry of display factories
- **Tests:** 8 tests passing

### Task 1.5: Configuration Schema
- **Status:** Completed
- **Files Created:**
  - `mviz-core/src/config.rs`
- **Types Implemented:**
  - `ConfigError` - Configuration error types
  - `WindowConfig` - Window size, position, fullscreen settings
  - `ViewConfig` - Camera position, FOV, clipping planes
  - `DisplayConfig` - Per-display configuration with properties
  - `GlobalConfig` - Fixed frame, background color, frame rate
  - `PanelConfig` - UI panel configuration
  - `AppConfig` - Complete application configuration
- **Features:**
  - YAML serialization/deserialization
  - File load/save support
  - Configuration validation (duplicate detection)
  - Default configurations with common displays
- **Tests:** 9 tests passing
- **Dependencies Added:** serde_json, serde_yaml

### Phase 1 Summary

**Total Tests:** 44 passing
**Crate Structure:**
```
mviz-core/
  Cargo.toml
  src/
    lib.rs
    display.rs
    config.rs
    types/
      mod.rs
      transform.rs
      point_cloud.rs
      marker.rs
```

**Workspace Updated:** Added mviz-core to workspace members in root Cargo.toml

**Warnings:** 6 warnings about unnecessary parentheses in point_cloud.rs colormap calculations (cosmetic, does not affect functionality)

---

## Phase 1 Stream B: Transform System

**Status:** COMPLETED
**Date:** 2026-01-06

### Task 1.6: Frame Tree
- **Status:** Completed
- **Files Created:**
  - `mviz-transform/Cargo.toml`
  - `mviz-transform/src/lib.rs`
  - `mviz-transform/src/frame_tree.rs`
- **Types Implemented:**
  - `FrameNode` - Node in frame tree with parent/children relationships
  - `FrameTree` - Coordinate frame hierarchy with path finding
- **Features:**
  - Parent-child frame relationships
  - `path_to_root()` - Find path from frame to root
  - `common_ancestor()` - Find common ancestor of two frames
  - `path_between()` - Find path between any two frames
  - Frame removal with orphan handling
- **Tests:** 12 tests passing

### Task 1.7: Transform Buffer
- **Status:** Completed
- **Files Created:**
  - `mviz-transform/src/transform_buffer.rs`
- **Types Implemented:**
  - `TransformKey` - (parent, child) frame pair key
  - `TransformHistory` - Time-indexed transforms with interpolation
  - `TransformBuffer` - Thread-safe transform buffer with frame tree
  - `TransformError` - NoPath, TransformNotFound, FrameNotFound, EmptyBuffer
  - `TransformResult<T>` - Result alias
- **Features:**
  - Linear interpolation between transforms
  - `prune_old()` - Remove transforms older than duration
  - `lookup_transform()` - Look up transform between any two frames
  - Automatic transform chaining through frame tree
- **Tests:** 9 tests passing
- **Notes:** Fixed thiserror named fields to tuple variants

---

## Phase 1 Stream C: Rerun Integration

**Status:** COMPLETED
**Date:** 2026-01-06

### Task 1.8-1.11: Core Adapters
- **Status:** Completed
- **Files Modified:**
  - `mviz-rerun-bridge/Cargo.toml` (added mviz-core dependency)
  - `mviz-rerun-bridge/src/lib.rs` (added core_adapters export)
- **Files Created:**
  - `mviz-rerun-bridge/src/core_adapters.rs`
- **Types Implemented:**
  - `PointCloudCoreAdapter` - Log PointCloud to Rerun with color modes
  - `TransformCoreAdapter` - Log Transform to Rerun with frame axes visualization
  - `MarkerCoreAdapter` - Log all Marker types to Rerun
- **Marker Types Supported:**
  - Arrow, Cube, Sphere, Cylinder
  - LineStrip, LineList
  - CubeList, SphereList
  - Points, Text, TriangleList
  - MeshResource (logged warning, not yet supported)
- **Color Modes Supported:**
  - FlatColor - Single color for all points
  - RGB - Per-point colors from point cloud data
  - AxisColor - Color by axis position (X, Y, Z) with colormap
  - Intensity - Color by intensity value with colormap
- **Tests:** 3 tests passing (colormap_lookup, compute_colors_flat, compute_colors_by_axis)
- **Notes:**
  - Fixed Rerun 0.28 API: `from_unmultiplied_rgba()`, `to_array()`
  - Fixed String to &str conversions for entity paths

### Phase 1 Streams B+C Summary

**New Tests:** 27 (12 frame_tree + 9 transform_buffer + 6 rerun-bridge)
**Total Workspace Tests:** 71 passing
**New Crates:**
```
mviz-transform/
  Cargo.toml
  src/
    lib.rs
    frame_tree.rs
    transform_buffer.rs

mviz-rerun-bridge/
  src/
    core_adapters.rs (new)
```

---

## Next Phase: Phase 2 - Display Plugins

**Planned Tasks:**
- Task 2.1: Point Cloud Display (with Makepad widget)
- Task 2.2: Marker Display (with Makepad widget)
- Task 2.3: Transform Display (with Makepad widget)
- Task 2.4: Grid Display
- Task 2.5: Plugin loading and registration

---

## Version History

| Version | Date | Commit | Description |
|---------|------|--------|-------------|
| v0.1.2 | 2026-01-06 | b574362 | Phase 1 Streams B+C: Transform System + Core Adapters |
| v0.1.1 | 2026-01-06 | 13bdb7b | Phase 1 Core Foundation complete |
| v0.1.0 | 2026-01-05 | 1c21c15 | Initial release with UI shell and simulation |

## Git History

### v0.1.1 (2026-01-06)
- **Commit:** 13bdb7b
- **Files Changed:** 11 files, +3221 lines
- **Tag:** v0.1.1
- **Pushed:** Yes
- **Tests:** 44 passing
