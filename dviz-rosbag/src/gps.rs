//! GPS/NMEA Message Processing
//!
//! Parses GPS-related messages from ROS bags:
//! - nmea_msgs/Sentence (NMEA sentences)
//! - sensor_msgs/TimeReference (GPS time)
//! - sensor_msgs/Temperature (temperature sensor)

use crate::{Result, RosBagError};

/// Parsed NMEA sentence
#[derive(Debug, Clone, Default)]
pub struct NmeaSentence {
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Raw NMEA sentence string
    pub sentence: String,
    /// Parsed sentence type (e.g., "GPGGA", "GPRMC")
    pub sentence_type: String,
}

/// Parsed GPS position from NMEA
#[derive(Debug, Clone, Copy, Default)]
pub struct GpsPosition {
    /// Latitude in degrees
    pub latitude: f64,
    /// Longitude in degrees
    pub longitude: f64,
    /// Altitude in meters
    pub altitude: f64,
    /// Fix quality (0=invalid, 1=GPS, 2=DGPS, etc.)
    pub fix_quality: u8,
    /// Number of satellites
    pub num_satellites: u8,
    /// Horizontal dilution of precision
    pub hdop: f32,
}

/// Parsed time reference
#[derive(Debug, Clone, Default)]
pub struct TimeReference {
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Time source (e.g., "gps")
    pub source: String,
    /// Referenced time in seconds
    pub time_ref: f64,
}

/// Parsed temperature reading
#[derive(Debug, Clone, Copy, Default)]
pub struct Temperature {
    /// Timestamp in seconds
    pub timestamp: f64,
    /// Temperature in Celsius
    pub temperature: f64,
    /// Variance
    pub variance: f64,
}

/// GPS/NMEA message processor
pub struct GpsProcessor {
    /// Last frame ID
    pub frame_id: String,
    /// Last timestamp
    pub timestamp: f64,
    /// Last parsed GPS position
    pub last_position: Option<GpsPosition>,
}

impl GpsProcessor {
    /// Create a new GPS processor
    pub fn new() -> Self {
        Self {
            frame_id: String::new(),
            timestamp: 0.0,
            last_position: None,
        }
    }

    /// Parse an nmea_msgs/Sentence message
    ///
    /// Layout:
    /// - header (seq: 4, stamp: 8, frame_id: 4+len)
    /// - sentence (string: 4+len)
    pub fn parse_nmea(&mut self, data: &[u8]) -> Result<NmeaSentence> {
        if data.len() < 20 {
            return Err(RosBagError::ParseError("Data too short for NMEA message".into()));
        }

        let mut offset = 0;

        // Parse header
        // seq (4 bytes)
        offset += 4;

        // stamp.sec (4 bytes) + stamp.nsec (4 bytes)
        let sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        self.timestamp = sec as f64 + nsec as f64 * 1e-9;
        offset += 8;

        // frame_id (string: 4 byte length + chars)
        let frame_id_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        self.frame_id = String::from_utf8_lossy(&data[offset..offset + frame_id_len]).to_string();
        offset += frame_id_len;

        // sentence (string: 4 byte length + chars)
        let sentence_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let sentence = String::from_utf8_lossy(&data[offset..offset + sentence_len]).to_string();

        // Extract sentence type (first field after $)
        let sentence_type = if sentence.starts_with('$') {
            sentence[1..].split(',').next().unwrap_or("").to_string()
        } else {
            String::new()
        };

        // Try to parse GPS position if it's a GPGGA or GPRMC sentence
        if sentence_type == "GPGGA" || sentence_type == "GNGGA" {
            self.last_position = self.parse_gpgga(&sentence);
        }

        Ok(NmeaSentence {
            timestamp: self.timestamp,
            sentence,
            sentence_type,
        })
    }

    /// Parse GPGGA sentence for position
    fn parse_gpgga(&self, sentence: &str) -> Option<GpsPosition> {
        let parts: Vec<&str> = sentence.split(',').collect();
        if parts.len() < 10 {
            return None;
        }

        // Parse latitude (ddmm.mmmmm format)
        let lat_str = parts.get(2)?;
        let lat_dir = parts.get(3)?;
        let latitude = self.parse_nmea_coord(lat_str, lat_dir)?;

        // Parse longitude (dddmm.mmmmm format)
        let lon_str = parts.get(4)?;
        let lon_dir = parts.get(5)?;
        let longitude = self.parse_nmea_coord(lon_str, lon_dir)?;

        // Fix quality
        let fix_quality = parts.get(6)?.parse().unwrap_or(0);

        // Number of satellites
        let num_satellites = parts.get(7)?.parse().unwrap_or(0);

        // HDOP
        let hdop = parts.get(8)?.parse().unwrap_or(99.0);

        // Altitude
        let altitude = parts.get(9)?.parse().unwrap_or(0.0);

        Some(GpsPosition {
            latitude,
            longitude,
            altitude,
            fix_quality,
            num_satellites,
            hdop,
        })
    }

    /// Parse NMEA coordinate (ddmm.mmmmm or dddmm.mmmmm format)
    fn parse_nmea_coord(&self, coord: &str, direction: &str) -> Option<f64> {
        if coord.is_empty() {
            return None;
        }

        let coord: f64 = coord.parse().ok()?;
        let degrees = (coord / 100.0).floor();
        let minutes = coord - degrees * 100.0;
        let mut result = degrees + minutes / 60.0;

        if direction == "S" || direction == "W" {
            result = -result;
        }

        Some(result)
    }

    /// Parse a sensor_msgs/TimeReference message
    ///
    /// Layout:
    /// - header (seq: 4, stamp: 8, frame_id: 4+len)
    /// - time_ref (sec: 4, nsec: 4)
    /// - source (string: 4+len)
    pub fn parse_time_reference(&mut self, data: &[u8]) -> Result<TimeReference> {
        if data.len() < 24 {
            return Err(RosBagError::ParseError("Data too short for TimeReference".into()));
        }

        let mut offset = 0;

        // Parse header
        // seq (4 bytes)
        offset += 4;

        // stamp.sec (4 bytes) + stamp.nsec (4 bytes)
        let sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        self.timestamp = sec as f64 + nsec as f64 * 1e-9;
        offset += 8;

        // frame_id (string: 4 byte length + chars)
        let frame_id_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        self.frame_id = String::from_utf8_lossy(&data[offset..offset + frame_id_len]).to_string();
        offset += frame_id_len;

        // time_ref (sec: 4, nsec: 4)
        let ref_sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let ref_nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        let time_ref = ref_sec as f64 + ref_nsec as f64 * 1e-9;
        offset += 8;

        // source (string: 4 byte length + chars)
        let source_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let source = String::from_utf8_lossy(&data[offset..offset + source_len]).to_string();

        Ok(TimeReference {
            timestamp: self.timestamp,
            source,
            time_ref,
        })
    }

    /// Parse a sensor_msgs/Temperature message
    ///
    /// Layout:
    /// - header (seq: 4, stamp: 8, frame_id: 4+len)
    /// - temperature (8 bytes double)
    /// - variance (8 bytes double)
    pub fn parse_temperature(&mut self, data: &[u8]) -> Result<Temperature> {
        if data.len() < 32 {
            return Err(RosBagError::ParseError("Data too short for Temperature".into()));
        }

        let mut offset = 0;

        // Parse header
        // seq (4 bytes)
        offset += 4;

        // stamp.sec (4 bytes) + stamp.nsec (4 bytes)
        let sec = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        let nsec = u32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
        self.timestamp = sec as f64 + nsec as f64 * 1e-9;
        offset += 8;

        // frame_id (string: 4 byte length + chars)
        let frame_id_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        self.frame_id = String::from_utf8_lossy(&data[offset..offset + frame_id_len]).to_string();
        offset += frame_id_len;

        // temperature (8 bytes double)
        let temperature = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // variance (8 bytes double)
        let variance = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());

        Ok(Temperature {
            timestamp: self.timestamp,
            temperature,
            variance,
        })
    }

    /// Get last known GPS position
    pub fn last_position(&self) -> Option<GpsPosition> {
        self.last_position
    }
}

impl Default for GpsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gps_processor_new() {
        let proc = GpsProcessor::new();
        assert!(proc.frame_id.is_empty());
        assert!(proc.last_position.is_none());
    }
}
