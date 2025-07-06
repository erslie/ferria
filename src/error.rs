
use std::io::ErrorKind;
use ratatui::crossterm;
use thiserror::Error;

#[derive(Error, Debug)]

pub enum FerriaError {

    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("APPError: {0}")]
    APPError(String),

    #[error("Audio Error: {0}")]
    AudioError(String),

    #[error("analyzer Error: {0}")]
    AnalyzerError(String),

    #[error("Visualizer Error: {0}")]
    VisualizerError(String),

    #[error("CLI Error: {0}")]
    CliError(#[from] clap::Error),

}