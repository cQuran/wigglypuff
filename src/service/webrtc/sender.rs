use log::info;

#[derive(Debug, Clone)]
struct Peer(Arc<PeerInner>);

#[derive(Debug, Clone)]
struct PeerWeak(Weak<PeerInner>);

#[derive(Debug)]
struct PeerInner {
    bin: gstreamer::Bin,
    webrtcbin: gstreamer::Element,
}

impl std::ops::Deref for Peer {
    type Target = PeerInner;

    fn deref(&self) -> &PeerInner {
        &self.0
    }
}

impl PeerWeak {
    fn upgrade(&self) -> Option<Peer> {
        self.0.upgrade().map(Peer)
    }
}

impl Peer {
    fn new(
        room_address: &Addr<service_room::Room>,
        room_name: &String,
        uuid: &String,
    ) -> Result<Self, Error> {
        info!(
            "[ROOM: {}] [UUID: {}] Creating WebRTC Pairing",
            room_name, uuid
        );
    }

    fn downgrade(&self) -> PeerWeak {
        PeerWeak(Arc::downgrade(&self.0))
    }

    fn on_negotiation_needed(&self) -> Result<(), Error> {
        // TODO
    }

    fn on_offer_created(
        &self,
        reply: Result<Option<&gstreamer::StructureRef>, gstreamer::PromiseError>,
    ) -> Result<(), Error> {
        // TODO
    }

    fn on_answer_created(
        &self,
        reply: Result<Option<&gstreamer::StructureRef>, gstreamer::PromiseError>,
    ) -> Result<(), Error> {
        // TODO
    }

    fn handle_sdp(&self, type_: &str, sdp: &str) -> Result<(), Error> {
        if type_ == "answer" {
            // TODO
        } else if type_ == "offer" {
            // TODO
        } else {
            bail!("Sdp type is not \"answer\" but \"{}\"", type_)
        }
    }

    // Handle incoming ICE candidates from the peer by passing them to webrtcbin
    fn handle_ice(&self, sdp_mline_index: u32, candidate: &str) -> Result<(), Error> {
        // TODO

        Ok(())
    }

    fn on_ice_candidate(&self, mlineindex: u32, candidate: String) -> Result<(), Error> {
        // TODO

        Ok(())
    }

    fn on_incoming_stream(&self, pad: &gstreamer::Pad) -> Result<(), Error> {
        // TODO
    }
}

impl Drop for PeerInner {
    fn drop(&mut self) {
        let _ = self.bin.set_state(gstreamer::State::Null);
    }
}