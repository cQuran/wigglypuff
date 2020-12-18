# wigglypuff
distributed websocket &amp; webrtc media server

# Design Proposals
![arch](assets/architecture.png)

# Concurrency Design
![arch](assets/actor-design.png)

# Mobile Client
![arch](assets/mobile.png)

# API Contract
1. HTTP GET STUN/TURN Server
```
# Endpoint: {url}/network_transversal
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
2. HTTP POST Create Room Channel & Websocket Room by Name

### request
```
# Endpoint: {url}/room/create
# Content-Type: application/json

{
	"name": str,
	"master_uuid": str
}
```

### response
```
# Content-Type: application/json
# Status-Code: 200 OK

{
    "success": true,
    "data": {
        "room_name": str
    }
}
```

3. HTTP GET All Room Channel

### response
```
# Endpoint: {url}/room/all
# Content-Type: application/json
# Status-Code: 200 OK

{
    "success": true,
    "data": [
        str,
        ...
    ]
```

4. HTTP POST Delete Room Channel & Websocket Room by Name

### request
```
# Endpoint: {url}/room/delete
# Content-Type: application/json

{
	"name": str
}
```

### response
```
# Content-Type: application/json
# Status-Code: 200 OK

{
    "success": true,
    "data": "room deleted"
}
```