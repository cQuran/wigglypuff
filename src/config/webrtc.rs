use crate::constants;
use gstreamer;
use log::info;

pub fn config_gstreamer(){
    gstreamer::init().unwrap();

    let needed = constants::PLUGIN_WEBRTC;
    let registry = gstreamer::Registry::get();
    let missing = needed
        .iter()
        .filter(|n| registry.find_plugin(n).is_none())
        .cloned()
        .collect::<Vec<_>>();

    info!("Missing plugins: {:?}", missing);
}
