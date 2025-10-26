# SwiftUI + Bevy(Rust)

A demo that integrates Bevy ECS with wgpu rendering for iOS/Swift applications. This library provides a C FFI interface to create a 3D rendering engine that can be called from Swift.

## Overview

This library demonstrates how to:
- Use Bevy ECS (Entity Component System) for game logic
- Render 3D graphics using wgpu with Metal backend on iOS
- Expose Rust functionality to Swift through C FFI
- Integrate with `CAMetalLayer` for native iOS rendering

## Features

- **ECS-based architecture**: Leverages Bevy's powerful Entity Component System
- **3D rendering**: Basic 3D cube rendering with wgpu
- **Transform system**: Position, rotation, and scale components
- **Animation system**: Spinning cube demo with configurable rotation speed
- **C FFI interface**: Easy integration with Swift/iOS applications

## Architecture

### Components

- `Transform3D`: Position, rotation, and scale for 3D objects
- `SpinY`: Rotation behavior component with configurable speed
- `CubeMeshTag`: Marker component for cube entities

### Systems

- `spin_system_3d`: Rotates entities with `SpinY` component around Y-axis

### Resources

- `DeltaTime`: Stores frame delta time for time-based animations

## C API

The library exposes three functions for Swift interop:

### `engine_init`

```c
*mut Engine engine_init(*mut c_void layer_ptr, u32 width, u32 height)
```

Initializes the engine with a CAMetalLayer pointer and surface dimensions.

**Parameters:**
- `layer_ptr`: Pointer to CAMetalLayer
- `width`: Surface width in pixels
- `height`: Surface height in pixels

**Returns:** Pointer to the Engine instance

### `engine_frame`

```c
void engine_frame(*mut Engine engine, f32 dt_seconds)
```

Updates and renders one frame.

**Parameters:**
- `engine`: Engine instance pointer
- `dt_seconds`: Delta time in seconds since last frame

### `engine_free`

```c
void engine_free(*mut Engine engine)
```

Cleans up and frees the engine instance.

**Parameters:**
- `engine`: Engine instance pointer to free

## Building

### For iOS (static library)

```bash
# Add iOS target
rustup target add aarch64-apple-ios

# Build for iOS device
cargo build --release --target aarch64-apple-ios

# Build for iOS simulator
cargo build --release --target aarch64-apple-ios-sim
```

The static library will be generated at:
- `target/aarch64-apple-ios/release/libbevy_swift.a`

## Dependencies

- **bevy_ecs** (0.17.2): Entity Component System
- **wgpu** (26.0.1): Cross-platform graphics API
- **glam** (0.27): Mathematics library for 3D graphics
- **pollster** (0.4.0): Async executor for blocking on futures
- **bytemuck** (1): Type-safe data casting for GPU buffers

## Example Usage in Swift

```swift
import Metal
import MetalKit

class GameView: MTKView {
    var engine: OpaquePointer?

    override init(frame: CGRect, device: MTLDevice?) {
        super.init(frame: frame, device: device)

        // Get CAMetalLayer
        let metalLayer = self.layer as! CAMetalLayer
        let layerPtr = Unmanaged.passUnretained(metalLayer).toOpaque()

        // Initialize engine
        engine = engine_init(
            layerPtr,
            UInt32(frame.width),
            UInt32(frame.height)
        )
    }

    func update(deltaTime: Float) {
        if let engine = engine {
            engine_frame(engine, deltaTime)
        }
    }

    deinit {
        if let engine = engine {
            engine_free(engine)
        }
    }
}
```