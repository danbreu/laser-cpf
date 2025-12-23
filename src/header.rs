use num_enum::TryFromPrimitive;

/// CPFv2 Header
/// Currently only includes required headers (record type H1, H2)
#[derive(Debug, Clone, Default)]
pub struct Header {
    /// Ephemeris Source (e.g., "HON", "UTX ")
    pub ephemeris_source: String,

    pub production_year_utc: i32,
    pub production_month_utc: i32,
    pub production_day_utc: i32,
    pub production_hour_utc: i32,

    pub sequence_number: i32,
    pub sub_daily_sequence: i32,

    /// Target name from official ILRS list (e.g. lageos1)
    pub target_name: String,
    /// Notes from the first CPF line (H1 record)
    /// e.g., "041202","DE-403" with no spaces
    pub notes: Option<String>,

    /// ILRS Satellite ID (Based on COSPAR ID)
    pub ilrs_satellite_id: i32,
    /// SIC (Provided by ILRS; set to "-1" for targets without SIC)
    pub sic: i32,
    /// NORAD ID (i.e., Satellite Catalog Number)
    pub norad_id: i32,

    pub start_year_utc: i32,
    pub start_month_utc: i32,
    pub start_day_utc: i32,
    pub start_hour_utc: i32,
    pub start_minute_utc: i32,
    pub start_second_utc: i32,

    pub end_year_utc: i32,
    pub end_month_utc: i32,
    pub end_day_utc: i32,
    pub end_hour_utc: i32,
    pub end_minute_utc: i32,
    pub end_second_utc: i32,

    /// Time between table entries (UTC seconds)
    pub entries_delta_seconds: Option<i32>,

    /// Compatibility with TIVs = 1 (=> integrable, geocentric ephemeris)
    pub tiv_compatibility: bool,

    pub target_class: TargetClass,
    pub reference_frame: ReferenceFrame,
    pub rotational_angle_type: RotationalAngleType,
    pub center_of_mass_correction: CenterOfMassCorrection,
    pub target_location_dynamics: TargetLocationDynamics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum TargetClass {
    #[default]
    NoRetroreflector = 0,
    PassiveRetroreflector = 1,
    Deprecated = 2,
    SynchronousTransponder = 3,
    AsynchronousTransponder = 4,
    Other = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum ReferenceFrame {
    #[default]
    GeocentricTrueBodyFixed = 0,
    GeocentricSpaceFixedTrueOfDate = 1,
    GeocentricSpaceFixedMeanOfDateJ2000 = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum RotationalAngleType {
    #[default]
    NotApplicable = 0,
    /// Lunar Euler angles: φ, θ, and ψ
    LunarEulerAngles = 1,
    /// North pole Right Ascension and Declination, and angle to prime meridian (α_0, δ_0, and W)
    NorthPoleRaDecPrimeMeridian = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum CenterOfMassCorrection {
    /// Prediction is for center of mass of target
    #[default]
    NoneApplied = 0,
    /// Prediction is for retro-reflector array
    Applied = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, TryFromPrimitive)]
#[repr(u8)]
pub enum TargetLocationDynamics {
    #[default]
    Other = 0,
    EarthOrbit = 1,
    LunarOrbit = 2,
    LunarSurface = 3,
    MarsOrbit = 4,
    MarsSurface = 5,
    VenusOrbit = 6,
    MercuryOrbit = 7,
    AsteroidOrbit = 8,
    AsteroidSurface = 9,
    SolarOrbitTransfer = 10,
}
