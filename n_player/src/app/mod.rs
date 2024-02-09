use crate::{FileTracks, LoaderMessage, PlayerMessage};
use n_audio::TrackTime;
use std::sync::mpsc::{Receiver, Sender};
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
                let passed = (time.ts_secs as f64 + time.ts_frac) as f32 / duration as f32;

                if duration != self.track_duration {
                    self.track_duration = duration;
                }

                if passed != self.duration {
                    self.duration = passed;
                }
            }
            AppMessage::CurrentUpdated(i) => {
                self.current = *i;
                self.track_duration = self.tracks[*i].duration;
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
            tx,
        }
        .build(ctx);

        ctx.add_stylesheet(include_style!("style/main.css"))
            .unwrap();

        VStack::new(ctx, |ctx| {
            Binding::new(ctx, AppData::current, |ctx, current| {
                Binding::new(ctx, AppData::loaded_info, move |ctx, _| {
                    let current = current.get(ctx);
                    VirtualList::new(ctx, AppData::tracks, 40.0, move |ctx, i, item| {
                        let track = item.get_fallible(ctx);
                        HStack::new(ctx, |ctx| {
                            if let Some(track) = track {
                                VStack::new(ctx, |ctx| {
                                    Label::new(ctx, &track.name).overflowx(Overflow::Hidden);
                                    Label::new(ctx, &track.artist).text_wrap(true);
                                })
                                .text_align(TextAlign::Left)
                                .width(Percentage(80.0));

                                Label::new(
                                    ctx,
                                    &format!(
                                        "{:02}:{:02}",
                                        track.duration / 60,
                                        track.duration % 60
                                    ),
                                )
                                .text_align(TextAlign::Right)
                                .width(Percentage(20.0));
                            }
                        })
                        .hoverable(true)
                        .class(if current == i { "song-current" } else { "song" })
                        .on_mouse_down(move |ctx, button| {
                            if let MouseButton::Left = button {
                                ctx.emit(AppMessage::Clicked(i));
                            }
                        })
                    });
                });
                Binding::new(ctx, AppData::duration, |ctx, duration| {
                    Binding::new(ctx, AppData::track_duration, move |ctx, track_duration| {
                        let duration_value = duration.get(ctx);
                        let track_duration = track_duration.get(ctx);
                        VStack::new(ctx, |ctx| {
                            MenuDivider::new(ctx)
                                .height(Pixels(2.0))
                                .width(Percentage(98.0))
                                .space(Percentage(1.0));
                            HStack::new(ctx, |ctx| {
                                Label::new(
                                    ctx,
                                    &format!(
                                        "{:02}:{:02}",
                                        (duration_value * track_duration as f32) as u64 / 60,
                                        (duration_value * track_duration as f32) as u64 % 60
                                    ),
                                )
                                .font_size(12.0)
                                .width(Stretch(2.0));
                                Slider::new(ctx, duration).width(Stretch(20.0)).on_changing(
                                    |ctx, value| ctx.emit(AppMessage::SliderDuration(value)),
                                );
                                Label::new(
                                    ctx,
                                    &format!(
                                        "{:02}:{:02}",
                                        track_duration / 60,
                                        track_duration % 60
                                    ),
                                )
                                .font_size(12.0)
                                .width(Stretch(2.0));
                                Slider::new(ctx, AppData::volume)
                                    .width(Stretch(5.0))
                                    .on_changing(|ctx, value| {
                                        ctx.emit(AppMessage::SliderVolume(value))
                                    });
                            })
                            .child_space(Stretch(1.0))
                            .col_between(Stretch(1.0));
                            HStack::new(ctx, |ctx| {
                                Label::new(ctx, AppData::current_name);
                                Button::new(ctx, |_| {}, |ctx| Label::new(ctx, "previous"));
                                Button::new(ctx, |_| {}, |ctx| Label::new(ctx, "stop/play"));
                                Button::new(ctx, |_| {}, |ctx| Label::new(ctx, "next"));
                            })
                            .col_between(Stretch(1.0))
                            .child_space(Stretch(1.0));
                        })
                        .min_height(Pixels(100.0))
                        .max_height(Pixels(100.0));
                    });
                });
            });
        });
    })
    // .should_poll()
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
    .title("N Music")
    .inner_size((400, 600))
    .run();
}
