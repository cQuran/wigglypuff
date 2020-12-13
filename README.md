# wigglypuff
distributed websocket &amp; webrtc media server

# Architecture
![arch](assets/architecture.png)

# Mobile Client
![arch](assets/mobile.png)

# API Contract
1. HTTP GET STUN/TURN Server (/network_transversal)
```
# Content-Type: application/json
# Status-Code: 200 OK

{
    "success": true,
    "data": {
        "stun": {
            "address": str
        },
        "turn": [
            {
                "address": str
                "username": str
                "credential": str
            },
            ...
        ]
    }
}
```