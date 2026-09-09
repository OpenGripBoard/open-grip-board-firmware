use crate::views::View;

pub struct AppViewModel {
    pub view: View,
    pub wifi_is_connected: bool,
    pub is_recording: bool,
    pub max_weight: u8,
    pub max_weight_avg: u8,
    pub language: Language,
}

pub enum Language {
    De,
    En,
}

impl Language {
    fn get_str(&self, key: &str) -> &str {
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
