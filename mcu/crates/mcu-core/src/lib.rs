//! Core MCU functionality shared across all firmware binaries.
//!
//! Provides CAN communication, watchdog, and heartbeat for RP2040/RP2350 MCUs.

#![no_std]

pub mod can;
pub mod heartbeat;
pub mod protocol;
pub mod watchdog;

