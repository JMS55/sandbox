use anyhow::{Context, Error};
use flume::{unbounded as unbounded_channel, Sender};
use gstreamer::event::Eos;
use gstreamer::glib::object::{Cast, ObjectExt};
use gstreamer::{
    Buffer as GstBuffer, Element, ElementExt, ElementExtManual, Fraction, GstBinExt, MessageView,
    Pipeline, State, CLOCK_TIME_NONE,
};
use gstreamer_app::AppSrc;
use gstreamer_video::{VideoFormat, VideoInfo};
use pixels::wgpu::*;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::SystemTime;

pub struct VideoRecorder {
    pub is_recording: bool,
    pub buffer_sender: Sender<(Buffer, u32, u32)>,
    buffers_sent: u32,
    buffers_processed: Arc<AtomicU32>,

    pipeline: Pipeline,
    filesink: Element,
    video_file_location: PathBuf,
}

impl VideoRecorder {
    pub fn new(device: Arc<Device>) -> Result<Self, Error> {
        gstreamer::init().context("Failed to initialize gstreamer")?;

        let pipeline = gstreamer::parse_launch(
            "appsrc name=src format=time is-live=true do-timestamp=true caps=video/x-raw,format=BGRA,framerate=60/1 !
            videoconvert !
            videoscale method=nearest-neighbour ! video/x-raw,width=1920,height=1080 !
            videorate ! video/x-raw,framerate=60/1 !
            x264enc !
            mp4mux !
            filesink name=filesink",
        )
        .context("Failed to create the pipeline")?
        .downcast::<Pipeline>()
        .map_err(|_| Error::msg("Failed to downcast the pipeline"))?;
        let app_src = pipeline
            .get_by_name("src")
            .context("Failed to find the appsrc element")?
            .downcast::<AppSrc>()
            .map_err(|_| Error::msg("Failed to downcast the appsrc element"))?;
        let filesink = pipeline
            .get_by_name("filesink")
            .context("Failed to find filesink appsrc element")?;

        let mut video_file_location = dirs_next::video_dir()
            .context("Failed to determine the video destination directory")?;
        video_file_location.push("sandbox");
        fs::create_dir_all(&video_file_location)
            .context("Failed to create the video destination directory")?;
        video_file_location.push("placeholder");

        let (buffer_sender, buffer_receiver) = unbounded_channel::<(Buffer, u32, u32)>();
        let buffers_processed = Arc::new(AtomicU32::new(0));
        thread::spawn({
            let buffers_processed = buffers_processed.clone();
            move || {
                for (buffer, width, height) in buffer_receiver.iter() {
                    let future = buffer.map_read(0, (width * height * 4) as u64);
                    device.poll(Maintain::Wait);
                    if let Ok(buffer) = pollster::block_on(future) {
                        let video_info = VideoInfo::builder(VideoFormat::Bgra, width, height)
                            .fps(Fraction::new(60, 1))
                            .build()
                            .unwrap();
                        app_src.set_caps(Some(&video_info.to_caps().unwrap()));
                        app_src
                            .push_buffer(GstBuffer::from_slice(buffer.as_slice().to_vec()))
                            .unwrap();
                    }
                    buffers_processed.fetch_add(1, Ordering::SeqCst);
                }
            }
        });

        Ok(Self {
            is_recording: false,
            buffers_sent: 0,
            buffers_processed,

            buffer_sender,
            pipeline,
            filesink,
            video_file_location,
        })
    }

    pub fn start_recording(&mut self) {
        assert!(!self.is_recording);
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

    pub fn upload_buffer(&mut self, buffer: Buffer, screen_width: u32, screen_height: u32) {
        assert!(self.is_recording);

        let _ = self
            .buffer_sender
            .send((buffer, screen_width, screen_height));
        self.buffers_sent += 1;
    }

    pub fn stop_recording(&mut self) {
        assert!(self.is_recording);
        self.is_recording = false;

        while self.buffers_sent != self.buffers_processed.load(Ordering::SeqCst) {}
        self.buffers_sent = 0;
        self.buffers_processed.store(0, Ordering::SeqCst);

        self.pipeline.send_event(Eos::new());
        let bus = self.pipeline.get_bus().unwrap();
        for message in bus.iter_timed(CLOCK_TIME_NONE) {
            match message.view() {
                MessageView::Eos(..) => break,
                MessageView::Error(error) => {
                    eprintln!("Error recording video: {:?}", error);
                    break;
                }
                _ => {}
            }
        }
        self.pipeline.set_state(State::Null).unwrap();
    }
}

impl Drop for VideoRecorder {
    fn drop(&mut self) {
        if self.is_recording {
            self.start_recording()
        }
    }
}
