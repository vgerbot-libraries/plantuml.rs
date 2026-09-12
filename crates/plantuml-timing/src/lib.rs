//! Timing diagram for plantuml.rs.
//!
//! Ported from: `net/sourceforge/plantuml/timingdiagram/` (36 files).
//!
//! Timing diagrams render digital/analog signals as horizontal waveforms.
//! Syntax:
//! ```text
//! @starttiming
//! binary A
//! binary B
//! A = 0
//! B = 0
//! A = 1
//! B = 1
//! @endtiming
//! ```

pub mod timing_diagram;
pub mod timing_parser;
pub mod timing_renderer;

pub use timing_diagram::TimingDiagram;
pub use timing_parser::{parse_timing_source, Signal, SignalChange, SignalType, TimingSource};
