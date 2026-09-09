use std::time::{Duration, Instant};
use strum_macros::EnumIter;

use crate::{
    app_errors::AppResult,
    views::{get_view_elements, AppDisplay, View},
};

pub enum ActionId {
    Default,
    StartRecording,
    LanguageSelection,
    StartTraining,
    ConnectApp,
    Wifi,
    Start,
    Globe,
}

pub struct AppViewModel {
    pub view: View,
    pub wifi_is_connected: bool,
    pub is_recording: bool,
    pub current_reading: u16,
    pub max_weight: u16,
    pub max_weight_avg: u16,
    pub language: Language,
    last_touch_processed: Instant,
}

pub trait AppDrawable {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()>;
    fn eval_touch(&self, x: &u32, y: &u32) -> bool;
    fn get_id(&self) -> (&ActionId, &Option<Language>);
}

impl AppViewModel {
    pub fn new() -> Self {
        Self {
            view: View::Boot,
            wifi_is_connected: false,
            is_recording: false,
            current_reading: 0,
            max_weight: 0,
            max_weight_avg: 0,
            language: Language::De,
            last_touch_processed: Instant::now(),
        }
    }

    pub fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        let ui_elements = get_view_elements(&self)?;
        for element in ui_elements {
            element.draw(display)?;
        }
        Ok(())
    }

    pub fn on_touch(&mut self, x: u16, y: u16) -> AppResult<(bool)> {
        let elapsed = self.last_touch_processed.elapsed();
        if elapsed > Duration::from_millis(300) {
            let ui_elements = get_view_elements(&mut *self)?;
            for element in ui_elements {
                if element.eval_touch(&x.into(), &y.into()) {
                    let (button_id, lang) = element.get_id();
                    self.execute_action(button_id, lang);
                    self.last_touch_processed = Instant::now();
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn execute_action(&mut self, button_id: &ActionId, lang: &Option<Language>) {
        match button_id {
            ActionId::Default => {
                println!("switching to HomeScreen screen");
                self.view = View::HomeScreen;
            }
            ActionId::ConnectApp => {
                println!("switching to ConnectApp screen");
                self.view = View::ConnectApp;
            }
            ActionId::Globe => {
                println!("switching to LanguageSelection screen");
                self.view = View::LanguageSelection;
            }
            ActionId::StartTraining => {
                println!("switching to Recording screen");
                self.view = View::Recording;
            }
            ActionId::Wifi => {
                self.wifi_is_connected = !self.wifi_is_connected;
            }
            ActionId::LanguageSelection => {
                self.language = lang.clone().unwrap_or(Language::De);
                self.view = View::HomeScreen;
            }
            _ => {}
        };
    }

    pub fn on_start_training() {}
    pub fn on_connect_app() {}
}

pub struct TouchZone {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    on_touch: Box<dyn Fn()>,
}

impl TouchZone {
    pub fn new(x: u32, y: u32, width: u32, height: u32, on_touch: Box<dyn Fn()>) -> Self {
        Self {
            x,
            y,
            width,
            height,
            on_touch,
        }
    }

    pub fn eval_touch(&mut self, x: &u32, y: &u32) {
        if (self.x..(self.x + self.width)).contains(x)
            && (self.y..(self.y + self.height)).contains(y)
        {
            (self.on_touch)();
        }
    }
}

#[derive(EnumIter, PartialEq, Clone)]
pub enum Language {
    De,
    En,
}

impl Language {
    pub fn get_str(&self, key: &str) -> &str {
        match key {
            "start_training" => match self {
                Language::De => "Training starten",
                Language::En => "start training",
            },
            "connect_app" => match self {
                Language::De => "App verbinden",
                Language::En => "connect app",
            },
            "ready" => match self {
                Language::De => "Alles bereit, du kannst die Aufzeichnung starten",
                Language::En => "Everything is ready, you can start recording",
            },
            "max_weight" => match self {
                Language::De => "Maximales Gewicht",
                Language::En => "Maximum weight",
            },
            "max_weight_avg" => match self {
                Language::De => "Maximales Gewicht 5s",
                Language::En => "Maximum weight 5s",
            },
            "lang_name" => match self {
                Language::De => "Deutsch",
                Language::En => "English",
            },
            "back" => match self {
                Language::De => "zurück",
                Language::En => "back",
            },
            "kg" => match self {
                Language::De => "kg",
                Language::En => "kg",
            },
            "scan_me" => match self {
                Language::De => "Scanne den QR-Code in der OpenGripBoard-App",
                Language::En => "Scan the QR-code with the OpenGripBoard-app",
            },
            _ => "key not found",
        }
    }
}
