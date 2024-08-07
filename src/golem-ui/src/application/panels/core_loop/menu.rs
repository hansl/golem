use crate::application::menu::style::MenuReturn;
use crate::application::menu::{text_menu, IntoTextMenuItem, TextMenuOptions};
use crate::application::panels::core_loop::menu::core_settings::{
    execute_core_settings, into_text_menu_item,
};
use crate::application::GoLEmApp;
use mister_fpga::core::MisterFpgaCore;
use one_fpga::{Core, GolemCore};
use tracing::error;

mod core_debug;
mod core_settings;
mod items;

#[derive(Debug, Copy, Clone, PartialEq)]
enum CoreMenuAction {
    Reset,
    CoreSettings,
    CoreMenuAction(core_settings::CoreMenuAction),
    InputMapping,
    DebugMenu,
    Back,
    Quit,

    Unselectable,
}

impl MenuReturn for CoreMenuAction {
    fn is_selectable(&self) -> bool {
        *self != Self::Unselectable
    }

    fn back() -> Option<Self> {
        Some(Self::Back)
    }
}

/// Shows the core menu and interact with it.
/// This will return `true` if the user decided to quit the core (in which case the
/// main MENU should be reloaded).
pub fn core_menu(app: &mut GoLEmApp, core: &mut GolemCore) -> bool {
    app.platform_mut().core_manager_mut().show_menu();

    let Some(c) = core.as_any_mut().downcast_mut::<MisterFpgaCore>() else {
        error!("Core is not a MisterFPGA core.");
        return false;
    };

    // Update saves.
    loop {
        match c.poll_mounts() {
            Ok(true) => {}
            Ok(false) => break,
            Err(e) => {
                error!(?e, "Error updating the SD card.");
                break;
            }
        }
    }

    let mut state = None;

    let result = loop {
        let status = c.status_bits();
        let mut additional_items = c
            .menu_options()
            .iter()
            .filter(|o| o.as_load_file().is_some())
            .filter_map(|i| into_text_menu_item(i, &status))
            .map(|i| i.map_action(CoreMenuAction::CoreMenuAction))
            .chain([
                ("-", "", CoreMenuAction::Unselectable).to_menu_item(),
                ("Input Mapping", "", CoreMenuAction::InputMapping).to_menu_item(),
                ("Debug", "", CoreMenuAction::DebugMenu).to_menu_item(),
            ])
            .collect::<Vec<_>>();

        let version = c
            .config()
            .version()
            .map(|s| ("Version", s, CoreMenuAction::Unselectable));
        let (result, new_state) = text_menu(
            app,
            "Core",
            &mut additional_items,
            TextMenuOptions::default()
                .with_state(state)
                .with_prefix(&[
                    ("Reset Core", "", CoreMenuAction::Reset),
                    ("Core Settings", "", CoreMenuAction::CoreSettings),
                ])
                .with_suffix(&[
                    version.unwrap_or(("", "", CoreMenuAction::Unselectable)),
                    ("Back", "<-", CoreMenuAction::Back),
                    ("Quit Core", "", CoreMenuAction::Quit),
                ])
                .with_back_menu(false),
        );
        state = Some(new_state);

        match result {
            CoreMenuAction::Reset => {
                c.status_pulse(0);
            }
            CoreMenuAction::CoreSettings => {
                if core_settings::core_settings(app, c) {
                    break false;
                }
            }
            CoreMenuAction::DebugMenu => {
                core_debug::debug_menu(app, c);
            }
            CoreMenuAction::InputMapping => {
                todo!();
            }
            CoreMenuAction::CoreMenuAction(action) => {
                if let Some(_) = execute_core_settings(app, c, action) {
                    break false;
                }
            }
            CoreMenuAction::Back => {
                break false;
            }
            CoreMenuAction::Quit => {
                break true;
            }

            CoreMenuAction::Unselectable => unreachable!(),
        }
    };

    app.platform_mut().core_manager_mut().hide_menu();
    result
}
