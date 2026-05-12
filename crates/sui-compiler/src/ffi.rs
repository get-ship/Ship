//! C ABI for Bun FFI. Returns JSON-encoded compile result; caller frees.

use std::ffi::{c_char, CStr, CString};
use std::fs;
use std::path::{Path, PathBuf};

use crate::{compile, CompileMode, ComponentMeta, CompileOpts};

/// Compile a .sui file by path, writing outputs to out_dir.
///
/// Returns JSON: `{"ok":true,"outputs":[...]}` or `{"ok":false,"error":"..."}`.
/// Caller MUST free the returned pointer via `sui_compiler_free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn sui_compiler_compile(
    path: *const c_char,
    component_name: *const c_char,
    out_dir: *const c_char,
) -> *mut c_char {
    let json = match std::panic::catch_unwind(|| do_compile(path, component_name, out_dir)) {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            let msg = serde_json::to_string(&e).unwrap_or_else(|_| "\"compile error\"".into());
            format!(r#"{{"ok":false,"error":{msg}}}"#)
        }
        Err(_) => r#"{"ok":false,"error":"panic in compile"}"#.into(),
    };
    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"internal"}"#).unwrap())
        .into_raw()
}

/// Free a string returned by `sui_compiler_compile`.
#[unsafe(no_mangle)]
pub extern "C" fn sui_compiler_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: ptr was produced by CString::into_raw in sui_compiler_compile.
    unsafe { drop(CString::from_raw(ptr)) }
}

fn do_compile(
    path: *const c_char,
    component_name: *const c_char,
    out_dir: *const c_char,
) -> Result<String, String> {
    // SAFETY: pointers originate from Bun FFI and are valid for the duration of this call.
    let path_str = unsafe { CStr::from_ptr(path) }
        .to_str()
        .map_err(|e| e.to_string())?;
    let out_dir_str = unsafe { CStr::from_ptr(out_dir) }
        .to_str()
        .map_err(|e| e.to_string())?;
    let component_name_opt = if component_name.is_null() {
        None
    } else {
        Some(
            unsafe { CStr::from_ptr(component_name) }
                .to_str()
                .map_err(|e| e.to_string())?,
        )
    };

    let sui_path = PathBuf::from(path_str);
    let out_dir_path = Path::new(out_dir_str);

    let source = fs::read_to_string(&sui_path)
        .map_err(|e| format!("read {}: {e}", sui_path.display()))?;

    let file_name = sui_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let stem = file_name
        .strip_suffix(".html.set")
        .or_else(|| file_name.strip_suffix(".sui"))
        .unwrap_or(file_name);

    let setup_url = paired_setup(&sui_path);
    let component_meta = component_name_opt.map(|name| ComponentMeta {
        name: name.to_string(),
        shadow: true,
    });
    let mode = if component_meta.is_some() {
        CompileMode::Component
    } else {
        CompileMode::Auto
    };

    let opts = CompileOpts {
        setup_url,
        mode,
        source_path: Some(sui_path.clone()),
        styles: None,
        component_meta,
        emit_dts: false,
        emit_ssr: true,
    };

    let output = compile(&source, opts).map_err(|e| e.render(&source))?;

    fs::create_dir_all(out_dir_path).map_err(|e| e.to_string())?;

    let js_path = out_dir_path.join(format!("{stem}.compiled.js"));
    fs::write(&js_path, &output.js)
        .map_err(|e| format!("write {}: {e}", js_path.display()))?;

    let mut outputs = vec![js_path.to_string_lossy().into_owned()];

    if let Some(cem) = output.cem {
        let p = out_dir_path.join(format!("{stem}.custom-elements.json"));
        fs::write(&p, serde_json::to_string_pretty(&cem).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        outputs.push(p.to_string_lossy().into_owned());
    }
    if let Some(schema) = output.schema {
        let p = out_dir_path.join(format!("{stem}.schema.json"));
        fs::write(&p, serde_json::to_string_pretty(&schema).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        outputs.push(p.to_string_lossy().into_owned());
    }
    if let Some(meta) = output.meta {
        let p = out_dir_path.join(format!("{stem}.sui.meta.json"));
        fs::write(&p, serde_json::to_string_pretty(&meta).unwrap_or_default())
            .map_err(|e| e.to_string())?;
        outputs.push(p.to_string_lossy().into_owned());
    }
    if let Some(ssr_html) = output.ssr_html {
        let p = out_dir_path.join(format!("{stem}.ssr.html"));
        fs::write(&p, ssr_html).map_err(|e| e.to_string())?;
        outputs.push(p.to_string_lossy().into_owned());
    }

    Ok(serde_json::json!({"ok": true, "outputs": outputs}).to_string())
}

fn paired_setup(path: &Path) -> Option<String> {
    let file_name = path.file_name()?.to_str()?;
    let stem = file_name
        .strip_suffix(".html.set")
        .or_else(|| file_name.strip_suffix(".sui"))?;
    let parent = path.parent()?;
    let filename = if stem == "page" {
        "controller.ts".to_string()
    } else {
        format!("{stem}.controller.ts")
    };
    if !parent.join(&filename).exists() {
        return None;
    }
    Some(format!("../{filename}"))
}
