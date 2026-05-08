use serde::{Deserialize, Serialize};

use super::RgbColor;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GridVisualConfig {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub cell_size: Option<u16>,
    #[serde(default)]
    pub color: Option<RgbColor>,
    #[serde(default)]
    pub major_every: Option<u8>,
    #[serde(default)]
    pub major_color: Option<RgbColor>,
}

impl Default for GridVisualConfig {
    fn default() -> Self {
        Self {
            enabled: Some(false),
            cell_size: Some(40),
            color: Some(RgbColor::rgba(255, 255, 255, 20)),
            major_every: Some(4),
            major_color: Some(RgbColor::rgba(255, 255, 255, 38)),
        }
    }
}

impl super::VisualConfig {
    pub fn grid_enabled(&self) -> bool {
        self.grid
            .as_ref()
            .and_then(|grid| grid.enabled)
            .unwrap_or(false)
    }

    pub fn grid_cell_size(&self) -> u16 {
        self.grid
            .as_ref()
            .and_then(|grid| grid.cell_size)
            .unwrap_or(40)
    }

    pub fn grid_color(&self) -> Option<RgbColor> {
        self.grid
            .as_ref()
            .and_then(|grid| grid.color)
            .or(Some(RgbColor::rgba(255, 255, 255, 20)))
    }

    pub fn grid_major_every(&self) -> u8 {
        self.grid
            .as_ref()
            .and_then(|grid| grid.major_every)
            .unwrap_or(4)
    }

    pub fn grid_major_color(&self) -> Option<RgbColor> {
        self.grid
            .as_ref()
            .and_then(|grid| grid.major_color)
            .or(Some(RgbColor::rgba(255, 255, 255, 38)))
    }
}
