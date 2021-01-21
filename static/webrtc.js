var wigglypuffConnection;
var rtcConfiguration = { iceServers: [{ urls: "stun:global.stun.twilio.com:3478?transport=udp" }] };
let uuid = makeid(5);
var uuid_new;

class Connection {
    constructor(uuid, rtcConfiguration, wigglypuffConnection) {
        this.uuid = uuid;
        this.rtcConfiguration = rtcConfiguration;
        this.wigglypuffConnection = wigglypuffConnection;
    }

    setError(text) {
        console.error(text);
    }

    createRTCConnection() {
        let peerConnection = new RTCPeerConnection(this.rtcConfiguration);
        peerConnection.ontrack = onRemoteTrack;
        peerConnection.oniceconnectionstatechange = event => {
            console.log("[" + this.uuid + "]", "[CONNECTION STATE CHANGE]", event.currentTarget.connectionState);
        }
        peerConnection.onnegotiationneeded = event => {
            console.log("[" + this.uuid + "]", "[NEGOTIATION NEED]");
        }
        peerConnection.onicecandidate = event => {
            console.log("[" + this.uuid + "]", "[ON ICE CANDIDATE]", event.currentTarget.connectionState);
            if (event.candidate) {
                this.wigglypuffConnection.send(JSON.stringify({ uuid: this.uuid, candidate: event.candidate.candidate, sdpMLineIndex: event.candidate.sdpMLineIndex, action: "ICECandidate" }));
            }
        };
        this.localStreamPromise = this.getLocalStream().then((stream) => {
            peerConnection.addStream(stream);
            return stream;
        }).catch(this.setError);

        this.peerConnection = peerConnection;
        return peerConnection;
    }

    setLocalDescription(desc, uuid) {
        console.log("[" + uuid + "]", "[SET LOCAL DESCRIPTION]");
        this.peerConnection.setLocalDescription(desc).then(() => {
            let sdp = this.peerConnection.localDescription;
            this.wigglypuffConnection.send(JSON.stringify({ uuid: uuid, type: sdp.type, sdp: sdp.sdp, action: "SessionDescription" }));
        });
    }

    setRemoteDescription(sdp, uuid) {
        console.log("[" + uuid + "]", "[SET REMOTE DESCRIPTION]");
        this.peerConnection.setRemoteDescription(sdp).catch(this.setError);
        this.localStreamPromise.then((stream) => {
            this.peerConnection.createAnswer()
                .then(e => this.setLocalDescription(e, uuid)).catch(this.setError);

        }).catch(this.setError);


    }

    addICECandidate(candidates) {
        console.log("[" + this.uuid + "]", "[ADD ICE CANDIDATE]");
        let ice = new RTCIceCandidate(candidates);
        this.peerConnection.addIceCandidate(ice).catch(e => {
            console.log("Failure during addIceCandidate(): " + e.name);
            console.log(e);
        });
    }

    getLocalStream() {
        var constraints = { video: false, audio: true };
        if (navigator.mediaDevices.getUserMedia) {
            return navigator.mediaDevices.getUserMedia(constraints);
        } else {
            this.setError("Browser doesn't support getUserMedia!");
        }
    }
}

let connections = [];

function onWigglypuffConnect() {
    console.log("CONNECT");

}
function onWigglypuffMessage(event) {
    var message = JSON.parse(event.data);
    if (message.data) {
        switch (message.data.action) {
            case "SessionDescription":
                let connection = new Connection(message.data.uuid, rtcConfiguration, wigglypuffConnection);
                connections.push(connection);
                connection.createRTCConnection();
                connection.setRemoteDescription(message.data, message.data.uuid);

                break;
            case "ICECandidate":
                let ice_exist = false;
                connections.forEach(connection => {
                    if (connection.uuid === message.data.uuid) {
                        let candidates = {
                            candidate: message.data.candidate, sdpMLineIndex: message.data.sdpMLineIndex
                        };
                        connection.addICECandidate(candidates);
                    }
                    ice_exist = true;
                });

                if (ice_exist === false) {
                    console.log("LOHH HEEEE");
                }
                break;
        }
    }

    if (message.action) {
        if (message.action === "NewUser") {
            console.log("[NEW]", message.uuid);
            uuid_new = message.uuid;
        }


    }

}

function onWigglypuffError(event) {
    console.log("ERROR");
}

function onWigglypuffClose(event) {
    console.log("CLOSED");
}
function makeid(length) {
    var result = '';
    var characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for (var i = 0; i < length; i++) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
}

function wigglypuffConnect() {
    var loc = window.location, new_uri;
    if (loc.protocol === "https:") {
        new_uri = "wss:";
    } else {
        new_uri = "ws:";
    }
    new_uri += "//" + loc.host;

    var wigglypuffUrl = new_uri + '/api/room/join/dssn/' + uuid;
    wigglypuffConnection = new WebSocket(wigglypuffUrl);
    wigglypuffConnection.addEventListener('open', onWigglypuffConnect);
    wigglypuffConnection.addEventListener('error', onWigglypuffError);
    wigglypuffConnection.addEventListener('message', onWigglypuffMessage);
    wigglypuffConnection.addEventListener('close', onWigglypuffClose);
}


function onConnectClicked() {
    console.log("[CLICK]", uuid_new);
    wigglypuffConnection.send(JSON.stringify({ uuid: uuid_new, action: "RequestPair" }));
}

function getVideoElement() {
    return document.getElementById("stream");
}

function onRemoteTrack(event) {
    console.log("[ON REMOTE TRACK]");
    if (getVideoElement().srcObject !== event.streams[0]) {
        console.log('Incoming stream');
        getVideoElement().srcObject = event.streams[0];
    }
}