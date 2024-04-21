use anyhow::Result;
use ffmpeg::ffi::{swr_alloc, swr_alloc_set_opts, swr_convert};
use ffmpeg_next as ffmpeg;
use std::{ffi::CString, path::Path};

const SAMPLE_RATE: u32 = 16000;

fn to_c_char(s: &str) -> Result<*const std::ffi::c_char> {
    Ok(CString::new(s)?.as_ptr())
}

pub fn read_audio(file: &Path) -> Result<Vec<f32>> {
    ffmpeg::init().unwrap();

    let mut audio: Vec<f32> = Vec::new();

    let mut ictx = ffmpeg::format::input(&file)?;
    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let stream_idx = input.index();

    let decoder_ctx = ffmpeg::codec::Context::from_parameters(input.parameters())?;
    let mut decoder = decoder_ctx.decoder().audio()?;

    println!("{:?}", decoder.format().bytes());

    unsafe {
        let swr = swr_alloc_set_opts(
            std::ptr::null_mut::<ffmpeg::ffi::SwrContext>(),
            ffmpeg::ffi::AV_CH_LAYOUT_MONO as i64,
            ffmpeg::ffi::AVSampleFormat::AV_SAMPLE_FMT_S16,
            SAMPLE_RATE as i32,
            decoder.channel_layout().bits() as i64,
            decoder.format().into(),
            decoder.rate() as i32,
            0,
            std::ptr::null_mut(),
        );
        let ret = ffmpeg::ffi::swr_init(swr);
        if ret != 0 {
            return Err(ffmpeg::Error::from(ret as i32).into());
        }

        let mut receive_decoded_samples =
            |decoder: &mut ffmpeg::decoder::Audio| -> Result<(), ffmpeg::Error> {
                let mut decoded = ffmpeg::frame::audio::Audio::empty();
                while decoder.receive_frame(&mut decoded).is_ok() {
                    let f = ffmpeg::frame::audio::Audio::empty();
                    let data = decoded.data(0);

                    let nb_samples = decoded.samples() as u64;

                    let delay = ffmpeg::ffi::swr_get_delay(swr, decoder.rate() as i64);
                    let mut nr_samples = ffmpeg::ffi::av_rescale_rnd(
                        delay + nb_samples as i64,
                        SAMPLE_RATE as i64,
                        decoder.rate() as i64,
                        ffmpeg::ffi::AVRounding::AV_ROUND_UP,
                    );

                    ffmpeg::ffi::av_samples_alloc(
                        (*f.as_ptr()).data.as_ptr() as *mut _,
                        std::ptr::null::<i32>().cast_mut(),
                        1,
                        nr_samples as i32,
                        ffmpeg::ffi::AVSampleFormat::AV_SAMPLE_FMT_S16,
                        0,
                    );

                    let d_ptr = data.as_ptr();
                    let f_ptr = (*f.as_ptr()).data.as_ptr();
                    nr_samples = swr_convert(
                        swr,
                        f_ptr as *mut _,
                        nr_samples as i32,
                        d_ptr as *mut _,
                        nb_samples as i32,
                    ) as i64;

                    if nr_samples < 0 {
                        return Err(ffmpeg::Error::from(nr_samples as i32));
                    }

                    let mut d = f.data(0).iter().step_by(4);
                    let mut v = Vec::new();
                    v.reserve(f.data(0).len() / 4);
                    for _ in 0..v.len() {
                        let bytes = [
                            d.next().unwrap().to_owned(),
                            d.next().unwrap().to_owned(),
                            d.next().unwrap().to_owned(),
                            d.next().unwrap().to_owned(),
                        ];
                        v.push(f32::from_ne_bytes(bytes));
                    }

                    audio.extend_from_slice(&v);
                }

                Ok(())
            };

        for (stream, packet) in ictx.packets() {
            if stream.index() == stream_idx {
                decoder.send_packet(&packet)?;
                receive_decoded_samples(&mut decoder)?;
            }
        }
    }

    Ok(audio)
}
