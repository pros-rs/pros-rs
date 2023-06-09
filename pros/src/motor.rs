use crate::{
    error::{FromErrno, PortError},
    position::Position,
};

/// The basic motor struct.
#[derive(Clone, Copy)]
pub struct Motor {
    port: u8,
}

//TODO: Implement good set_velocity and get_velocity functions.
//TODO: Measure the number of counts per rotation. Fow now we assume it is 4096
impl Motor {
    pub fn new(port: u8, brake_mode: BrakeMode) -> Result<Self, PortError> {
        unsafe {
            pros_sys::motor_set_encoder_units(
                port,
                pros_sys::motor_encoder_units_e_E_MOTOR_ENCODER_DEGREES,
            );
            pros_sys::motor_set_brake_mode(port, brake_mode.into());

            PortError::from_last_errno()?;
        }

        Ok(Self { port })
    }

    /// Takes in a f32 from -1 to 1 that is scaled to -12 to 12 volts.
    /// Useful for driving motors with controllers.
    pub fn set_output(&self, output: f32) {
        unsafe {
            pros_sys::motor_move(self.port, (output * 127.0) as i32);
        }
    }

    /// Takes in and i8 between -127 and 127 which is scaled to -12 to 12 Volts.
    pub fn set_raw_output(&self, raw_output: i8) {
        unsafe {
            pros_sys::motor_move(self.port, raw_output as i32);
        }
    }

    /// Takes in a voltage that must be between -12 and 12 Volts.
    pub fn set_voltage(&self, voltage: f32) -> Result<(), MotorError> {
        if voltage > 12.0 || voltage < -12.0 || voltage == f32::NAN {
            return Err(MotorError::VoltageOutOfRange);
        }
        unsafe {
            pros_sys::motor_move_voltage(self.port, (voltage * 1000.0) as i32);
        }

        Ok(())
    }

    /// Moves the motor to an absolute position, based off of the last motor zeroing.
    /// units for the velocity is RPM.
    pub fn set_position_absolute(&self, position: Position, velocity: i32) {
        unsafe {
            pros_sys::motor_move_absolute(self.port, position.into_degrees(), velocity);
        }
    }

    /// Moves the motor to a position relative to the current position.
    /// units for velocity is RPM.
    pub fn set_position_relative(&self, position: Position, velocity: i32) {
        unsafe {
            pros_sys::motor_move_relative(self.port, position.into_degrees(), velocity);
        }
    }

    /// Returns the power drawn by the motor in Watts.
    pub fn power(&self) -> f64 {
        unsafe { pros_sys::motor_get_power(self.port) }
    }

    /// Returns the toueque output of the motor in Nm.
    pub fn tourque(&self) -> f64 {
        unsafe { pros_sys::motor_get_torque(self.port) }
    }

    /// Returns the voltage the motor is drawing in volts.
    pub fn voltage(&self) -> f64 {
        unsafe { pros_sys::motor_get_voltage(self.port) as f64 / 1000.0 }
    }

    /// Returns the current position of the motor.
    pub fn position(&self) -> Position {
        unsafe { Position::from_degrees(pros_sys::motor_get_position(self.port)) }
    }

    /// Returns the current draw of the motor.
    pub fn current_draw(&self) -> i32 {
        unsafe { pros_sys::motor_get_current_draw(self.port) }
    }

    /// Sets the current position to zero.
    pub fn zero(&self) {
        unsafe {
            pros_sys::motor_set_zero_position(self.port, self.position().into_degrees());
        }
    }

    /// Stops the motor based on the current [`BrakeMode`]
    pub fn brake(&self) {
        unsafe {
            pros_sys::motor_brake(self.port);
        }
    }

    /// Sets the current position to the given position.
    pub fn set_zero_position(&self, position: Position) {
        unsafe {
            pros_sys::motor_set_zero_position(self.port, position.into_degrees());
        }
    }

    /// Sets how the motor should act when stopping.
    pub fn set_brake_mode(&self, brake_mode: BrakeMode) {
        unsafe {
            pros_sys::motor_set_brake_mode(self.port, brake_mode.into());
        }
    }

    //TODO: Test this, as im not entirely sure of the actuall implementation
    /// Get the current state of the motor.
    pub fn get_state(&self) -> MotorState {
        unsafe { (pros_sys::motor_get_flags(self.port) as u32).into() }
    }
}

/// Determines how a motor should act when braking.
pub enum BrakeMode {
    /// Motor never brakes.
    None,
    /// Motor brakes when stopped.
    Brake,
    /// Motor exerts force to hold the same position.
    Hold,
}

impl Into<pros_sys::motor_brake_mode_e_t> for BrakeMode {
    fn into(self) -> pros_sys::motor_brake_mode_e_t {
        match self {
            Self::Brake => pros_sys::motor_brake_mode_e_E_MOTOR_BRAKE_BRAKE,
            Self::Hold => pros_sys::motor_brake_mode_e_E_MOTOR_BRAKE_HOLD,
            Self::None => pros_sys::motor_brake_mode_e_E_MOTOR_BRAKE_COAST,
        }
    }
}

/// Represents what the physical motor is currently doing.
pub struct MotorState {
    pub busy: bool,
    pub stopped: bool,
    /// the motor is at zero encoder units of rotation.
    pub zeroed: bool,
}

//TODO: Test this, like mentioned above
impl From<u32> for MotorState {
    fn from(value: u32) -> Self {
        Self {
            busy: (value & 0b001) == 0b001,
            stopped: (value & 0b010) == 0b010,
            zeroed: (value & 0b100) == 0b100,
        }
    }
}

#[derive(Debug)]
pub enum MotorError {
    VoltageOutOfRange,
}
impl core::error::Error for MotorError {}

impl core::fmt::Display for MotorError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Self::VoltageOutOfRange =>
                    "The voltage supplied was outside of the allowed range (-12 to 12)!",
            }
        )
    }
}
