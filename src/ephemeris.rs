use num_enum::TryFromPrimitive;

/// CPFv2 Ephemeris Data
/// Currently only includes position entries (record type = 10).
#[derive(Debug, Clone)]
pub struct Ephemeris {
    /// None when parsed with assert_common_epoch_only
    pub direction_flag: Option<Vec<DirectionFlag>>,
    /// Modified Julian Date (MJD)
    pub mjd: Vec<i32>,
    pub seconds_of_day: Vec<f64>,
    /// None when parsed with assert_no_leap_second
    pub leap_second_flag: Option<Vec<i32>>,
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
