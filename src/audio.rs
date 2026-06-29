pub struct AudioHandler {
    _handle: rodio::MixerDeviceSink,
    player: rodio::Player,
}
impl AudioHandler {
    const HOVER_SOUND: &[u8] = include_bytes!("../assets/hover-click.wav");

    pub fn new() -> AudioHandler {
        let handle = rodio::DeviceSinkBuilder::open_default_sink()
            .expect("open default audio stream");
        let player = rodio::Player::connect_new(handle.mixer());
        AudioHandler {
            _handle: handle,
            player,
        }
    }

    pub fn play_hover_sound(&self) {
        let cursor = std::io::Cursor::new(AudioHandler::HOVER_SOUND);
        let source = rodio::Decoder::try_from(cursor).unwrap();
        self.play_sound_immediate(source);
    }

    fn play_sound_immediate(
        &self,
        source: rodio::Decoder<std::io::Cursor<&'static [u8]>>,
    ) {
        if let Some(Ok(config)) =
            crate::config::APP_CONFIG.get().map(|m| m.lock())
        {
            if !config.sound_effects {
                return;
            }
        }
        self.player.skip_one();
        self.player.append(source);
    }
}

lazy_static::lazy_static! {
    pub static ref AUDIO_HANDLER: std::sync::Arc<AudioHandler>  = std::sync::Arc::new(AudioHandler::new());
}
