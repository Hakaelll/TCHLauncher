use tauri::{
    menu::{Menu, MenuItem},
    App, Emitter, Manager, State, Wry,
};

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Es,
    En,
}
impl Language {
    pub fn text(self,spanish:&'static str,english:&'static str)->&'static str {
        match self {Self::Es=>spanish,Self::En=>english}
    }
    fn labels(self) -> (&'static str, &'static str) {
        (self.text("Mostrar TCHLauncher","Show TCHLauncher"),self.text("Salir","Quit"))
    }
}

pub struct TrayMenu {
    show: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

#[tauri::command]
pub fn set_application_language(
    menu: State<'_, TrayMenu>,
    language: Language,
) -> Result<(), String> {
    let (show, quit) = language.labels();
    menu.show
        .set_text(show)
        .map_err(|error| error.to_string())?;
    menu.quit.set_text(quit).map_err(|error| error.to_string())
}

pub fn install(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let (show_label, quit_label) = Language::Es.labels();
    let show = MenuItem::with_id(app, "show", show_label, true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", quit_label, true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &quit])?;
    tauri::tray::TrayIconBuilder::new()
        .icon(
            app.default_window_icon()
                .ok_or("Icono de ventana ausente")?
                .clone(),
        )
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "show" => crate::show_main(app),
            "quit" => {
                if app.state::<crate::Engine>().processes.any_active() {
                    crate::show_main(app);
                    let _ = app.emit("close-with-servers", ());
                } else {
                    app.exit(0);
                }
            }
            _ => (),
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                crate::show_main(tray.app_handle());
            }
        })
        .build(app)?;
    app.manage(TrayMenu { show, quit });
    Ok(())
}
