use bitflags::bitflags;
use nom::{
    bytes::complete::take,
    combinator::{cond, map},
    multi::count,
    number::complete::{
        le_f32, le_f64, le_i16, le_i32, le_i64, le_i8, le_u16, le_u32, le_u64, le_u8,
    },
    Parser,
};
use strum::FromRepr;

use crate::binary::{
    errors::{ParseError, ParseResult},
    scalars::{
        byte, dword, fixed, parse_color, parse_dword_as_usize, parse_point, parse_rect, parse_size,
        parse_string, parse_uuid, word, Color, Double, Dword, Fixed, Float, Point, Rect, Size,
        Uuid,
    },
};

bitflags! {
    pub struct UserDataFlags: Dword {
        const HAS_TEXT = 0x1;
        const HAS_COLOR = 0x2;
        const HAS_PROPERTIES = 0x4;
    }
}

#[derive(Debug)]
pub struct UserDataChunk<'a> {
    pub text: Option<&'a str>,
    pub color: Option<Color>,
    pub properties_maps: Option<ParseResult<'a, Vec<PropertiesMap<'a>>>>,
}

#[derive(Debug)]
pub struct PropertiesMap<'a> {
    pub properties: Vec<Property<'a>>,
    pub extension_entry_id: Dword,
}

#[derive(Debug)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: Value<'a>,
}

#[derive(FromRepr, Debug, Copy, Clone)]
#[repr(u16)]
pub enum PropertyType {
    Bool = 0x0001,
    Int8 = 0x0002,
    Uint8 = 0x0003,
    Int16 = 0x0004,
    Uint16 = 0x0005,
    Int32 = 0x0006,
    Uint32 = 0x0007,
    Int64 = 0x0008,
    Uint64 = 0x0009,
    Fixed = 0x000A,
    Float = 0x000B,
    Double = 0x000C,
    String = 0x000D,
    Point = 0x000E,
    Size = 0x000F,
    Rect = 0x0010,
    Vector = 0x0011,
    PropertiesMap = 0x0012,
    Uuid = 0x0013,
}

#[derive(Debug)]
pub enum Value<'a> {
    Bool(bool),
    Int8(i8),
    Uint8(u8),
    Int16(i16),
    Uint16(u16),
    Int32(i32),
    Uint32(u32),
    Int64(i64),
    Uint64(u64),
    Fixed(Fixed),
    Float(f32),
    Double(f64),
    String(&'a str),
    Point(Point),
    Size(Size),
    Rect(Rect),
    Vector(Vector<'a>),
    MixedVector(Vec<Value<'a>>),
    PropertiesMap(PropertiesMap<'a>),
    Uuid(Uuid),
}

#[derive(Debug)]
pub enum Vector<'a> {
    Mixed(Vec<Value<'a>>),
    Bool(Vec<bool>),
    Int8(Vec<i8>),
    Uint8(Vec<u8>),
    Int16(Vec<i16>),
    Uint16(Vec<u16>),
    Int32(Vec<i32>),
    Uint32(Vec<u32>),
    Int64(Vec<i64>),
    Uint64(Vec<u64>),
    Fixed(Vec<Fixed>),
    Float(Vec<Float>),
    Double(Vec<Double>),
    String(Vec<&'a str>),
    Point(Vec<Point>),
    Size(Vec<Size>),
    Rect(Vec<Rect>),
    Vector(Vec<Vector<'a>>),
    PropertiesMap(Vec<PropertiesMap<'a>>),
    Uuid(Vec<Uuid>),
}

pub fn parse_user_data_chunk(input: &[u8]) -> ParseResult<'_, UserDataChunk<'_>> {
    let (input, flags) = dword(input)?;
    let flags = UserDataFlags::from_bits_truncate(flags);
    let (input, text) = cond(flags.contains(UserDataFlags::HAS_TEXT), parse_string).parse(input)?;
    let (input, color) =
        cond(flags.contains(UserDataFlags::HAS_COLOR), parse_color).parse(input)?;
    let (input, properties_maps) = cond(
        flags.contains(UserDataFlags::HAS_PROPERTIES),
        parse_properties_maps,
    )
    .parse(input)?;
    Ok((
        input,
        (UserDataChunk {
            text,
            color,
            properties_maps,
        }),
    ))
}

pub fn parse_properties_maps(
    input: &[u8],
) -> ParseResult<'_, ParseResult<'_, Vec<PropertiesMap<'_>>>> {
    let (input, size_maps) = parse_dword_as_usize(input)?;
    let (input, num_maps) = parse_dword_as_usize(input)?;
    // FIXME handle underflows
    let (input, input_maps) = take(size_maps - 4)(input)?;
    Ok((
        input,
        count(parse_properties_map, num_maps).parse(input_maps),
    ))
}

pub fn parse_properties_map(input: &[u8]) -> ParseResult<'_, PropertiesMap<'_>> {
    let (input, extension_entry_id) = dword(input)?;
    let (input, num_props) = parse_dword_as_usize(input)?;
    let (input, properties) = count(parse_property, num_props).parse(input)?;
    Ok((
        input,
        PropertiesMap {
            extension_entry_id,
            properties,
        },
    ))
}

pub fn parse_property(input: &[u8]) -> ParseResult<'_, Property<'_>> {
    let (input, name) = parse_string(input)?;
    let (input, value) = parse_value(input)?;
    Ok((input, Property { name, value }))
}

pub fn parse_value(input: &[u8]) -> ParseResult<'_, Value<'_>> {
    let (input, prop_type) = word(input)?;
    let prop_type = PropertyType::from_repr(prop_type).ok_or(nom::Err::Failure(
        ParseError::InvalidPropertyType(prop_type),
    ))?;
    Ok(match prop_type {
        PropertyType::Bool => map(byte, |b| Value::Bool(b != 0)).parse(input)?,
        PropertyType::Int8 => map(le_i8, Value::Int8).parse(input)?,
        PropertyType::Uint8 => map(le_u8, Value::Uint8).parse(input)?,
        PropertyType::Int16 => map(le_i16, Value::Int16).parse(input)?,
        PropertyType::Uint16 => map(le_u16, Value::Uint16).parse(input)?,
        PropertyType::Int32 => map(le_i32, Value::Int32).parse(input)?,
        PropertyType::Uint32 => map(le_u32, Value::Uint32).parse(input)?,
        PropertyType::Int64 => map(le_i64, Value::Int64).parse(input)?,
        PropertyType::Uint64 => map(le_u64, Value::Uint64).parse(input)?,
        PropertyType::Fixed => map(fixed, Value::Fixed).parse(input)?,
        PropertyType::Float => map(le_f32, Value::Float).parse(input)?,
        PropertyType::Double => map(le_f64, Value::Double).parse(input)?,
        PropertyType::String => map(parse_string, Value::String).parse(input)?,
        PropertyType::Point => map(parse_point, Value::Point).parse(input)?,
        PropertyType::Size => map(parse_size, Value::Size).parse(input)?,
        PropertyType::Rect => map(parse_rect, Value::Rect).parse(input)?,
        PropertyType::Vector => map(parse_vector, Value::Vector).parse(input)?,
        PropertyType::PropertiesMap => {
            map(parse_properties_map, Value::PropertiesMap).parse(input)?
        }
        PropertyType::Uuid => map(parse_uuid, Value::Uuid).parse(input)?,
    })
}

pub fn parse_vector(input: &[u8]) -> ParseResult<'_, Vector<'_>> {
    let (input, len_elements) = parse_dword_as_usize(input)?;
    let (input, prop_type) = word(input)?;
    if prop_type == 0 {
        return map(count(parse_value, len_elements), Vector::Mixed).parse(input);
    }
    let prop_type = PropertyType::from_repr(prop_type).ok_or(nom::Err::Failure(
        ParseError::InvalidPropertyType(prop_type),
    ))?;
    let (input, vec) = match prop_type {
        PropertyType::Bool => {
            map(count(map(byte, |b| b != 0), len_elements), Vector::Bool).parse(input)?
        }
        PropertyType::Int8 => map(count(le_i8, len_elements), Vector::Int8).parse(input)?,
        PropertyType::Uint8 => map(count(le_u8, len_elements), Vector::Uint8).parse(input)?,
        PropertyType::Int16 => map(count(le_i16, len_elements), Vector::Int16).parse(input)?,
        PropertyType::Uint16 => map(count(le_u16, len_elements), Vector::Uint16).parse(input)?,
        PropertyType::Int32 => map(count(le_i32, len_elements), Vector::Int32).parse(input)?,
        PropertyType::Uint32 => map(count(le_u32, len_elements), Vector::Uint32).parse(input)?,
        PropertyType::Int64 => map(count(le_i64, len_elements), Vector::Int64).parse(input)?,
        PropertyType::Uint64 => map(count(le_u64, len_elements), Vector::Uint64).parse(input)?,
        PropertyType::Fixed => map(count(fixed, len_elements), Vector::Fixed).parse(input)?,
        PropertyType::Float => map(count(le_f32, len_elements), Vector::Float).parse(input)?,
        PropertyType::Double => map(count(le_f64, len_elements), Vector::Double).parse(input)?,
        PropertyType::String => {
            map(count(parse_string, len_elements), Vector::String).parse(input)?
        }
        PropertyType::Point => map(count(parse_point, len_elements), Vector::Point).parse(input)?,
        PropertyType::Size => map(count(parse_size, len_elements), Vector::Size).parse(input)?,
        PropertyType::Rect => map(count(parse_rect, len_elements), Vector::Rect).parse(input)?,
        PropertyType::Vector => {
            map(count(parse_vector, len_elements), Vector::Vector).parse(input)?
        }
        PropertyType::PropertiesMap => map(
            count(parse_properties_map, len_elements),
            Vector::PropertiesMap,
        )
        .parse(input)?,
        PropertyType::Uuid => map(count(parse_uuid, len_elements), Vector::Uuid).parse(input)?,
    };
    Ok((input, vec))
}
