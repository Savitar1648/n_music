use crate::{FileTracks, LoaderMessage, PlayerMessage};
use n_audio::TrackTime;
use std::sync::mpsc::{Receiver, Sender};
use vizia::image;
use vizia::image::ImageFormat;
use vizia::prelude::*;

enum AppMessage {
    Loaded(Vec<LoaderMessage>),
    InitTracks(FileTracks),
    Clicked(usize),
    SliderDuration(f32),
    SliderVolume(f32),
    TimeUpdate(TrackTime),
    CurrentUpdated(usize),
}

#[derive(Lens)]
struct AppData {
    tracks: FileTracks,
    current: usize,
    duration: f32,
    track_duration: u64,
    loaded_info: u64,
    volume: f32,
    current_name: String,
    formatted_ts: String,
    formatted_dur: String,
    tx: Sender<PlayerMessage>,
}

impl Model for AppData {
    fn event(&mut self, ctx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, _meta| match app_event {
            AppMessage::InitTracks(tracks) => self.tracks = tracks.clone(),
            AppMessage::Loaded(loaded_messages) => {
                for loaded in loaded_messages {
                    match loaded {
                        LoaderMessage::Duration(i, dur) => {
                            self.tracks[*i].duration = *dur;
                            if *i == self.current {
                                self.track_duration = *dur;
                            }
                        }
                        LoaderMessage::Artist(i, artist) => {
                            self.tracks[*i].artist = artist.clone();
                        }
                        LoaderMessage::Image(i, cover) => {
                            self.tracks[*i].cover = cover.clone();
                        }
                    }
                }
                self.loaded_info += 1;
            }
            AppMessage::Clicked(i) => {
                self.tx
                    .send(PlayerMessage::Clicked(*i))
                    .expect("can't send clicked song");
            }
            AppMessage::SliderDuration(value) => {
                let seeked_time = *value * self.track_duration as f32;
                self.tx
                    .send(PlayerMessage::Seek(seeked_time))
                    .expect("can't send seek message");
            }
            AppMessage::SliderVolume(value) => {
                self.volume = *value;
                self.tx
                    .send(PlayerMessage::Volume(*value))
                    .expect("can't send volume message");
            }
            AppMessage::TimeUpdate(time) => {
                let duration = time.dur_secs;
                let ts = time.ts_secs as f64 + time.ts_frac;
                let passed = ts as f32 / duration as f32;

                if duration != self.track_duration {
                    self.track_duration = duration;
                    self.formatted_dur = format!("{:02}:{:02}", duration / 60, duration % 60);
                }

                if passed != self.duration {
                    self.duration = passed;
                    self.formatted_ts = format!("{:02}:{:02}", ts as u64 / 60, ts as u64 % 60)
                }
            }
            AppMessage::CurrentUpdated(i) => {
                self.tracks[self.current].current = false;
                self.tracks[*i].current = true;
                self.current = *i;
                self.track_duration = self.tracks[*i].duration;
                self.formatted_dur = format!(
                    "{:02}:{:02}",
                    self.track_duration / 60,
                    self.track_duration % 60
                );
                self.current_name = self.tracks[*i].name.clone();
                ctx.emit(WindowEvent::SetTitle(format!(
                    "N Music - {}",
                    self.current_name
                )));
            }
        });
    }
}

pub fn run(rx: Receiver<PlayerMessage>, tx: Sender<PlayerMessage>) {
    Application::new(|ctx| {
        AppData {
            tracks: FileTracks { tracks: vec![] },
            current: 0,
            duration: 0.0,
            loaded_info: 0,
            volume: 1.0,
            track_duration: 0,
            current_name: String::new(),
            formatted_ts: String::new(),
            formatted_dur: String::new(),
            tx,
        }
        .build(ctx);

        ctx.add_stylesheet(include_style!("style/main.css"))
            .unwrap();
        ctx.emit(EnvironmentEvent::SetThemeMode(AppTheme::System));

        VStack::new(ctx, |ctx| {
            VirtualList::new(ctx, AppData::tracks, 60.0, move |ctx, i, item| {
                let track = item.get(ctx);
                HStack::new(ctx, |ctx| {
                    if !track.cover.is_empty() {
                        let mut image_format = ImageFormat::Jpeg;
                        let image =
                            match image::load_from_memory_with_format(&track.cover, image_format) {
                                Ok(image) => image,
                                Err(_) => {
                                    image_format = ImageFormat::Png;
                                    match image::load_from_memory_with_format(
                                        &track.cover,
                                        image_format,
                                    ) {
                                        Ok(image) => image,
                                        Err(_) => {
                                            image_format = ImageFormat::WebP;
                                            image::load_from_memory_with_format(
                                                &track.cover,
                                                image_format,
                                            )
                                            .unwrap()
                                        }
                                    }
                                }
                            };

                        ctx.load_image("", image, ImageRetentionPolicy::DropWhenNoObservers);
                    }
                    VStack::new(ctx, |ctx| {
                        Label::new(ctx, &track.name).overflowx(Overflow::Hidden);
                        Label::new(ctx, &track.artist).overflowx(Overflow::Hidden);
                    })
                    .text_align(TextAlign::Left)
                    .width(Percentage(80.0));

                    Label::new(
                        ctx,
                        &format!("{:02}:{:02}", track.duration / 60, track.duration % 60),
                    )
                    .text_align(TextAlign::Right)
                    .width(Percentage(20.0));
                })
                .hoverable(true)
                .toggle_class("song", !track.current)
                .toggle_class("song-current", track.current)
                .on_mouse_down(move |ctx, button| {
                    if let MouseButton::Left = button {
                        ctx.emit(AppMessage::Clicked(i));
                    }
                })
            });
            VStack::new(ctx, |ctx| {
                MenuDivider::new(ctx)
                    .height(Pixels(2.0))
                    .width(Percentage(98.0))
                    .space(Percentage(1.0));
                HStack::new(ctx, |ctx| {
                    Label::new(ctx, AppData::formatted_ts)
                        .font_size(13.0)
                        .width(Stretch(2.0));
                    Slider::new(ctx, AppData::duration)
                        .width(Stretch(20.0))
                        .on_changing(|ctx, value| ctx.emit(AppMessage::SliderDuration(value)));
                    Label::new(ctx, AppData::formatted_dur)
                        .font_size(13.0)
                        .width(Stretch(2.0));
                    Slider::new(ctx, AppData::volume)
                        .width(Stretch(5.0))
                        .on_changing(|ctx, value| ctx.emit(AppMessage::SliderVolume(value)));
                })
                .height(Auto)
                .child_space(Stretch(1.0))
                .col_between(Stretch(1.0));
                HStack::new(ctx, |ctx| {
                    Label::new(ctx, AppData::current_name);
                    Button::new(ctx, |ctx| Label::new(ctx, "previous"))
                        .variant(ButtonVariant::Text);
                    Button::new(ctx, |ctx| Label::new(ctx, "stop/play"))
                        .variant(ButtonVariant::Text);
                    Button::new(ctx, |ctx| Label::new(ctx, "next")).variant(ButtonVariant::Text);
                })
                .height(Auto)
                .col_between(Stretch(1.0))
                .child_space(Stretch(1.0));
            })
            .min_height(Pixels(75.0))
            .max_height(Pixels(75.0));
        });
    })
    .should_poll()
    .on_idle(move |ctx| {
        while let Ok(message) = rx.try_recv() {
            match message {
                PlayerMessage::InitTracks(tracks) => {
                    ctx.emit(AppMessage::InitTracks(tracks));
                }
                PlayerMessage::Loaded(loaded) => {
                    ctx.emit(AppMessage::Loaded(loaded));
                }
                PlayerMessage::TimeUpdate(time) => {
                    ctx.emit(AppMessage::TimeUpdate(time));
                }
                PlayerMessage::CurrentUpdated(i) => {
                    ctx.emit(AppMessage::CurrentUpdated(i));
                }
                _ => {}
            }
        }
    })
    .vsync(true)
    .title("N Music")
    .inner_size((400, 600))
    .run();
}
