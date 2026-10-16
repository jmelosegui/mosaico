pub mod action;
pub mod autostart;
pub mod banner;
pub mod context;
pub mod daemon;
pub mod debug;
pub mod doctor;
mod doctor_animations;
mod doctor_runtime;
pub mod init;
pub mod pause;
mod request;
pub mod start;
pub mod status;
pub mod stop;
pub mod update;

#[cfg(test)]
mod fake_desktop;
#[cfg(test)]
mod test_support;
