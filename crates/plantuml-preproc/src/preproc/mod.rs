//! Legacy preprocessor module.
//!
//! Ported from `net.sourceforge.plantuml.preproc`.

pub mod comment_emoji;
pub mod configuration_store;
pub mod define;
pub mod define_signature;
pub mod define_variable;
pub mod defines;
pub mod diagram_detector;
pub mod diagram_extractor;
pub mod eval_boolean;
pub mod eval_math;
pub mod future_image;
pub mod numeric_compare;
pub mod option_key;
pub mod preprocessing_artifact;
pub mod read_line;
pub mod read_line_concat;
pub mod read_line_list;
pub mod read_line_numbered;
pub mod read_line_reader;
pub mod read_line_simple;
pub mod read_line_with_yaml_header;
pub mod spm;
pub mod stdlib;
pub mod stdlib_sprite;
pub mod stdlib_sprite_svg;
pub mod sub;
pub mod truth;
pub mod uncomment_read_line;
pub mod variables;

// Re-export key types
pub use comment_emoji::CommentEmoji;
pub use configuration_store::ConfigurationStore;
pub use define::Define;
pub use define_signature::DefineSignature;
pub use define_variable::DefineVariable;
pub use defines::Defines;
pub use diagram_detector::DiagramDetector;
pub use diagram_extractor::DiagramExtractor;
pub use eval_boolean::EvalBoolean;
pub use eval_math::EvalMath;
pub use future_image::FutureImage;
pub use numeric_compare::NumericCompare;
pub use option_key::OptionKey;
pub use preprocessing_artifact::PreprocessingArtifact;
pub use read_line::ReadLine;
pub use read_line_concat::ReadLineConcat;
pub use read_line_list::ReadLineList;
pub use read_line_numbered::ReadLineNumbered;
pub use read_line_reader::ReadLineReader;
pub use read_line_simple::ReadLineSimple;
pub use read_line_with_yaml_header::ReadLineWithYamlHeader;
pub use spm::SpmChannel;
pub use stdlib::Stdlib;
pub use stdlib_sprite::StdlibSprite;
pub use stdlib_sprite_svg::StdlibSpriteSvg;
pub use sub::Sub;
pub use truth::Truth;
pub use uncomment_read_line::UncommentReadLine;
pub use variables::Variables;
