use std::time::{Duration, Instant};
use strum_macros::EnumIter;

use crate::{app_errors::AppResult, views::{AppDisplay, View, get_view_elements}};

pub struct AppViewModel {
    pub view: View,
    pub wifi_is_connected: bool,
    pub is_recording: bool,
    pub max_weight: u8,
    pub max_weight_avg: u8,
    pub language: Language,
    last_touch_processed: Instant,
}

pub trait AppDrawable {
    fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()>;
}

impl AppViewModel {
    pub fn new() -> Self {
        Self {
            view: View::Boot,
            wifi_is_connected: false,
            is_recording: false,
            max_weight: 0,
            max_weight_avg: 0,
            language: Language::De,
            last_touch_processed: Instant::now(),
        }
    }

    pub fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()>{
        let ui_elements = get_view_elements(&self)?;
        for element in ui_elements{
            element.draw(display)?;
        }
        Ok(())
    }

    pub fn on_touch(&mut self, x: u16, y: u16) -> bool {
        let elapsed = self.last_touch_processed.elapsed();
        if elapsed > Duration::from_secs(1) {
            match self.view {
                View::Boot => {
                    self.view = View::HomeScreen;
                }
                View::HomeScreen => {
                    if (8..248).contains(&x) && (8..54).contains(&y) {
                        self.view = View::LanguageSelection
                    };
                }
                _ => {}
            };
            self.last_touch_processed = Instant::now();
            return true;
        }
        false
    }

    pub fn on_start_training(){}
    pub fn on_connect_app(){}
    
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
