use crate::sandbox::{SIMULATION_HEIGHT, SIMULATION_WIDTH};
use gstreamer::glib::object::{Cast, ObjectExt};
use gstreamer::{
    Buffer, Element, ElementExt, ElementExtManual, Event, Format, GstBinExt, MessageView, Pipeline,
    State, CLOCK_TIME_NONE,
};
use gstreamer_app::AppSrc;
use gstreamer_video::{VideoFormat, VideoInfo};
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct VideoRecorder {
    pub is_recording: bool,
    pipeline: Pipeline,
    app_src: AppSrc,
    filesink: Element,
    video_file_location: PathBuf,
}

impl VideoRecorder {
    pub fn new() -> Self {
        gstreamer::init().unwrap();

        let pipeline = gstreamer::parse_launch(
            "appsrc name=src is-live=true do-timestamp=true ! videoconvert ! x264enc ! mp4mux ! filesink name=filesink",
        )
        .unwrap()
        .downcast::<Pipeline>()
        .unwrap();

        let app_src = pipeline
            .get_by_name("src")
            .unwrap()
            .dynamic_cast::<AppSrc>()
            .unwrap();
        let video_info = VideoInfo::new(
            VideoFormat::Rgba,
            SIMULATION_WIDTH as u32,
            SIMULATION_HEIGHT as u32,
        )
        .build()
        .unwrap();
        app_src.set_caps(Some(&video_info.to_caps().unwrap()));
        app_src.set_property_format(Format::Time);

        let filesink = pipeline.get_by_name("filesink").unwrap();

        let mut video_file_location = dirs_next::video_dir().unwrap();
        video_file_location.push("sandbox");
        let _ = fs::create_dir(&video_file_location);
        video_file_location.push("placeholder.mp4");

        Self {
            is_recording: false,
            pipeline,
            app_src,
            filesink,
            video_file_location,
        }
    }

    pub fn start_recording(&mut self) {
        self.is_recording = true;
        self.video_file_location.set_file_name(&format!(
            "sandbox-{}.mp4",
            humantime::format_rfc3339_seconds(SystemTime::now())
        ));
        self.filesink
            .set_property("location", &self.video_file_location.to_str().unwrap())
            .unwrap();
        self.pipeline.set_state(State::Playing).unwrap();
    }

    pub fn upload_frame(&mut self, frame: &[u8]) {
        if self.is_recording {
            self.app_src
                .push_buffer(Buffer::from_slice(frame.to_vec()))
                .unwrap();
        }
    }

    pub fn stop_recording(&mut self) {
        self.is_recording = false;
        self.pipeline.send_event(Event::new_eos().build());
        let bus = self.pipeline.get_bus().unwrap();
        for message in bus.iter_timed(CLOCK_TIME_NONE) {
            match message.view() {
                MessageView::Eos(..) => break,
                _ => {}
            }
        }
        self.pipeline.set_state(State::Null).unwrap();
    }
}
