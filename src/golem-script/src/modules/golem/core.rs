use std::path::PathBuf;

use boa_engine::value::TryFromJs;
use boa_engine::{js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use boa_macros::{Finalize, JsData, Trace};
use one_fpga::core::Rom;
use one_fpga::runner::CoreLaunchInfo;

use golem_ui::application::panels::core_loop::run_core_loop;

use crate::HostData;

/// The core type from JavaScript.
#[derive(Debug, Trace, Finalize, JsData)]
pub enum CoreType {
    Path { path: JsString },
}

impl TryFromJs for CoreType {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Object(object) => {
                match object
                    .get(js_string!("type"), context)?
                    .to_string(context)?
                    .to_std_string_escaped()
                    .as_str()
                {
                    "path" => {
                        let path = object
                            .get(js_string!("path"), context)?
                            .to_string(context)?;
                        Ok(CoreType::Path { path })
                    }
                    _ => Err(JsError::from_opaque(
                        js_string!("Invalid core type.").into(),
                    )),
                }
            }
            _ => Err(JsError::from_opaque(
                js_string!("Invalid core type.").into(),
            )),
        }
    }
}

/// The game type for JavaScript.
#[derive(Debug, Trace, Finalize, JsData)]
pub enum GameType {
    RomPath { path: JsString },
}

impl TryFromJs for GameType {
    fn try_from_js(value: &JsValue, context: &mut Context) -> JsResult<Self> {
        match value {
            JsValue::Object(object) => {
                match object
                    .get(js_string!("type"), context)?
                    .to_string(context)?
                    .to_std_string_escaped()
                    .as_str()
                {
                    "rom-path" => {
                        let path = object
                            .get(js_string!("path"), context)?
                            .to_string(context)?;
                        Ok(GameType::RomPath { path })
                    }
                    _ => Err(JsError::from_opaque(
                        js_string!("Invalid core type.").into(),
                    )),
                }
            }
            _ => Err(JsError::from_opaque(
                js_string!("Invalid core type.").into(),
            )),
        }
    }
}

#[derive(Debug, Trace, Finalize, JsData, TryFromJs)]
struct RunOptions {
    core: CoreType,
    game: Option<GameType>,
    files: Option<Vec<Option<String>>>,
    savestate: Option<String>,
    showmenu: Option<bool>,
    autoloop: Option<bool>,
}

fn run_(options: RunOptions, ContextData(app): ContextData<HostData>) {
    let app = app.app_mut();
    let mut core_options = match &options.core {
        CoreType::Path { path } => CoreLaunchInfo::rbf(PathBuf::from(path.to_std_string_escaped())),
    };

    match &options.game {
        Some(GameType::RomPath { path }) => {
            core_options =
                core_options.with_rom(Rom::File(PathBuf::from(path.to_std_string_escaped())));
        }
        None => {}
    };

    if let Some(files) = &options.files {
        for (i, file) in files
            .iter()
            .enumerate()
            .filter_map(|(i, x)| x.as_ref().map(|x| (i, x)))
        {
            core_options
                .files
                .insert(i, one_fpga::runner::Slot::File(PathBuf::from(file)));
        }
    }

    eprintln!("Launching core: {:?}", core_options);
    let mut core = app
        .platform_mut()
        .core_manager_mut()
        .launch(core_options)
        .unwrap();

    if options.autoloop.unwrap_or(true) {
        run_core_loop(&mut *app, &mut core, options.showmenu.unwrap_or(true));
    }
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("core"),
        [(js_string!("run"), run_.into_js_function_copied(context))].into_js_module(context),
    ))
}
