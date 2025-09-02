# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library (`ptouch`) that provides USB printing capability for Brother P-Touch QL series label printers. The crate allows programmatic generation and printing of label images using the rusb library for USB communication.

## Development Commands

### Building
```bash
cargo build
```

### Running Examples
```bash
# Read printer status with debug logging
RUST_LOG=debug cargo run --example read_status

# Print test labels (various options available)
cargo run --example print_rust [normal|high|multiple|qr]

# Examples:
cargo run --example print_rust normal    # Normal resolution single label
cargo run --example print_rust multiple  # Multiple high resolution labels
cargo run --example print_rust qr        # QR code labels
```

### Testing
```bash
cargo test
```

## Core Architecture

### Module Structure

- **`printer.rs`** - Main `Printer` struct handling USB communication, status checking, and print operations
- **`model.rs`** - Defines supported printer models (QL-720NW, QL-800, QL-820NWB, etc.) with their PIDs and specifications
- **`media.rs`** - Media type definitions (Continuous vs DieCut) with detailed specifications for dimensions and offsets
- **`error.rs`** - Error types for USB, device, and printer-specific errors
- **`utils.rs`** - Image processing utilities for converting bitmap data to printer format

### Key Design Patterns

#### USB Device Management
The `Printer::new()` method handles complex USB device initialization:
- Finds printers by vendor ID (0x04f9) and product ID
- Matches devices by serial number string
- Handles kernel driver detachment (varies by model)
- Configures USB endpoints for bulk transfers

#### Iterator-Based Printing
The `print()` method accepts any `Iterator<Item = Matrix>`, enabling lazy label generation:
```rust
printer.print(vec![label_data].into_iter())?; // Single label
printer.print(LabelGenerator { count: 5 })?;  // Multiple labels via custom iterator
```

#### Media Specification System
Each media type has detailed specifications defining:
- Physical dimensions (mm and dots)
- Print area offsets (left, effective width, right margins)
- Feed values and validation rules
- Status parsing from printer responses

### Configuration System

The `Config` struct uses builder pattern for printer settings:
- Media type and model (immutable after creation) 
- Resolution modes (normal/high)
- Auto-cut behavior and frequency
- Two-color printing support
- Compression settings

### Status Monitoring

The printer status system parses 32-byte responses containing:
- Current media type and dimensions
- Error states and phase information
- Model identification and notification status

### Enhanced Print Completion Handling

The library features improved print completion monitoring with smart status polling:

#### Key Features
- **Adaptive Status Polling**: Replaces fixed retry approach with intelligent monitoring
- **Phase Transition Monitoring**: Tracks Printing → Receiving state transitions
- **Immediate Error Detection**: Detects printer errors (cover open, media issues) during print jobs
- **Timeout Protection**: Prevents indefinite waiting with configurable timeouts
- **Detailed Logging**: Enhanced debug output for troubleshooting

#### Implementation Details
- `wait_for_print_completion()`: Core monitoring function with adaptive polling
- Applied to both 0x0C (Print) and 0x1A (Print then Eject) commands
- Error types: `PrintTimeout`, `UnexpectedPhase`, `PrinterError`
- Typical completion time: ~1.8s for standard labels (reduced from previous approach)

## Hardware Integration Notes

- **Kernel Driver Handling**: QL-800 requires kernel driver detachment, QL-820NWB does not
- **USB Timeout**: Long labels (>1000mm) may cause USB timeout errors
- **Media Detection**: Printer automatically detects installed media tape, verified against config
- **Two-Color Support**: Requires alternating raster line commands for red/black printing

### Print Command Flow
The library handles two main print commands:
- **0x0C (FF : Print)**: Used for intermediate pages in multi-page jobs
- **0x1A (Control-Z : Print then Eject)**: Used for final page with media ejection
- Both commands now use `wait_for_print_completion()` for reliable status monitoring

## Image Data Format

Labels must be provided as `Matrix` type (`Vec<Vec<u8>>`):
- Width: 720px for normal printers, 1296px for wide printers
- 1-bit bitmap data packed into bytes (8 pixels per byte)
- Use `step_filter_normal()` utility to convert grayscale images
- Content positioning depends on media width (see media specifications)