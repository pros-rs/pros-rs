use pros_sys::PROS_ERR;
use snafu::Snafu;

use crate::error::{bail_on, map_errno, PortError};

pub struct GpsStatus {
    pub x: f64,
    pub y: f64,
    pub pitch: f64,
    pub roll: f64,
    pub yaw: f64,
    pub heading: f64,

    pub accel_x: f64,
    pub accel_y: f64,
    pub accel_z: f64,
}

pub struct GpsSensor {
    port: u8,
}

impl GpsSensor {
    pub fn new(port: u8) -> Result<Self, GpsError> {
        unsafe {
            bail_on!(
                PROS_ERR,
                pros_sys::gps_initialize_full(port, 0.0, 0.0, 0.0, 0.0, 0.0)
            );
        }

        Ok(Self { port })
    }

    pub fn set_offset(&self, x: f64, y: f64) {
        unsafe {
            pros_sys::gps_set_offset(self.port, x, y);
        }
    }

    pub fn error(&self) -> f64 {
        unsafe { pros_sys::gps_get_error(self.port) }
    }

    pub fn status(&self) -> GpsStatus {
        unsafe {
            let status = pros_sys::gps_get_status(self.port);
            let accel = pros_sys::gps_get_accel(self.port);
            let heading = pros_sys::gps_get_heading(self.port);

            GpsStatus {
                x: status.x,
                y: status.y,
                pitch: status.pitch,
                roll: status.roll,
                yaw: status.yaw,
                heading,

                accel_x: accel.x,
                accel_y: accel.y,
                accel_z: accel.z,
            }
        }
    }

    pub fn zero_rotation(&self) {
        unsafe {
            pros_sys::gps_tare_rotation(self.port);
        }
    }
}

#[derive(Debug, Snafu)]
pub enum GpsError {
    #[snafu(display("GPS sensor is still calibrating."))]
    StillCalibrating,
    #[snafu(display("{source}"), context(false))]
    Port { source: PortError },
}
impl core::error::Error for GpsError {}

map_errno! {
    GpsError {
        EAGAIN => Self::StillCalibrating,
    }
    inherit PortError;
}
