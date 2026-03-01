pub mod ephemeris;
pub mod header;
pub mod reader;

pub use ephemeris::Ephemeris;
pub use header::Header;
pub use reader::{ParseOptions, read_cpf_v2, read_cpf_v2_with_options};

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
        let (header, _ephemeris) = read_cpf_v2(reader).unwrap();

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
        let result = read_cpf_v2(reader);
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_parse_gps36_ephemeris() {
        use ephemeris::DirectionFlag;

        let data = include_bytes!("../test_data/gps36_cpf_051129_33401.codv2");
        let reader = Cursor::new(data);
        let (_header, ephemeris) = read_cpf_v2(reader).unwrap();

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
        let (_header, ephemeris) = read_cpf_v2(reader).unwrap();
        assert_eq!(ephemeris.mjd.len(), 0);
    }

    #[test]
    fn test_missing_h1() {
        let data =
            b"H2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
        assert!(
            matches!(result, Err(ParseError::InvalidRecordType { expected, .. }) if expected == "H1")
        );
    }

    #[test]
    fn test_missing_h2() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
        assert!(
            matches!(result, Err(ParseError::InvalidRecordType { expected, .. }) if expected == "H2")
        );
    }

    #[test]
    fn test_missing_h9() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_missing_99() {
        let data = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
        assert!(matches!(result, Err(..)));
    }

    #[test]
    fn test_invalid_cpf_format() {
        let data = b"H1 AUA 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
        assert!(matches!(result, Err(ParseError::InvalidFieldValue { field, .. }) if field == 1));
    }

    #[test]
    fn test_invalid_cpf_version() {
        let data = b"H1 CPF 1 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";
        let reader = Cursor::new(data);
        let result = read_cpf_v2(reader);
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
        let (_header, ephemeris) = read_cpf_v2_with_options(reader, &options).unwrap();
        assert!(ephemeris.direction_flag.is_none());
        assert!(ephemeris.leap_second_flag.is_none());
        assert_eq!(ephemeris.mjd.len(), 480);
    }

    #[test]
    fn test_parse_options_violations() {
        let header = b"H1 CPF 2 COD 2005 11 30 04 334 1 gps36\nH2 9401601 3636 23027 2005 11 29 23 59 47 2005 12 04 23 44 47 900 1 1 0 0 0 1\nH9\n";

        // direction_flag=1 with assert_common_epoch_only
        let mut data = Vec::from(&header[..]);
        data.extend_from_slice(
            b"10 1 53703 86387.000000 0 -20733881.936 1385083.581 16779721.134\n99\n",
        );
        let result = read_cpf_v2_with_options(
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
        let result = read_cpf_v2_with_options(
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
