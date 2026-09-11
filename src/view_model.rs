use std::collections::VecDeque;

use strum_macros::EnumIter;

use crate::{
    app_errors::AppResult,
    views::{get_view_elements, AppDisplay, View},
};

pub enum ActionId {
    None,
    Default,
    StartRecording,
    LanguageSelection,
    StartTraining,
    ConnectApp,
    Wifi,
    Start,
    Globe,
    Stop,
    Exit,
}

pub struct AppViewModel {
    pub view: View,
    pub wifi_is_connected: bool,
    pub is_recording: bool,
    pub current_reading: f32,
    pub past_readings: VecDeque<f32>,
    pub max_weight: f32,
    pub max_weight_avg: f32,
    pub language: Language,
    touch_active: bool,
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
            current_reading: 0.0,
            past_readings: vec![0.0_f32; 20].into(),
            max_weight: 0.0,
            max_weight_avg: 0.0,
            language: Language::De,
            touch_active: false,
        }
    }

    pub fn draw(&self, display: &mut AppDisplay<'_>) -> AppResult<()> {
        let ui_elements = get_view_elements(&self)?;
        for element in ui_elements {
            element.draw(display)?;
        }
        Ok(())
    }

    pub fn on_touch(&mut self, event: Option<(u16, u16)>) -> AppResult<bool> {
        match event {
            Some((x, y)) => {
                if self.touch_active {
                    return Ok(false);
                }
                self.touch_active = true;
                let ui_elements = get_view_elements(self)?;
                for element in ui_elements {
                    if element.eval_touch(&x.into(), &y.into()) {
                        let (id, lang) = element.get_id();
                        self.execute_action(id, lang);
                        return Ok(true);
                    }
                }
            }
            None => {
                self.touch_active = false;
            }
        }
        Ok(false)
    }

    fn execute_action(&mut self, button_id: &ActionId, lang: &Option<Language>) {
        match button_id {
            ActionId::Default => {
                self.view = View::HomeScreen;
            }
            ActionId::ConnectApp => {
                self.view = View::ConnectApp;
            }
            ActionId::Globe => {
                self.view = View::LanguageSelection;
            }
            ActionId::StartTraining => {
                self.view = View::Recording;
            }
            ActionId::Wifi => {}
            ActionId::LanguageSelection => {
                self.language = lang.clone().unwrap_or(Language::De);
                self.view = View::HomeScreen;
            }
            ActionId::Start => {
                self.past_readings = vec![0.0_f32; 20].into();
                self.max_weight = 0.0;
                self.max_weight_avg = 0.0;
                self.is_recording = true;
            }
            ActionId::Stop => {
                self.is_recording = false;
                self.view = View::Statistics;
            }
            ActionId::Exit => {
                self.view = View::HomeScreen;
            }
            ActionId::None => {}
            _ => {}
        };
    }
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
                Language::De => "Maximales\nGewicht:",
                Language::En => "Maximum\nweight:",
            },
            "max_weight_avg" => match self {
                Language::De => "Maximales\nGewicht 4s:",
                Language::En => "Maximum\nweight 4s:",
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
