use qrcode::QrCode;
use qrcode::render::unicode::Dense1x2;

pub fn ansi(url: &str) -> String {
    QrCode::new(url.as_bytes())
        .map(|code| {
            code.render::<Dense1x2>()
                .dark_color(Dense1x2::Light)
                .light_color(Dense1x2::Dark)
                .build()
        })
        .unwrap_or_default()
}
