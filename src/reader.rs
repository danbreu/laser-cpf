use std::io::BufRead;
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

use crate::ephemeris::{DirectionFlag, Ephemeris};
use crate::header::*;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("{line}: invalid record type. expected {expected}")]
    InvalidRecordType { line: usize, expected: &'static str },

    #[error("{line}: invalid record length. expected {expected}, got {got}")]
    InvalidRecordSize {
        line: usize,
        expected: &'static str,
        got: usize,
    },

    #[error("{line}: invalid integer at {field}")]
    InvalidInteger {
        line: usize,
        field: usize,
        #[source]
        source: std::num::ParseIntError,
    },

    #[error("{line}: invalid float at {field}")]
    InvalidFloat {
        line: usize,
        field: usize,
        #[source]
        source: std::num::ParseFloatError,
    },

    #[error("{line}: invalid value at {field}, {reason}")]
    InvalidFieldValue {
        line: usize,
        field: usize,
        reason: &'static str,
    },

    #[error("{line}: empty line")]
    EmptyLine { line: usize },

    #[error("I/O error")]
    IoError(#[from] std::io::Error),

    #[error("unexpected end of file")]
    UnexpectedEof,
}

enum State {
    ExpectH1,
    ExpectH2,
    ExpectH9,
    ReadingEphemeris,
}

struct Record<'a> {
    parts: &'a [&'a str],
    line: usize,
}

impl<'a> Record<'a> {
    fn expect_len(&self, expected: usize, label: &'static str) -> Result<(), ParseError> {
        if self.parts.len() != expected {
            Err(ParseError::InvalidRecordSize {
                line: self.line,
                expected: label,
                got: self.parts.len(),
            })
        } else {
            Ok(())
        }
    }

    fn int<T: FromStr<Err = ParseIntError>>(&self, field: usize) -> Result<T, ParseError> {
        self.parts[field]
            .parse()
            .map_err(|e| ParseError::InvalidInteger {
                line: self.line,
                field,
                source: e,
            })
    }

    fn float(&self, field: usize) -> Result<f64, ParseError> {
        self.parts[field]
            .parse()
            .map_err(|e| ParseError::InvalidFloat {
                line: self.line,
                field,
                source: e,
            })
    }

    fn enumv<T: TryFrom<u8>>(&self, field: usize, reason: &'static str) -> Result<T, ParseError> {
        let raw: u8 = self.int(field)?;
        raw.try_into().map_err(|_| ParseError::InvalidFieldValue {
            line: self.line,
            field,
            reason,
        })
    }

    fn str(&self, field: usize) -> &'a str {
        self.parts[field]
    }
}

pub fn read_cpf_v2(mut reader: impl BufRead) -> Result<(Header, Ephemeris), ParseError> {
    let mut header = Header::default();
    let mut ephemeris = Ephemeris {
        direction_flag: Vec::new(),
        mjd: Vec::new(),
        seconds_of_day: Vec::new(),
        leap_second_flag: Vec::new(),
        position_m: Vec::new(),
    };

    let mut state = State::ExpectH1;
    let mut line_num = 0;
    let mut line_buf = String::new();

    loop {
        line_num += 1;
        line_buf.clear();

        let bytes_read = reader.read_line(&mut line_buf)?;
        if bytes_read == 0 {
            return Err(ParseError::UnexpectedEof);
        }

        let parts: Vec<&str> = line_buf.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ParseError::EmptyLine { line: line_num });
        }

        let record = Record {
            parts: &parts,
            line: line_num,
        };
        let record_type = record.str(0).to_uppercase();

        match state {
            State::ExpectH1 => {
                if record_type != "H1" {
                    return Err(ParseError::InvalidRecordType {
                        line: line_num,
                        expected: "H1",
                    });
                }
                read_h1(&record, &mut header)?;
                state = State::ExpectH2;
            }
            State::ExpectH2 => {
                if record_type != "H2" {
                    return Err(ParseError::InvalidRecordType {
                        line: line_num,
                        expected: "H2",
                    });
                }
                read_h2(&record, &mut header)?;
                state = State::ExpectH9;
            }
            State::ExpectH9 => {
                if record_type != "H9" {
                    return Err(ParseError::InvalidRecordType {
                        line: line_num,
                        expected: "H9",
                    });
                }
                state = State::ReadingEphemeris;
            }
            State::ReadingEphemeris => match record_type.as_str() {
                "10" => {
                    read_10(&record, &mut ephemeris)?;
                }
                "99" => {
                    break;
                }
                _ => {
                    return Err(ParseError::InvalidRecordType {
                        line: line_num,
                        expected: "10 or 99",
                    });
                }
            },
        }
    }

    Ok((header, ephemeris))
}

fn read_10(r: &Record, ephemeris: &mut Ephemeris) -> Result<(), ParseError> {
    r.expect_len(8, "8")?;

    let direction_flag: DirectionFlag = r.enumv(1, "must be 0, 1, or 2")?;
    let mjd: i32 = r.int(2)?;
    let seconds_of_day: f64 = r.float(3)?;
    let leap_second_flag: i32 = r.int(4)?;
    let x: f64 = r.float(5)?;
    let y: f64 = r.float(6)?;
    let z: f64 = r.float(7)?;

    ephemeris.push_position(
        direction_flag,
        mjd,
        seconds_of_day,
        leap_second_flag,
        [x, y, z],
    );

    Ok(())
}

fn read_h1(r: &Record, header: &mut Header) -> Result<(), ParseError> {
    if r.parts.len() != 11 && r.parts.len() != 12 {
        return Err(ParseError::InvalidRecordSize {
            line: r.line,
            expected: "11 or 12",
            got: r.parts.len(),
        });
    }

    // CPFv2 prefix
    if r.str(1) != "CPF" {
        return Err(ParseError::InvalidFieldValue {
            line: r.line,
            field: 1,
            reason: "must be 'CPF'",
        });
    }
    if r.str(2) != "2" {
        return Err(ParseError::InvalidFieldValue {
            line: r.line,
            field: 2,
            reason: "must be '2'",
        });
    }

    header.ephemeris_source = r.str(3).to_string();
    header.production_year_utc = r.int(4)?;
    header.production_month_utc = r.int(5)?;
    header.production_day_utc = r.int(6)?;
    header.production_hour_utc = r.int(7)?;
    header.sequence_number = r.int(8)?;
    header.sub_daily_sequence = r.int(9)?;
    header.target_name = r.str(10).to_string();
    header.notes = r.parts.get(11).map(|s| s.to_string());

    Ok(())
}

fn read_h2(r: &Record, header: &mut Header) -> Result<(), ParseError> {
    r.expect_len(23, "23")?;

    header.ilrs_satellite_id = r.int(1)?;
    header.sic = r.int(2)?;
    header.norad_id = r.int(3)?;
    header.start_year_utc = r.int(4)?;
    header.start_month_utc = r.int(5)?;
    header.start_day_utc = r.int(6)?;
    header.start_hour_utc = r.int(7)?;
    header.start_minute_utc = r.int(8)?;
    header.start_second_utc = r.int(9)?;
    header.end_year_utc = r.int(10)?;
    header.end_month_utc = r.int(11)?;
    header.end_day_utc = r.int(12)?;
    header.end_hour_utc = r.int(13)?;
    header.end_minute_utc = r.int(14)?;
    header.end_second_utc = r.int(15)?;

    let time_between: i32 = r.int(16)?;
    header.entries_delta_seconds = if time_between == 0 {
        None
    } else {
        Some(time_between)
    };

    let tiv_compat: u8 = r.int(17)?;
    header.tiv_compatibility = tiv_compat != 0;

    header.target_class = r.enumv(18, "must be in [0, 5]")?;
    header.reference_frame = r.enumv(19, "must be in {0, 1, 2}")?;
    header.rotational_angle_type = r.enumv(20, "must be in {0, 1, 2}")?;
    header.center_of_mass_correction = r.enumv(21, "must be in {0, 1}")?;
    header.target_location_dynamics = r.enumv(22, "must be in [0, 10]")?;

    Ok(())
}
