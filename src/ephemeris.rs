use num_enum::TryFromPrimitive;

/// CPFv2 Ephemeris Data
/// Currently only includes position entries (record type = 10).
#[derive(Debug, Clone)]
pub struct Ephemeris {
    pub direction_flag: Vec<DirectionFlag>,
    /// Modified Julian Date (MJD)
    pub mjd: Vec<i32>,
    pub seconds_of_day: Vec<f64>,
    /// Leap second flag (0 or the value of the new leap second)
    pub leap_second_flag: Vec<i32>,
    /// Positions in meters (x, y, z)
    pub position_m: Vec<[f64; 3]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
#[repr(u8)]
pub enum DirectionFlag {
    /// Common epoch: instantaneous vector between geocenter and target,
    /// without light-time iteration
    CommonEpoch = 0,
    /// Transmit: position vector contains light-time iterated travel time
    /// from the geocenter to the target at the transmit epoch
    Transmit = 1,
    /// Receive: position vector contains light-time iterated travel time
    /// from the target to the geocenter at the receive epoch
    Receive = 2,
}

impl Ephemeris {
    /// Push position record entry
    pub fn push_position(
        &mut self,
        direction_flag: DirectionFlag,
        mjd: i32,
        seconds_of_day_utc: f64,
        leap_second_flag: i32,
        position: [f64; 3],
    ) {
        self.direction_flag.push(direction_flag);
        self.mjd.push(mjd);
        self.seconds_of_day.push(seconds_of_day_utc);
        self.leap_second_flag.push(leap_second_flag);
        self.position_m.push(position);
    }
}
