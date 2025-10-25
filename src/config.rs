use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub grid: GridConfig,
    #[serde(default)]
    pub observer: ObserverConfig,
    #[serde(default)]
    pub subcell: SubcellConfig,
    #[serde(default)]
    pub actors: ActorsConfig,
    #[serde(default)]
    pub visual: VisualConfig,
    #[serde(default)]
    pub default_grid_file: DefaultGridFileConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize)]
pub struct GridConfig {
    #[serde(default = "default_cols")]
    pub cols: i32,
    #[serde(default = "default_rows")]
    pub rows: i32,
    #[serde(default = "default_cell_width")]
    pub cell_width: f32,
    #[serde(default = "default_cell_height")]
    pub cell_height: f32,
}

#[derive(Debug, Deserialize)]
pub struct ObserverConfig {
    #[serde(default = "default_observer_x")]
    pub x: i32,
    #[serde(default = "default_observer_y")]
    pub y: i32,
    #[serde(default)]
    pub messy_x: bool,
    #[serde(default)]
    pub messy_y: bool,
}

#[derive(Debug, Deserialize)]
pub struct SubcellConfig {
    #[serde(default = "default_display_mode")]
    pub display_mode: String,
    #[serde(default)]
    pub movement_enabled: bool,
    #[serde(default)]
    pub show_markers: bool,
    #[serde(default = "default_reservation_mode")]
    pub reservation_mode: String,
    #[serde(default)]
    pub early_reservation_enabled: bool,
}

#[derive(Debug, Deserialize)]
pub struct ActorsConfig {
    #[serde(default = "default_actor_speed")]
    pub default_speed: f32,
    #[serde(default = "default_size_ratio")]
    pub size_ratio: f32,
    #[serde(default = "default_collision_radius_ratio")]
    pub collision_radius_ratio: f32,
}

#[derive(Debug, Deserialize)]
pub struct VisualConfig {
    #[serde(default = "default_window_title")]
    pub window_title: String,
    #[serde(default = "default_bg_r")]
    pub background_r: u8,
    #[serde(default = "default_bg_g")]
    pub background_g: u8,
    #[serde(default = "default_bg_b")]
    pub background_b: u8,
    #[serde(default = "default_show_corners")]
    pub show_corners: bool,
}

#[derive(Debug, Deserialize)]
pub struct DefaultGridFileConfig {
    #[serde(default = "default_grid_file_path")]
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_enable_action_log")]
    pub enable_action_log: bool,
    #[serde(default = "default_action_log_path")]
    pub action_log_path: String,
}

// Default values
fn default_cols() -> i32 { 40 }
fn default_rows() -> i32 { 40 }
fn default_cell_width() -> f32 { 20.0 }
fn default_cell_height() -> f32 { 15.0 }
fn default_observer_x() -> i32 { 20 }
fn default_observer_y() -> i32 { 20 }
fn default_display_mode() -> String { "none".to_string() }
fn default_reservation_mode() -> String { "Square".to_string() }
fn default_actor_speed() -> f32 { 120.0 }
fn default_size_ratio() -> f32 { 0.9 }
fn default_collision_radius_ratio() -> f32 { 0.3 }
fn default_window_title() -> String { "RustGame3 - Grid Raycasting Demo".to_string() }
fn default_bg_r() -> u8 { 30 }
fn default_bg_g() -> u8 { 30 }
fn default_bg_b() -> u8 { 30 }
fn default_show_corners() -> bool { true }
fn default_grid_file_path() -> String { "claude_tasks/default_grid_layout.txt".to_string() }
fn default_enable_action_log() -> bool { true }
fn default_action_log_path() -> String { "action_log.json".to_string() }

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            cols: default_cols(),
            rows: default_rows(),
            cell_width: default_cell_width(),
            cell_height: default_cell_height(),
        }
    }
}

impl Default for ObserverConfig {
    fn default() -> Self {
        Self {
            x: default_observer_x(),
            y: default_observer_y(),
            messy_x: false,
            messy_y: false,
        }
    }
}

impl Default for SubcellConfig {
    fn default() -> Self {
        Self {
            display_mode: default_display_mode(),
            movement_enabled: false,
            show_markers: false,
            reservation_mode: default_reservation_mode(),
            early_reservation_enabled: false,
        }
    }
}

impl Default for ActorsConfig {
    fn default() -> Self {
        Self {
            default_speed: default_actor_speed(),
            size_ratio: default_size_ratio(),
            collision_radius_ratio: default_collision_radius_ratio(),
        }
    }
}

impl Default for VisualConfig {
    fn default() -> Self {
        Self {
            window_title: default_window_title(),
            background_r: default_bg_r(),
            background_g: default_bg_g(),
            background_b: default_bg_b(),
            show_corners: default_show_corners(),
        }
    }
}

impl Default for DefaultGridFileConfig {
    fn default() -> Self {
        Self {
            path: default_grid_file_path(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enable_action_log: default_enable_action_log(),
            action_log_path: default_action_log_path(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            grid: GridConfig::default(),
            observer: ObserverConfig::default(),
            subcell: SubcellConfig::default(),
            actors: ActorsConfig::default(),
            visual: VisualConfig::default(),
            default_grid_file: DefaultGridFileConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from file, or use defaults if file doesn't exist
    pub fn load() -> Self {
        match fs::read_to_string("config.toml") {
            Ok(contents) => {
                match toml::from_str(&contents) {
                    Ok(config) => {
                        println!("Loaded configuration from config.toml");
                        config
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse config.toml: {}", e);
                        eprintln!("Using default configuration");
                        Config::default()
                    }
                }
            }
            Err(_) => {
                println!("No config.toml found, using default configuration");
                Config::default()
            }
        }
    }
}
