#![no_std]
#![no_main]

use core::time::Duration;

use pros::{devices::smart::SmartPort, prelude::*};

#[derive(Default)]
pub struct Robot;

impl SyncRobot for Robot {
    fn opcontrol(&mut self) -> pros::Result {
        let mut imu = InertialSensor::new(unsafe { SmartPort::new(1) })?;

        imu.calibrate_blocking()?;

        loop {
            let euler = imu.euler()?;

            println!(
                "Pitch: {} Roll: {} Yaw: {}",
                euler.pitch, euler.roll, euler.yaw
            );

            pros::task::delay(Duration::from_secs(1));
        }
    }
}

sync_robot!(Robot);
