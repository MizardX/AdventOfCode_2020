use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("Syntax error")]
    SyntaxError,
    #[error("Invalid field name")]
    InvalidField,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
    #[error("Height is neither 'cm' not 'in'")]
    InvalidHeightUnit,
    #[error("Value out of range")]
    ValueOutOfRange,
    #[error("Invalid hair color (hex)")]
    InvalidHairColor,
    #[error("Invalid eye color (named)")]
    InvalidEyeColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    /// Birth Year
    Byr,
    /// Issue Year
    Iyr,
    /// Expiration Year
    Eyr,
    /// Height
    Hgt,
    /// Hair Color
    Hcl,
    /// Eye Color
    Ecl,
    /// Passport ID
    Pid,
    /// Country ID
    Cid,
}

impl Field {
    const fn all() -> [Self; 8] {
        [
            Self::Byr,
            Self::Iyr,
            Self::Eyr,
            Self::Hgt,
            Self::Hcl,
            Self::Ecl,
            Self::Pid,
            Self::Cid,
        ]
    }

    const fn is_optional(self) -> bool {
        matches!(self, Self::Cid)
    }
}

impl FromStr for Field {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "byr" => Self::Byr,
            "iyr" => Self::Iyr,
            "eyr" => Self::Eyr,
            "hgt" => Self::Hgt,
            "hcl" => Self::Hcl,
            "ecl" => Self::Ecl,
            "pid" => Self::Pid,
            "cid" => Self::Cid,
            _ => return Err(ParseError::InvalidField),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Ranged<const LOW: u16, const HIGH: u16>(u16);

impl<const LOW: u16, const HIGH: u16> Ranged<LOW, HIGH> {
    fn new(value: u16) -> Result<Self, ParseError> {
        (LOW..=HIGH)
            .contains(&value)
            .then_some(Self(value))
            .ok_or(ParseError::ValueOutOfRange)
    }
}

impl<const LOW: u16, const HIGH: u16> FromStr for Ranged<LOW, HIGH> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.parse()?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Height {
    Cm(Ranged<150, 193>),
    In(Ranged<59, 76>),
}

impl FromStr for Height {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= 2 {
            return Err(ParseError::InvalidHeightUnit);
        }
        let (val, unit) = s.split_at(s.len() - 2);
        Ok(match unit {
            "cm" => Self::Cm(val.parse()?),
            "in" => Self::In(val.parse()?),
            _ => return Err(ParseError::InvalidHeightUnit),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct HairColor(u32);

impl HairColor {
    fn new(value: u32) -> Result<Self, ParseError> {
        (value <= 0xFF_FF_FF)
            .then_some(Self(value))
            .ok_or(ParseError::InvalidHairColor)
    }
}

impl FromStr for HairColor {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 7
            && s.starts_with('#')
            && let Ok(value) = u32::from_str_radix(&s[1..], 16)
        {
            Self::new(value)
        } else {
            Err(ParseError::InvalidHairColor)
        }
    }
}

impl Debug for HairColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HairColor({:#06x})", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EyeColor {
    Amb,
    Blu,
    Brn,
    Gry,
    Grn,
    Hzl,
    Oth,
}

impl FromStr for EyeColor {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "amb" => Self::Amb,
            "blu" => Self::Blu,
            "brn" => Self::Brn,
            "gry" => Self::Gry,
            "grn" => Self::Grn,
            "hzl" => Self::Hzl,
            "oth" => Self::Oth,
            _ => return Err(ParseError::InvalidEyeColor),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PassportId([u8; 9]);

impl FromStr for PassportId {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(digits) = <[u8; 9]>::try_from(s.as_bytes())
            && digits.iter().all(u8::is_ascii_digit)
        {
            Ok(Self(digits))
        } else {
            Err(ParseError::ValueOutOfRange)
        }
    }
}

impl Debug for PassportId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PassportId(*b{:?})", str::from_utf8(&self.0).unwrap())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Validated<T> {
    #[default]
    None,
    Invalid,
    Valid(T),
}

impl<T> FromStr for Validated<T>
where
    T: FromStr,
{
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse().map_or(Self::Invalid, Self::Valid))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Passport {
    byr: Validated<Ranged<1920, 2002>>,
    iyr: Validated<Ranged<2010, 2020>>,
    eyr: Validated<Ranged<2020, 2030>>,
    hgt: Validated<Height>,
    hcl: Validated<HairColor>,
    ecl: Validated<EyeColor>,
    pid: Validated<PassportId>,
}

impl Passport {
    fn set_parsed(&mut self, field: Field, value: &str) -> Result<(), ParseError> {
        match field {
            Field::Byr => self.byr = value.parse()?,
            Field::Iyr => self.iyr = value.parse()?,
            Field::Eyr => self.eyr = value.parse()?,
            Field::Hgt => self.hgt = value.parse()?,
            Field::Hcl => self.hcl = value.parse()?,
            Field::Ecl => self.ecl = value.parse()?,
            Field::Pid => self.pid = value.parse()?,
            Field::Cid => (),
        }
        Ok(())
    }

    const fn has_field(&self, field: Field) -> bool {
        match field {
            Field::Byr => !matches!(self.byr, Validated::None),
            Field::Iyr => !matches!(self.iyr, Validated::None),
            Field::Eyr => !matches!(self.eyr, Validated::None),
            Field::Hgt => !matches!(self.hgt, Validated::None),
            Field::Hcl => !matches!(self.hcl, Validated::None),
            Field::Ecl => !matches!(self.ecl, Validated::None),
            Field::Pid => !matches!(self.pid, Validated::None),
            Field::Cid => false,
        }
    }

    const fn is_field_valid(&self, field: Field) -> bool {
        match field {
            Field::Byr => matches!(self.byr, Validated::Valid(..)),
            Field::Iyr => matches!(self.iyr, Validated::Valid(..)),
            Field::Eyr => matches!(self.eyr, Validated::Valid(..)),
            Field::Hgt => matches!(self.hgt, Validated::Valid(..)),
            Field::Hcl => matches!(self.hcl, Validated::Valid(..)),
            Field::Ecl => matches!(self.ecl, Validated::Valid(..)),
            Field::Pid => matches!(self.pid, Validated::Valid(..)),
            Field::Cid => true,
        }
    }

    fn has_all_fields(&self) -> bool {
        Field::all()
            .into_iter()
            .all(|f| f.is_optional() || self.has_field(f))
    }

    fn all_fields_valid(&self) -> bool {
        Field::all()
            .into_iter()
            .all(|f| f.is_optional() || self.is_field_valid(f))
    }
}

impl FromStr for Passport {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut passport = Self::default();
        for pair in s.split_ascii_whitespace() {
            let (field_str, value) = pair.split_once(':').ok_or(ParseError::SyntaxError)?;
            if let Ok(field) = field_str.parse() {
                passport.set_parsed(field, value)?;
            }
        }
        Ok(passport)
    }
}

#[aoc_generator(day4)]
fn parse(input: &str) -> Result<Vec<Passport>, ParseError> {
    input.split("\n\n").map(str::parse).collect()
}

#[aoc(day4, part1)]
fn part_1(passports: &[Passport]) -> usize {
    passports.iter().filter(|p| p.has_all_fields()).count()
}

#[aoc(day4, part2)]
fn part_2(passports: &[Passport]) -> usize {
    passports.iter().filter(|p| p.all_fields_valid()).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    const EXAMPLE1: &str = "\
        ecl:gry pid:860033327 eyr:2020 hcl:#fffffd\n\
        byr:1937 iyr:2017 cid:147 hgt:183cm\n\
        \n\
        iyr:2013 ecl:amb cid:350 eyr:2023 pid:028048884\n\
        hcl:#cfa07d byr:1929\n\
        \n\
        hcl:#ae17e1 iyr:2013\n\
        eyr:2024\n\
        ecl:brn pid:760753108 byr:1931\n\
        hgt:179cm\n\
        \n\
        hcl:#cfa07d eyr:2025 pid:166559648\n\
        iyr:2011 ecl:brn hgt:59in\
    ";

    const EXAMPLE2: &str = "\
        eyr:1972 cid:100\n\
        hcl:#18171d ecl:amb hgt:170 pid:186cm iyr:2018 byr:1926\n\
        \n\
        iyr:2019\n\
        hcl:#602927 eyr:1967 hgt:170cm\n\
        ecl:grn pid:012533040 byr:1946\n\
        \n\
        hcl:dab227 iyr:2012\n\
        ecl:brn hgt:182cm pid:021572410 eyr:2020 byr:1992 cid:277\n\
        \n\
        hgt:59cm ecl:zzz\n\
        eyr:2038 hcl:74454a iyr:2023\n\
        pid:3556412378 byr:2007\
    ";

    const EXAMPLE3: &str = "\
        pid:087499704 hgt:74in ecl:grn iyr:2012 eyr:2030 byr:1980\n\
        hcl:#623a2f\n\
        \n\
        eyr:2029 ecl:blu cid:129 byr:1989\n\
        iyr:2014 pid:896056539 hcl:#a97842 hgt:165cm\n\
        \n\
        hcl:#888785\n\
        hgt:164cm byr:2001 iyr:2015 cid:88\n\
        pid:545766238 ecl:hzl\n\
        eyr:2022\n\
        \n\
        iyr:2010 hgt:158cm hcl:#b6652a ecl:blu byr:1944 eyr:2021 pid:093154719\
    ";

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_parse() {
        let result = parse(EXAMPLE1).unwrap();
        assert_eq!(
            result,
            [
                Passport {
                    byr: Validated::Valid(Ranged(1937)),
                    iyr: Validated::Valid(Ranged(2017)),
                    eyr: Validated::Valid(Ranged(2020)),
                    hgt: Validated::Valid(Height::Cm(Ranged(183))),
                    hcl: Validated::Valid(HairColor(0xfffffd)),
                    ecl: Validated::Valid(EyeColor::Gry),
                    pid: Validated::Valid(PassportId(*b"860033327")),
                },
                Passport {
                    byr: Validated::Valid(Ranged(1929)),
                    iyr: Validated::Valid(Ranged(2013)),
                    eyr: Validated::Valid(Ranged(2023)),
                    hgt: Validated::None,
                    hcl: Validated::Valid(HairColor(0xcfa07d)),
                    ecl: Validated::Valid(EyeColor::Amb),
                    pid: Validated::Valid(PassportId(*b"028048884")),
                },
                Passport {
                    byr: Validated::Valid(Ranged(1931)),
                    iyr: Validated::Valid(Ranged(2013)),
                    eyr: Validated::Valid(Ranged(2024)),
                    hgt: Validated::Valid(Height::Cm(Ranged(179))),
                    hcl: Validated::Valid(HairColor(0xae17e1)),
                    ecl: Validated::Valid(EyeColor::Brn),
                    pid: Validated::Valid(PassportId(*b"760753108")),
                },
                Passport {
                    byr: Validated::None,
                    iyr: Validated::Valid(Ranged(2011)),
                    eyr: Validated::Valid(Ranged(2025)),
                    hgt: Validated::Valid(Height::In(Ranged(59))),
                    hcl: Validated::Valid(HairColor(0xcfa07d)),
                    ecl: Validated::Valid(EyeColor::Brn),
                    pid: Validated::Valid(PassportId(*b"166559648")),
                }
            ]
        );
    }

    #[test]
    fn test_part_1() {
        let passports = parse(EXAMPLE1).unwrap();
        let result = part_1(&passports);
        assert_eq!(result, 2);
    }

    #[test_case(EXAMPLE2 => 0)]
    #[test_case(EXAMPLE3 => 4)]
    fn test_part_2(input: &str) -> usize {
        let passports = parse(input).unwrap();
        part_2(&passports)
    }
}
