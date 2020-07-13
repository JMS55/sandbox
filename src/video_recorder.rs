use crate::sandbox::{SANDBOX_HEIGHT, SANDBOX_WIDTH};
use gstreamer::event::Eos;
use gstreamer::glib::object::{Cast, ObjectExt};
use gstreamer::{
    Buffer, Element, ElementExt, ElementExtManual, GstBinExt, MessageView, Pipeline, State,
    CLOCK_TIME_NONE,
};
use gstreamer_app::AppSrc;
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

        let pipeline = gstreamer::parse_launch(&format!(
            "appsrc name=src format=time is-live=true do-timestamp=true caps=video/x-raw,format=RGBA,width={},height={},framerate=60/1 !
            videoconvert !
            videoscale method=nearest-neighbour ! video/x-raw,width=1920,height=1080 !
            videorate ! video/x-raw,framerate=60/1 !
            x264enc !
            mp4mux !
            filesink name=filesink",
            SANDBOX_WIDTH, SANDBOX_HEIGHT
        ))
        .unwrap()
        .downcast::<Pipeline>()
        .unwrap();

        let app_src = pipeline
            .get_by_name("src")
            .unwrap()
            .downcast::<AppSrc>()
            .unwrap();

        let filesink = pipeline.get_by_name("filesink").unwrap();

        let mut video_file_location = dirs_next::video_dir().unwrap();
        video_file_location.push("sandbox");
        let _ = fs::create_dir(&video_file_location);
        video_file_location.push("placeholder");

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
        self.pipeline.send_event(Eos::new());
        let bus = self.pipeline.get_bus().unwrap();
        for message in bus.iter_timed(CLOCK_TIME_NONE) {
            match message.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(error) => {
                    println!("Error recording video: {:?}", error);
                    break;
                }
                _ => {}
            }
        }
        self.pipeline.set_state(State::Null).unwrap();
    }
}
