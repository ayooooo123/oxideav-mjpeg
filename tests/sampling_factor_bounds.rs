#[test]
fn valid_jpeg_rejects_out_of_range_component_sampling() {
    let valid = include_bytes!("fixtures/sampling/samp_4x2.jpg");
    assert!(oxideav_mjpeg::info(valid).is_ok());
    let sof = valid.windows(2).position(|marker| matches!(marker, [0xff, 0xc0 | 0xc1 | 0xc2])).expect("SOF");
    let components = valid[sof + 9] as usize;
    for component in 0..components {
        for factor in [0x01, 0x10, 0x00, 0x51, 0x15, 0xff] {
            let mut bytes = valid.to_vec();
            bytes[sof + 11 + component * 3] = factor;
            assert!(oxideav_mjpeg::info(&bytes).is_err(), "component {component}, factor {factor:#x}");
            assert!(oxideav_mjpeg::decode(&bytes).is_err(), "component {component}, factor {factor:#x}");
        }
    }
}
