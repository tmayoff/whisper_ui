use anyhow::Result;
use ffmpeg_next as ffmpeg;
use std::path::PathBuf;

pub fn read_audio(file: PathBuf) -> Result<Vec<u8>> {
    ffmpeg::init().unwrap();

    let mut audio: Vec<u8> = Vec::new();

    let mut ictx = ffmpeg::format::input(&file)?;
    let input = ictx
        .streams()
        .best(ffmpeg::media::Type::Audio)
        .ok_or(ffmpeg::Error::StreamNotFound)?;
    let stream_idx = input.index();

    let decoder_ctx = ffmpeg::codec::Context::from_parameters(input.parameters())?;
    let mut decoder = decoder_ctx.decoder().audio()?;

    let mut receive_decoded_samples =
        |decoder: &mut ffmpeg::decoder::Audio| -> Result<(), ffmpeg::Error> {
            let mut decoded = ffmpeg::frame::audio::Audio::empty();
            while decoder.receive_frame(&mut decoded).is_ok() {
                //let mut f = ffmpeg::frame::audio::Audio::empty();
                let data = decoded.data(0);
                audio.extend_from_slice(data);
            }
            Ok(())
        };

    for (stream, packet) in ictx.packets() {
        if stream.index() == stream_idx {
            decoder.send_packet(&packet)?;
            receive_decoded_samples(&mut decoder)?;
        }
    }

    Ok(audio)
}
