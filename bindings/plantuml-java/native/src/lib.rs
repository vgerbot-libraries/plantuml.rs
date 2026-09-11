//! JNI bridge for Java bindings.
//!
//! Exposes native methods consumed by `com.vgerbot.plantuml.PlantUml`.
//! Each call renders to a Java `String`; parse failures yield an empty string.
//!
//! Ported from: plantuml-jni (new, no Java source).

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::jstring;

/// `com.vgerbot.plantuml.PlantUml.renderSvg(String): String`
#[no_mangle]
pub extern "system" fn Java_com_vgerbot_plantuml_PlantUml_renderSvg(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    source: JString<'_>,
) -> jstring {
    let source: String = env.get_string(&source).unwrap().into();
    let svg = plantuml_engine::render_svg(&source).unwrap_or_default();
    env.new_string(svg)
        .map(jni::objects::JString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

/// `com.vgerbot.plantuml.PlantUml.renderPreproc(String): String`
#[no_mangle]
pub extern "system" fn Java_com_vgerbot_plantuml_PlantUml_renderPreproc(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    source: JString<'_>,
) -> jstring {
    let source: String = env.get_string(&source).unwrap().into();
    let text = plantuml_engine::render_preproc(&source).unwrap_or_default();
    env.new_string(text)
        .map(jni::objects::JString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}
