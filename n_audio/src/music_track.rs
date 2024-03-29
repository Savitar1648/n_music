use crate::{from_path_to_name_without_ext, TrackTime, PROBE};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia_core::meta::{StandardTagKey, Value};

/// The basics where everything is built upon
pub struct MusicTrack {
    file: File,
    name: String,
    ext: String,
}

impl MusicTrack {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<OsStr>,
    {
        let path = Path::new(&path);
        let file = File::open(path)?;
        Ok(MusicTrack {
            file,
            name: from_path_to_name_without_ext(path),
            ext: path
                .extension()
                .ok_or(String::from("no extension"))?
                .to_str()
                .unwrap()
                .to_string(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the `FormatReader` provided by Symphonia
    pub fn get_format(&self) -> Box<dyn FormatReader> {
        let file = self.file.try_clone().expect("Can't copy file");
        let media_stream = MediaSourceStream::new(Box::new(file), std::default::Default::default());
        let mut hint = Hint::new();
        hint.with_extension(self.ext.as_ref());
        let meta_ops = MetadataOptions::default();
        let fmt_ops = FormatOptions {
            enable_gapless: true,
            ..Default::default()
        };
        let probed = PROBE
            .format(&hint, media_stream, &fmt_ops, &meta_ops)
            .expect("Format not supported");
        probed.format
    }

    pub fn get_duration(&self) -> TrackTime {
        let format = self.get_format();
        Self::get_duration_from_format(format.as_ref())
    }

    pub fn get_artist(&mut self) -> String {
        let mut format = self.get_format();
        Self::get_artist_from_format(&mut format)
    }

    pub fn get_cover(&mut self) -> Vec<u8> {
        let mut format = self.get_format();
        Self::get_cover_from_format(&mut format)
    }

    pub fn get_duration_from_format<F: FormatReader + ?Sized>(format: &F) -> TrackTime {
        let track = format.default_track().expect("Can't load tracks");
        let time_base = track.codec_params.time_base.unwrap();

        let duration = track
            .codec_params
            .n_frames
            .map(|frames| track.codec_params.start_ts + frames)
            .unwrap();
        let time = time_base.calc_time(duration);

        TrackTime {
            ts_secs: 0,
            ts_frac: 0.0,
            dur_secs: time.seconds,
            dur_frac: time.frac,
        }
    }

    pub fn get_artist_from_format(format: &mut Box<dyn FormatReader>) -> String {
        let metadata = format.metadata();
        let current = metadata.current().unwrap().clone();
        let tags = current.tags().to_vec();

        if let Value::String(artist) = &tags
            .iter()
            .filter(|tag| tag.std_key.is_some())
            .find(|tag| tag.std_key == Some(StandardTagKey::Artist))
            .unwrap()
            .value
        {
            artist.to_string()
        } else {
            String::from("ARTIST")
        }
    }

    pub fn get_cover_from_format(format: &mut Box<dyn FormatReader>) -> Vec<u8> {
        let metadata = format.metadata();
        let current = metadata.current().unwrap().clone();
        let tags = current.tags().to_vec();

        // if let Value::String(cover_encoded) =
        //     &tags.iter().find(|tag| tag.std_key.is_none()).unwrap().value
        // {
        //     base64::prelude::BASE64_STANDARD
        //         .decode(cover_encoded.as_bytes())
        //         .unwrap_or(vec![])
        // } else {
        //     vec![]
        // }

        vec![]
    }
}
