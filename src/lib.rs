//! Parser for [CPFv2](https://ilrs.gsfc.nasa.gov/data_and_products/formats/cpf.html) (Consolidated Prediction Format) files.
//!
//! # Example
//!
//! ```no_run
//! use std::io::BufReader;
//! use std::fs::File;
//! use laser_cpf::{ParseOptions, read_cpf_v2};
//!
//! let file = File::open("terrasarx_cpf_250101_00101.gfz")?;
//! let reader = BufReader::new(file);
//! let (header, ephemeris) = read_cpf_v2(reader, &ParseOptions::default())?;
//!
//! println!("Target: {}", header.target_name);
//! println!("Records: {}", ephemeris.mjd.len());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod ephemeris;
pub mod header;
pub mod reader;

pub use ephemeris::{DirectionFlag, Ephemeris};
pub use header::{
    CenterOfMassCorrection, Header, ReferenceFrame, RotationalAngleType, TargetClass,
    TargetLocationDynamics,
};
pub use reader::{ParseError, ParseOptions, read_cpf_v2};

#[cfg(test)]
mod tests {
    use super::*;
    use header::{CenterOfMassCorrection, ReferenceFrame, TargetClass, TargetLocationDynamics};
    use reader::ParseError;
    use std::io::Cursor;

    #[test]
    fn test_parse_gps36_header() {
        let data = include_bytes!("../test_data/gps36_cpf_051129_33401.codv2");
        let reader = Cursor::new(data);
        let (header, _ephemeris) = read_cpf_v2(reader, &ParseOptions::default()).unwrap();

        // H1 fields
        assert_eq!(header.ephemeris_source, "COD");
        assert_eq!(header.production_year_utc, 2005);
        assert_eq!(header.production_month_utc, 11);
        assert_eq!(header.production_day_utc, 30);
        assert_eq!(header.production_hour_utc, 4);
        assert_eq!(header.sequence_number, 334);
        assert_eq!(header.sub_daily_sequence, 1);
        assert_eq!(header.target_name, "gps36");
        assert!(header.notes.is_none());

        // H2 fields
        assert_eq!(header.ilrs_satellite_id, 9401601);
        assert_eq!(header.sic, 3636);
        assert_eq!(header.norad_id, 23027);
        assert_eq!(header.start_year_utc, 2005);
        assert_eq!(header.start_month_utc, 11);
        assert_eq!(header.start_day_utc, 29);
        assert_eq!(header.start_hour_utc, 23);
        assert_eq!(header.start_minute_utc, 59);
        assert_eq!(header.start_second_utc, 47);
        assert_eq!(header.end_year_utc, 2005);
        assert_eq!(header.end_month_utc, 12);
        assert_eq!(header.end_day_utc, 4);
        assert_eq!(header.end_hour_utc, 23);
        assert_eq!(header.end_minute_utc, 44);
        assert_eq!(header.end_second_utc, 47);
        assert_eq!(header.entries_delta_seconds, Some(900));
        assert!(header.tiv_compatibility);
        assert_eq!(header.target_class, TargetClass::PassiveRetroreflector);
        assert_eq!(
            header.reference_frame,
            ReferenceFrame::GeocentricTrueBodyFixed
        );
        assert_eq!(
            header.center_of_mass_correction,
            CenterOfMassCorrection::NoneApplied
        );
        assert_eq!(
            header.target_location_dynamics,
            TargetLocationDynamics::EarthOrbit
        );
    }

    #[test]
    fn test_notes() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_parse_gps36_ephemeris() {
        use ephemeris::DirectionFlag;

        let data = include_bytes!("../test_data/gps36_cpf_051129_33401.codv2");
        let reader = Cursor::new(data);
        let (_header, ephemeris) = read_cpf_v2(reader, &ParseOptions::default()).unwrap();

        // 480 position records in file
        assert_eq!(ephemeris.mjd.len(), 480);

        // First record: 10 0 53703  86387.000000  0  -20733881.936   1385083.581  16779721.134
        assert_eq!(
            ephemeris.direction_flag.as_ref().unwrap()[0],
            DirectionFlag::CommonEpoch
        );
        assert_eq!(ephemeris.mjd[0], 53703);
        assert!((ephemeris.seconds_of_day[0] - 86387.0).abs() < 1e-6);
        assert!(ephemeris.leap_second_flag.as_ref().unwrap()[0] == 0);
        assert!((ephemeris.position_m[0][0] - (-20733881.936)).abs() < 1e-6);
        assert!((ephemeris.position_m[0][1] - 1385083.581).abs() < 1e-6);
        assert!((ephemeris.position_m[0][2] - 16779721.134).abs() < 1e-6);

        // Last record: 10 0 53708  85487.000000  0  -20242610.289    844653.053  17406764.424
        let last = ephemeris.mjd.len() - 1;
        assert_eq!(ephemeris.mjd[last], 53708);
        assert!((ephemeris.seconds_of_day[last] - 85487.0).abs() < 1e-6);
        assert!((ephemeris.position_m[last][0] - (-20242610.289)).abs() < 1e-6);
        assert!((ephemeris.position_m[last][1] - 844653.053).abs() < 1e-6);
        assert!((ephemeris.position_m[last][2] - 17406764.424).abs() < 1e-6);
    }

    #[test]
    fn test_empty_ephemeris() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n99";
        let reader = Cursor::new(data);
        let (_header, ephemeris) = read_cpf_v2(reader, &ParseOptions::default()).unwrap();
        assert_eq!(ephemeris.mjd.len(), 0);
    }

    #[test]
    fn test_missing_h1() {
        let data =
            b"H2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(
            matches!(result, Err(ParseError::InvalidRecordType { expected, .. }) if expected == "H1")
        );
    }

    #[test]
    fn test_missing_h2() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(
            matches!(result, Err(ParseError::InvalidRecordType { expected, .. }) if expected == "H2")
        );
    }

    #[test]
    fn test_missing_h9() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_missing_99() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_invalid_cpf_format() {
        let data = b"H1 AUA 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(result, Err(ParseError::InvalidFieldValue { field, .. }) if field == 1));
    }

    #[test]
    fn test_invalid_cpf_version() {
        let data = b"H1 CPF 1 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(result, Err(ParseError::InvalidFieldValue { field, .. }) if field == 2));
    }

    #[test]
    fn test_parse_options_assertions() {
        let data = include_bytes!("../test_data/gps36_cpf_051129_33401.codv2");
        let reader = Cursor::new(data);
        let options = ParseOptions {
            assert_common_epoch_only: true,
            assert_no_leap_second: true,
        };
        let (_header, ephemeris) = read_cpf_v2(reader, &options).unwrap();
        assert!(ephemeris.direction_flag.is_none());
        assert!(ephemeris.leap_second_flag.is_none());
        assert_eq!(ephemeris.mjd.len(), 480);
    }

    #[test]
    fn test_no_velocity_records() {
        let data = include_bytes!("../test_data/gps36_cpf_051129_33401.codv2");
        let reader = Cursor::new(data);
        let (_header, ephemeris) = read_cpf_v2(reader, &ParseOptions::default()).unwrap();
        assert!(ephemeris.velocity_m_per_s.is_none());
    }

    #[test]
    fn test_parse_velocity_records() {
        // Transponder-style grouping: 10-1, 10-2, 20-1, 20-2
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\n\
            H2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n\
            H9\n\
            10 1 53703 86387.000000 0 -20733881.936 1385083.581 16779721.134\n\
            10 2 53703 86387.000000 0 -20600000.000 1400000.000 16800000.000\n\
            20 1 1234.567 -891.012 345.678\n\
            20 2 -100.200 300.400 -500.600\n\
            99\n";
        let reader = Cursor::new(data);
        let (_header, ephemeris) = read_cpf_v2(reader, &ParseOptions::default()).unwrap();

        assert_eq!(ephemeris.position_m.len(), 2);
        let vel = ephemeris.velocity_m_per_s.as_ref().unwrap();
        assert_eq!(vel.len(), 2);

        assert!((vel[0][0] - 1234.567).abs() < 1e-6);
        assert!((vel[0][1] - (-891.012)).abs() < 1e-6);
        assert!((vel[0][2] - 345.678).abs() < 1e-6);

        assert!((vel[1][0] - (-100.200)).abs() < 1e-6);
        assert!((vel[1][1] - 300.400).abs() < 1e-6);
        assert!((vel[1][2] - (-500.600)).abs() < 1e-6);
    }

    #[test]
    fn test_velocity_direction_flag_mismatch() {
        // 10 with direction_flag=1, then 20 with direction_flag=2 -> should error
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\n\
            H2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n\
            H9\n\
            10 1 53703 86387.000000 0 -20733881.936 1385083.581 16779721.134\n\
            20 2 1234.567 -891.012 345.678\n\
            99\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(
            result,
            Err(ParseError::InvalidFieldValue { field: 1, reason, .. })
            if reason.contains("direction_flag")
        ));
    }

    #[test]
    fn test_velocity_without_position() {
        // 20 record before any 10 -> should error
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\n\
            H2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n\
            H9\n\
            20 0 1234.567 -891.012 345.678\n\
            99\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader, &ParseOptions::default());
        assert!(matches!(
            result,
            Err(ParseError::InvalidFieldValue { field: 0, reason, .. })
            if reason.contains("without corresponding position")
        ));
    }

    #[test]
    fn test_parse_options_violations() {
        let header = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";

        // direction_flag=1 with assert_common_epoch_only
        let mut data = Vec::from(&header[..]);
        data.extend_from_slice(
            b"10 1 53703 86387.000000 0 -20733881.936 1385083.581 16779721.134\n99\n",
        );
        let result = read_cpf_v2(
            Cursor::new(data),
            &ParseOptions {
                assert_common_epoch_only: true,
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(ParseError::AssertionViolation { message, .. })
            if message.contains("direction_flag")
        ));

        // leap_second=1 with assert_no_leap_second
        let mut data = Vec::from(&header[..]);
        data.extend_from_slice(
            b"10 0 53703 86387.000000 1 -20733881.936 1385083.581 16779721.134\n99\n",
        );
        let result = read_cpf_v2(
            Cursor::new(data),
            &ParseOptions {
                assert_no_leap_second: true,
                ..Default::default()
            },
        );
        assert!(matches!(
            result,
            Err(ParseError::AssertionViolation { message, .. })
            if message.contains("leap_second")
        ));
    }
}
