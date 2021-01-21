use crate::constants;
use gstreamer;
use log::info;

pub fn check_plugins() {
    gstreamer::init().unwrap();

    let plugin = constants::PLUGIN_WEBRTC;
    let registry = gstreamer::Registry::get();
    let missing = plugin
        .iter()
        .filter(|n| registry.find_plugin(n).is_none())
        .cloned()
        .collect::<Vec<_>>();

    info!("Missing plugins: {:?}", missing);
}
