var wigglypuffConnection;
var rtcConfiguration = { iceServers: [{ urls: "stun:global.stun.twilio.com:3478?transport=udp" }] };
let uuid = makeid(5);

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
            console.log(event);
        }
        peerConnection.onnegotiationneeded = event => {
            console.log(event);
        }
        peerConnection.onicecandidate = event => {
            if (event.candidate) {
                console.log("INI UUUDD", this.uuid);
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

    setLocalDescription(desc) {
        this.peerConnection.setLocalDescription(desc).then(() => {
            let sdp = this.peerConnection.localDescription;
            console.log("InIII", this.uuid);
            this.wigglypuffConnection.send(JSON.stringify({ uuid: this.uuid, type: sdp.type, sdp: sdp.sdp, action: "SessionDescription" }));
        });
    }

    setRemoteDescription(sdp) {
        console.log(this.peerConnection);
        this.peerConnection.setRemoteDescription(sdp).catch(this.setError);
        this.localStreamPromise.then((stream) => {
            this.peerConnection.createAnswer()
                .then(e => this.setLocalDescription(e)).catch(this.setError);

        }).catch(this.setError);


    }

    addICECandidate(candidates) {
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
                if (connections.length === 0) {
                    let connection = new Connection(uuid, rtcConfiguration, wigglypuffConnection);
                    connection.createRTCConnection();
                    connection.setRemoteDescription(message.data);
                    connections.push(connection);
                } else {
                    connections.forEach(connection => {
                        if (connection.uuid === message.uuid) {
                            connection.setRemoteDescription(message.data);
                        }
                    });
                }
                break;
            case "ICECandidate":
                console.log("PESANN", message);
                connections.forEach(connection => {
                    if (connection.uuid === message.uuid) {
                        console.log("ICE MASUKKKK", connection.uuid, message.uuid);
                        let candidates = {
                            candidate: message.data.candidate, sdpMLineIndex: message.data.sdpMLineIndex
                        };
                        connection.addICECandidate(candidates);
                    }
                });
                break;
        }
    }

    if (message.action) {
        console.log("ADAAA", message);
        if (message.action === "NewUser") {
            let connection = new Connection(message.uuid, rtcConfiguration, wigglypuffConnection);
            connection.createRTCConnection();
            connections.push(connection);
            console.log("MUNCUL BARU");
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
    var wigglypuffUrl = 'ws://cquran.my.id:443/api/room/join/dssn/' + uuid;
    wigglypuffConnection = new WebSocket(wigglypuffUrl);
    wigglypuffConnection.addEventListener('open', onWigglypuffConnect);
    wigglypuffConnection.addEventListener('error', onWigglypuffError);
    wigglypuffConnection.addEventListener('message', onWigglypuffMessage);
    wigglypuffConnection.addEventListener('close', onWigglypuffClose);
}


function onConnectClicked() {
    var clickAya = {
        'action': 'ClickAya',
        'aya': 1
    }
    wigglypuffConnection.send(JSON.stringify(clickAya));
}

function getVideoElement() {
    return document.getElementById("stream");
}

function onRemoteTrack(event) {
    if (getVideoElement().srcObject !== event.streams[0]) {
        console.log('Incoming stream');
        getVideoElement().srcObject = event.streams[0];
    }
}