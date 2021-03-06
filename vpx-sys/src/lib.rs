// TODO do w/out the unions?
#![feature(untagged_unions)]

pub mod vpx;

#[cfg(test)]
mod tests {
    use super::vpx::*;
    use std::mem;
    use std::ffi::CStr;
    #[test]
    fn version() {
        println!("{}", unsafe {
            CStr::from_ptr(vpx_codec_version_str()).to_string_lossy()
        });
        println!("{}", unsafe {
            CStr::from_ptr(vpx_codec_build_config()).to_string_lossy()
        });
    }
    #[test]
    fn encode() {
        let w = 360;
        let h = 360;
        let align = 32;
        let kf_interval = 10;
        let mut raw = unsafe { mem::uninitialized() };
        let mut ctx = unsafe { mem::uninitialized() };

        let ret = unsafe { vpx_img_alloc(&mut raw, vpx_img_fmt::VPX_IMG_FMT_I420, w, h, align) };
        if ret.is_null() {
            panic!("Image allocation failed");
        }
        mem::forget(ret); // raw and ret are the same
        print!("{:#?}", raw);

        let mut cfg = unsafe { mem::uninitialized() };
        let mut ret = unsafe { vpx_codec_enc_config_default(vpx_codec_vp9_cx(), &mut cfg, 0) };

        if ret != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("Default Configuration failed");
        }

        cfg.g_w = w;
        cfg.g_h = h;
        cfg.g_timebase.num = 1;
        cfg.g_timebase.den = 30;
        cfg.rc_target_bitrate = 100 * 1014;

        ret = unsafe {
            vpx_codec_enc_init_ver(
                &mut ctx,
                vpx_codec_vp9_cx(),
                &mut cfg,
                0,
                VPX_ENCODER_ABI_VERSION as i32,
            )
        };

        if ret != vpx_codec_err_t::VPX_CODEC_OK {
            panic!("Codec Init failed");
        }

        let mut out = 0;
        for i in 0..100 {
            let mut flags = 0;
            if i % kf_interval == 0 {
                flags |= VPX_EFLAG_FORCE_KF;
            }
            unsafe {
                let ret = vpx_codec_encode(
                    &mut ctx,
                    &mut raw,
                    i,
                    1,
                    flags as i64,
                    VPX_DL_GOOD_QUALITY as u64,
                );
                if ret != vpx_codec_err_t::VPX_CODEC_OK {
                    panic!("Encode failed {:?}", ret);
                }

                let mut iter = mem::zeroed();
                loop {
                    let pkt = vpx_codec_get_cx_data(&mut ctx, &mut iter);

                    if pkt.is_null() {
                        break;
                    } else {
                        println!("{:#?}", unsafe { *pkt }.kind);
                        out = 1;
                    }
                }
            }
        }

        if out != 1 {
            panic!("No packet produced");
        }
    }
}
