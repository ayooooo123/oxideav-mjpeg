//! The registry decoder reports the size and pixel layout of the image it
//! last returned (oxideav-core `Decoder::output_video_dimensions` /
//! `output_pixel_format`): odd sizes, one image after another, each a
//! change of size and layout.
//!
//! `decoded_format/*.jpg` are single FFmpeg `mjpeg` frames of `testsrc`.
//! FFmpeg codes its 4:2:2 picture as 2×2 luma with 1×2 chroma, and its
//! 4:4:4 picture as 1×2 on every component. As FFmpeg does, the decoder
//! halves factors that are all even, so those decode as `yuvj422p` and
//! `yuvj444p`, FFmpeg's formats for them.

#![cfg(feature = "registry")]

use oxideav_core::{CodecId, CodecParameters, Frame, Packet, PixelFormat, TimeBase};

/// An image's width, height and pixel format.
type Layout = (u32, u32, PixelFormat);

#[test]
fn each_frame_reports_its_own_size_and_layout() {
    let images: [(&[u8], Layout); 3] = [
        (
            include_bytes!("decoded_format/a_420_33x17.jpg"),
            (33, 17, PixelFormat::YuvJ420P),
        ),
        (
            include_bytes!("decoded_format/b_422_34x19.jpg"),
            (34, 19, PixelFormat::YuvJ422P),
        ),
        (
            include_bytes!("decoded_format/c_444_35x18.jpg"),
            (35, 18, PixelFormat::YuvJ444P),
        ),
    ];
    let mut dec =
        oxideav_mjpeg::registry::make_decoder(&CodecParameters::video(CodecId::new("mjpeg")))
            .expect("decoder");
    let mut last = None;
    for (at, (jpeg, (w, h, format))) in images.into_iter().enumerate() {
        dec.send_packet(&Packet::new(0, TimeBase::new(1, 25), jpeg.to_vec()))
            .expect("send");
        if last.is_some() {
            assert_eq!(
                dec.output_video_dimensions(),
                last,
                "image {at}: before it is returned"
            );
        }
        let frame = match dec.receive_frame() {
            Ok(Frame::Video(frame)) => frame,
            other => panic!("image {at}: {other:?}"),
        };
        assert_eq!(dec.output_video_dimensions(), Some((w, h)), "image {at}");
        assert_eq!(dec.output_pixel_format(), Some(format), "image {at}");
        let planes = frame.image_planes();
        assert_eq!(planes.len(), format.plane_count(), "image {at}: planes");
        for (i, plane) in planes.iter().enumerate() {
            let (_, rows) = format.plane_dimensions(i, w, h).unwrap();
            let row = format.plane_row_bytes(i, w).unwrap();
            assert!(plane.stride >= row, "image {at}: plane {i} stride");
            assert!(
                plane.data.len() >= plane.stride * (rows as usize - 1) + row,
                "image {at}: plane {i} rows"
            );
        }
        last = Some((w, h));
    }
}
