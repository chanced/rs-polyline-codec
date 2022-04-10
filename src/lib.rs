use std::{error::Error, fmt};

/// LatLng is a tuple composed of latitude and longitude.
#[derive(Debug, Clone, Copy)]
pub struct LatLng(f64, f64);

/// `Point` contains accessors for a coordinate's latitude (`lat`) and longitude
/// (`lng`).
///
/// Note: `Point` is not implemented for `&[f64; 2]` due to geojson's format
/// being `[longitude, latitude]` rather than `[latitude, longitude]` which is
/// expected by this algorithm.
pub trait Point: fmt::Debug {
    fn lat(&self) -> f64;
    fn lng(&self) -> f64;
}

impl<P: Point> PartialEq<P> for LatLng {
    fn eq(&self, other: &P) -> bool {
        self.0 == other.lat() && self.1 == other.lng()
    }
}

impl Point for LatLng {
    fn lat(&self) -> f64 {
        self.0
    }
    fn lng(&self) -> f64 {
        self.1
    }
}

impl Point for (f64, f64) {
    fn lat(&self) -> f64 {
        self.0
    }
    fn lng(&self) -> f64 {
        self.1
    }
}

#[derive(Debug)]
pub struct InvalidEncodingError {
    pub encoded_path: String,
}

impl fmt::Display for InvalidEncodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: invalid encoding: {}", self.encoded_path)
    }
}
impl Error for InvalidEncodingError {}

#[derive(Debug)]
pub struct InvalidLatLngError {
    pub lat: f64,
    pub lng: f64,
}
impl fmt::Display for InvalidLatLngError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error: invalid lat lng: ({}, {})", self.lat, self.lng)
    }
}

/// Decodes an encoded path string into a sequence of LatLngs.
///
/// See https://developers.google.com/maps/documentation/utilities/polylinealgorithm
///
///  #### Example
/// ```rust
/// let encoded = "_p~iF~ps|U_ulLnnqC_mqNvxq`@";
/// assert_eq!(polyline_codec::decode(encoded, 5).unwrap(), vec![
///     (38.5, -120.2),
///     (40.7, -120.95),
///     (43.252, -126.453)
/// ]);
/// ```
pub fn decode(encoded_path: &str, precision: u32) -> Result<Vec<LatLng>, InvalidEncodingError> {
    let factor = 10_i32.pow(precision) as f64;
    // let encoded_path = encoded_path.encode_utf16();
    // TODO: need to see if I can just use the str len
    // let len = encoded_path.clone().count();
    let len = encoded_path.len();
    let mut path = Vec::with_capacity(len / 2);
    let mut index = 0;
    let mut lat = 0.0;
    let mut lng = 0.0;

    while index < len {
        let mut result: i32 = 1;
        let mut shift = 0;
        let mut b: i32;
        loop {
            // b = (encoded_path.clone().nth(index).unwrap() as i32) - 63 - 1;
            b = (encoded_path
                .chars()
                .nth(index)
                .ok_or(InvalidEncodingError {
                    encoded_path: encoded_path.into(),
                })? as i32)
                - 63
                - 1;
            index += 1;
            result += (b << shift) as i32;
            shift += 5;
            if b < 0x1f {
                break;
            }
        }
        lat += (if result & 1 != 0 {
            !(result >> 1)
        } else {
            result >> 1
        }) as f64;
        result = 1;
        shift = 0;
        loop {
            b = (encoded_path.chars().nth(index).unwrap() as i32) - 63 - 1;
            index += 1;
            result += (b << shift) as i32;
            shift += 5;
            if b < 0x1f {
                break;
            }
        }

        lng += (if result & 1 != 0 {
            !(result >> 1)
        } else {
            result >> 1
        }) as f64;
        path.push(LatLng(lat / factor, lng / factor));
    }
    path.shrink_to_fit();
    Ok(path)
}

/// Polyline encodes an array of objects having lat and lng properties.
///
/// See https://developers.google.com/maps/documentation/utilities/polylinealgorithm
///
/// #### Example
/// ```rust
/// let path = vec![
///     (38.5, -120.2),
///     (40.7, -120.95),
///     (43.252, -126.453),
/// ];
/// assert_eq!(polyline_codec::encode(&path, 5).unwrap(), "_p~iF~ps|U_ulLnnqC_mqNvxq`@");
///
/// ```
pub fn encode<P: Point>(path: &[P], precision: u32) -> Result<String, InvalidLatLngError> {
    let factor = 10_f64.powi(precision as i32);
    let transform = |p: &P| LatLng((p.lat() * factor).round(), (p.lng() * factor).round());
    polyline_encode_line(path, transform)
}

///
/// Encodes a generic polyline, optionally performing a transform on each point
/// before encoding it.
#[doc(hidden)]
pub fn polyline_encode_line<P, F>(array: &[P], transform: F) -> Result<String, InvalidLatLngError>
where
    P: Point,
    F: Fn(&P) -> LatLng,
{
    let mut v: Vec<String> = Vec::new();
    let mut start = LatLng(0.0, 0.0);
    let mut end;
    for p in array {
        validate(p)?;
        end = transform(p);
        encode_signed(
            end.lat().round() as i64 - start.lat().round() as i64,
            &mut v,
        ); // lat
        encode_signed(
            end.lng().round() as i64 - start.lng().round() as i64,
            &mut v,
        ); // lng
        start = end;
    }
    Ok(v.join(""))
}

pub(crate) fn validate<P: Point>(p: &P) -> Result<(), InvalidLatLngError> {
    if p.lat() < -90.0 || p.lat() > 90.0 || p.lng() < -180.0 || p.lng() > 180.0 {
        Err(InvalidLatLngError {
            lat: p.lat(),
            lng: p.lng(),
        })
    } else {
        Ok(())
    }
}

/// Encodes the given value in the compact polyline format, appending the
/// encoded value to the given array of strings.
fn encode_signed(value: i64, v: &mut Vec<String>) {
    encode_unsigned(if value < 0 { !(value << 1) } else { value << 1 }, v)
}

fn encode_unsigned(value: i64, v: &mut Vec<String>) {
    let mut value = value;
    while value >= 0x20 {
        let s = vec![((0x20 | (value & 0x1f)) + 63) as u16];
        v.push(String::from_utf16(&s).expect("failed to encode utf16"));
        value >>= 5;
    }

    v.push(String::from_utf16(&[(value + 63) as u16]).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::*;

    #[test]
    fn test_decodes_to_an_empty_array() {
        let v: Vec<LatLng> = vec![];
        assert_eq!(decode("", 5).unwrap(), v);
    }
    #[test]
    fn test_decodes_a_string_into_a_vec_of_lat_lng_pairs() {
        assert_eq!(
            decode("_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5).unwrap(),
            &[(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)]
        );
        dbg!(decode("_p~iF~ps|U_ulLnnqC_mqNvxq`@", 5).unwrap());
    }

    #[test]
    fn test_decodes_with_precision_0() {
        assert_eq!(
            decode("mAnFC@CH", 0).unwrap(),
            &[(39.0, -120.0), (41.0, -121.0), (43.0, -126.0)]
        );
    }
    #[test]
    fn test_encodes_an_empty_vec() {
        let v: Vec<LatLng> = vec![];
        assert_eq!(encode(&v, 5).unwrap(), "");
    }

    #[test]
    fn encodes_a_vec_of_lat_lng_pairs_into_a_string() {
        assert_eq!(
            encode(&[(38.5, -120.2), (40.7, -120.95), (43.252, -126.453)], 5).unwrap(),
            "_p~iF~ps|U_ulLnnqC_mqNvxq`@"
        );
    }

    proptest! {
            #[test]
            fn test_random_roundtrip(path: Vec<(f64, f64)>) {

                let should_error = path.iter().any(|p| p.0 > 90.0 || p.0 < -90.0 || p.1 > 180.0 || p.1 < -180.0);
                if should_error {
                    prop_assert!(encode(&path, 5).is_err());
                } else {
                    let encoded = encode(&path, 5).unwrap();
                    let decoded = decode(&encoded, 5).unwrap();
                    prop_assert_eq!(encoded, encode(&decoded, 5).unwrap());
                }

                // let encoded = encode(&[path], 5);
                // if should_err {
                //     assert!(encoded.is_err());
                // } else {
                //     let encoded = encoded.upwrap();
                // }

                // let decoded = decode(&encoded, 5).unwrap();
                // prop_assert_eq!(path, decoded);
        }
    }
    proptest! {
        #[test]
        fn test_valid_roundtrip(p0 in -90.0..90.0, p1 in -180.0..180.0) {
            // TODO: need to learn proptest better. I need a strategy that
            // generates a Vec<(f64,f64)> within the specified ranges

            let path = vec![(p0, p1)];
            dbg!(&path);
            let encoded = encode(&path, 5).unwrap();
            dbg!(&encoded);
            let decoded = decode(&encoded, 5).unwrap();
            prop_assert_eq!(encoded, encode(&decoded, 5).unwrap());
        }
    }
}
